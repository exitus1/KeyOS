// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

mod disposable_server;
mod test_server;

use std::{
    future::Future,
    time::{Duration, Instant},
};

use futures_lite::{FutureExt, StreamExt};
use keyos_integration_test::{assert, assert_eq, fail};
use worker::{test_executor, WorkerHandle};

use crate::{
    disposable_server::DisposablePermissions,
    test_server::{api::TestData, *},
};

async fn test_basic_task_spawning(worker: &WorkerHandle) {
    let task1 = worker.spawn(async { 42 });
    let task2 = worker.spawn(async { 43 });

    assert_eq!(task1.await, 42);
    assert_eq!(task2.await, 43);
}

async fn test_performance(worker: &WorkerHandle) {
    let start = std::time::Instant::now();

    let mut tasks = vec![];
    for i in 0..100 {
        let task = worker.spawn(async move { i * 2 });
        tasks.push(task);
    }

    let mut results = vec![];
    for task in tasks {
        results.push(task.await)
    }

    let elapsed = start.elapsed();

    // Verify all results
    for (i, result) in results.iter().enumerate() {
        assert_eq!(*result, i * 2);
    }

    assert!(elapsed < std::time::Duration::from_millis(100), "performance test took too long");
}

async fn test_worker_sleep(worker: &WorkerHandle) {
    let start = Instant::now();

    let sleep2 = worker.spawn({
        let sleep = worker.sleep(Duration::from_millis(200));
        async move {
            sleep.await;
            log::info!("waking task 2");
            2
        }
    });

    let sleep1 = worker.spawn({
        let sleep = worker.sleep(Duration::from_millis(100));
        async move {
            sleep.await;
            log::info!("waking task 1");
            1
        }
    });

    let result = sleep1.race(sleep2).await;
    let elapsed = start.elapsed();
    // Hosted CI can introduce scheduler jitter; keep a bound that is strict enough
    // to catch regressions without flaking on normal contention.
    assert!(
        elapsed >= Duration::from_millis(100) && elapsed < Duration::from_millis(250),
        "elapsed {elapsed:?}"
    );
    assert_eq!(result, 1);

    // duration 0 doesn't break anything
    worker
        .spawn({
            let sleep = worker.sleep(Duration::from_nanos(100));
            async move {
                sleep.await;
                0
            }
        })
        .await;
}

async fn test_timeout_completes_before_timeout(worker: &WorkerHandle) {
    let result = worker
        .timeout(
            async {
                worker.sleep(Duration::from_millis(50)).await;
                42
            },
            Duration::from_millis(200),
        )
        .await;

    assert_eq!(result, Ok(42), "timeout should complete successfully when future finishes before timeout");
}

async fn test_timeout_expires(worker: &WorkerHandle) {
    let start = Instant::now();
    let result = worker
        .timeout(
            async {
                // never completes
                std::future::pending::<i32>().await
            },
            Duration::from_millis(100),
        )
        .await;
    let elapsed = start.elapsed();

    assert_eq!(
        result,
        Err(worker::TimeoutError),
        "timeout should expire when future doesn't complete in time"
    );

    assert!(
        elapsed >= Duration::from_millis(100) && elapsed < Duration::from_millis(250),
        "timeout elapsed {elapsed:?}"
    );
}

async fn test_scalar_subscription(worker: &WorkerHandle) {
    let sub_task = worker.spawn({
        let mut ticks = worker.subscribe_scalar::<TestPermissions, _>(api::ScalarTickSub);
        async move {
            let mut events = Vec::new();
            for _ in 0..5 {
                if let Some(event) = ticks.next().await {
                    events.push(event);
                }
            }
            events
        }
    });

    for _ in 0..5 {
        let _ = worker.async_archive::<TestPermissions, _>(api::ArchiveIncrementTick).await;
    }

    let events = sub_task.await;
    assert_eq!(events.len(), 5, "should receive 5 tick events");
}

async fn test_archive_subscription(worker: &WorkerHandle) {
    let sub_task = worker.spawn({
        let mut counter_events = worker.subscribe_archive::<TestPermissions, _>(api::ArchiveTickSub);
        async move {
            let mut events = Vec::new();
            for _ in 0..5 {
                if let Some(event) = counter_events.next().await {
                    events.push(event);
                }
            }
            events
        }
    });

    for _ in 0..5 {
        let _ = worker.async_archive::<TestPermissions, _>(api::ArchiveIncrementTick).await;
    }

    let events = sub_task.await;
    assert_eq!(events.len(), 5, "should receive 5 tick events");
}

async fn test_subscription_cancellation(worker: &WorkerHandle) {
    let scalar_ticks = worker.subscribe_scalar::<TestPermissions, _>(api::ScalarTickSub);
    let archive_ticks = worker.subscribe_archive::<TestPermissions, _>(api::ArchiveTickSub);

    let scalar_task = worker.spawn(async move { scalar_ticks.fold(0, |acc, _| acc + 1).await });

    let archive_task = worker.spawn(async move { archive_ticks.fold(0, |acc, _| acc + 1).await });

    // Send some ticks
    for _ in 0..3 {
        let _ = worker.async_archive::<TestPermissions, _>(api::ArchiveIncrementTick).await;
    }

    let drop_response = worker.async_archive::<TestPermissions, _>(api::ArchiveDropAllSubs).await;
    assert!(drop_response.dropped_count > 0, "should have dropped some subscribers");

    // Send more ticks (these should not be received)
    for _ in 0..3 {
        let _ = worker.async_archive::<TestPermissions, _>(api::ArchiveIncrementTick).await;
    }

    let scalar_count = scalar_task.await;
    let archive_count = archive_task.await;

    assert_eq!(scalar_count, 3, "should receive exactly 3 scalar events before cancellation");
    assert_eq!(archive_count, 3, "should receive exactly 3 archive events before cancellation");
}

async fn test_archive_async(worker: &WorkerHandle) {
    // we can have 16 in flight messages at the same time.
    let pending =
        (0..15).map(|_| worker.async_archive::<TestPermissions, _>(api::ArchiveTick)).collect::<Vec<_>>();
    let tick = worker.async_archive::<TestPermissions, _>(api::ArchiveIncrementTick).await;

    // this req will not be sent until a slot frees up
    let waiting = worker.async_archive::<TestPermissions, _>(api::ArchiveTick);
    assert!(!waiting.is_finished());

    for (ii, req) in pending.into_iter().enumerate() {
        let response = req.await;
        log::info!("received response {ii}");
        assert_eq!(response.tick, tick.tick, "Archive counter should match tick after increment");
    }

    let tick = worker.async_archive::<TestPermissions, _>(api::ArchiveIncrementTick).await;
    let response = waiting.await;
    assert_eq!(response.tick, tick.tick, "Archive counter should match tick after increment");
}

async fn test_archive_dropped_request(worker: &WorkerHandle) {
    let response = worker.async_archive::<TestPermissions, _>(api::ArchiveDrop);
    let result = response.await;
    assert!(result.is_none(), "Should return default value when server drops the request");
}

async fn test_fallible_subscriptions(worker: &WorkerHandle) {
    let scalar_success = worker
        .try_subscribe_scalar::<TestPermissions, _>(api::ScalarSubFallible { should_succeed: true })
        .await;
    match scalar_success {
        Ok(mut subscription) => {
            let _ = worker.async_archive::<TestPermissions, _>(api::ArchiveIncrementTick).await;
            let _ = worker.async_archive::<TestPermissions, _>(api::ArchiveIncrementTick).await;

            let first = subscription.next().await;
            let second = subscription.next().await;
            assert!(first.is_some(), "should receive first scalar event");
            assert!(second.is_some(), "should receive second scalar event");
        }
        Err(e) => {
            fail!("Expected successful scalar subscription but got error: {:?}", e);
        }
    }

    let scalar_fail = worker
        .try_subscribe_scalar::<TestPermissions, _>(api::ScalarSubFallible { should_succeed: false })
        .await;
    assert!(matches!(scalar_fail, Err(_)), "scalar subscription should fail");

    let archive_success = worker
        .try_subscribe_archive::<TestPermissions, _>(api::ArchiveSubFallible { should_succeed: true })
        .await;
    match archive_success {
        Ok(mut subscription) => {
            let _ = worker.async_archive::<TestPermissions, _>(api::ArchiveIncrementTick).await;
            let _ = worker.async_archive::<TestPermissions, _>(api::ArchiveIncrementTick).await;

            let first = subscription.next().await;
            let second = subscription.next().await;
            assert!(first.is_some(), "should receive first archive event");
            assert!(second.is_some(), "should receive second archive event");
        }
        Err(e) => {
            fail!("Expected successful archive subscription but got error: {:?}", e);
        }
    }

    let archive_fail = worker
        .try_subscribe_archive::<TestPermissions, _>(api::ArchiveSubFallible { should_succeed: false })
        .await;
    assert!(matches!(archive_fail, Err(_)), "archive subscription should fail");
}

async fn test_scalar_async(worker: &WorkerHandle) {
    let r1 = worker
        .async_scalar::<TestPermissions, _>(api::ScalarDoubleData(TestData { a: 1, b: 2, c: 3, d: false }))
        .await;
    let r2 = worker
        .async_scalar::<TestPermissions, _>(api::ScalarDoubleData(TestData { a: 10, b: 20, c: 30, d: true }))
        .await;

    assert_eq!(r1, TestData { a: 2, b: 4, c: 6, d: true });
    assert_eq!(r2, TestData { a: 20, b: 40, c: 60, d: false });
}

async fn test_scalar_async_dropped(worker: &WorkerHandle) {
    let response = worker.async_scalar::<TestPermissions, _>(test_server::api::ScalarDropRequest).await;
    assert_eq!(response, TestData::server_default());
}

async fn test_async_req_intense(worker: &WorkerHandle) {
    let ticks =
        (0..5).map(|_| worker.async_archive::<TestPermissions, _>(api::ArchiveTick)).collect::<Vec<_>>();
    let intervals =
        (0..40).map(|_| worker.async_scalar::<TestPermissions, _>(api::ScalarInterval)).collect::<Vec<_>>();
    for req in intervals {
        assert_eq!(req.await, 1, "all requests should resolve, despite more requests that slots");
    }

    assert!(!ticks.iter().any(|t| t.is_finished()), "none of these should be resolved yet");
    let count = worker.async_archive::<TestPermissions, _>(api::ArchiveIncrementTick).await;
    for tick in ticks {
        assert_eq!(tick.await.tick, count.tick);
    }
}

async fn test_try_async_with_dead_server(worker: &WorkerHandle) {
    let disposable = disposable_server::start_disposable_server();

    let echo_result =
        worker.async_scalar::<DisposablePermissions, _>(disposable_server::ScalarEcho(42)).await;
    assert_eq!(echo_result, 42, "disposable server echo");

    log::info!("dropping disposable server");
    drop(disposable);
    // some time to make sure server is shutdown
    std::thread::sleep(Duration::from_millis(500));
    log::info!("dropped disposable server");

    let scalar_err =
        worker.try_async_scalar::<DisposablePermissions, _>(disposable_server::ScalarEcho(123)).await;
    log::info!("received scalar message");
    assert_eq!(
        scalar_err,
        Err(server::xous::Error::ServerNotFound),
        "scalar message to dead server should fail"
    );

    let archive_err = worker
        .try_async_archive::<DisposablePermissions, _>(disposable_server::ArchiveEcho { value: 456 })
        .await;

    assert_eq!(
        archive_err,
        Err(server::xous::Error::ServerNotFound),
        "archive message to dead server should fail"
    );
}

async fn test_pending_connection_retry(worker: &WorkerHandle) {
    mod retry_server {
        use server::{listen_and_connect, xous, CheckedConn, Server};

        #[derive(server::Server)]
        #[name = "test/retry"]
        pub struct RetryServer;

        pub struct RetryServerHandle(CheckedConn<RetryPermissions>);

        impl Drop for RetryServerHandle {
            fn drop(&mut self) { self.0.try_send_scalar(ShutdownRetry).ok(); }
        }

        pub fn start_retry_server() -> RetryServerHandle {
            let server = RetryServer;
            let pid = xous::current_pid().expect("current pid");
            RetryServerHandle(listen_and_connect(server, pid).into())
        }

        #[derive(server::Message)]
        #[response(u32)]
        pub struct RetryEcho(pub u32);

        #[derive(server::Message)]
        struct ShutdownRetry;

        impl Server for RetryServer {}

        #[derive(Debug, Default, Clone, server::Permissions)]
        #[server_name = "test/retry"]
        #[all_permissions]
        pub struct RetryPermissions;

        impl server::BlockingScalarHandler<RetryEcho> for RetryServer {
            fn handle(&mut self, msg: RetryEcho, _: xous::PID, _: &mut server::ServerContext<Self>) -> u32 {
                msg.0
            }
        }

        impl server::ScalarHandler<ShutdownRetry> for RetryServer {
            fn handle(
                &mut self,
                _msg: ShutdownRetry,
                _: server::xous::PID,
                ctx: &mut server::ServerContext<Self>,
            ) {
                ctx.shutdown();
            }
        }
    }

    let _ = worker.async_archive::<TestPermissions, _>(api::ArchiveIncrementTick).await;
    assert!(!worker.get_retry_timer_active(), "should not have callback active");

    let pending_request =
        worker.async_scalar::<retry_server::RetryPermissions, _>(retry_server::RetryEcho(100));
    std::thread::sleep(Duration::from_millis(50));
    let immediate_request = worker
        .async_scalar::<TestPermissions, _>(api::ScalarDoubleData(TestData { a: 5, b: 10, c: 15, d: false }))
        .await;

    assert_eq!(
        immediate_request,
        TestData { a: 10, b: 20, c: 30, d: true },
        "worker can fulfill other requests while connection is blocked"
    );
    assert!(worker.get_retry_timer_active(), "should not have callback active yet");

    let _retry_server = retry_server::start_retry_server();
    let result = pending_request.await;

    assert!(!worker.get_retry_timer_active(), "callback should be cleared");
    assert_eq!(result, 100);
}

async fn test_clone_subscription(worker: &WorkerHandle) {
    let counter_events = worker.subscribe_archive::<TestPermissions, _>(api::ArchiveTickSub);

    let sub_task_1 = worker.spawn({
        let mut counter_events = counter_events.clone();
        async move {
            let mut events = Vec::new();
            for _ in 0..5 {
                if let Some(event) = counter_events.next().await {
                    events.push(event);
                }
            }
            events
        }
    });

    let sub_task_2 = worker.spawn({
        let mut counter_events = counter_events.clone();
        async move {
            let mut events = Vec::new();
            for _ in 0..5 {
                if let Some(event) = counter_events.next().await {
                    events.push(event);
                }
            }
            events
        }
    });

    for _ in 0..5 {
        let _ = worker.async_archive::<TestPermissions, _>(api::ArchiveIncrementTick).await;
    }

    assert_eq!(sub_task_1.await.len(), 5, "should receive 5 tick events");

    for _ in 0..5 {
        let _ = worker.async_archive::<TestPermissions, _>(api::ArchiveIncrementTick).await;
    }

    assert_eq!(sub_task_2.await.len(), 5, "should receive 5 tick events");
}

fn run_test<F>(test_name: &str, test: F)
where
    F: Future<Output = ()>,
{
    let start = std::time::Instant::now();
    match test_executor::block_timeout(test, Duration::from_secs(5)) {
        Some(_) => {
            let elapsed = start.elapsed();
            log::info!("✓ {} test passed (elapsed: {:?})", test_name, elapsed);
        }
        None => {
            fail!("test timed out {test_name}");
        }
    }
}

fn main() {
    log_server::init_wait(env!("CARGO_CRATE_NAME")).unwrap();
    log::set_max_level(log::LevelFilter::Debug);

    log::info!("Starting async worker integration tests...\n");

    let worker = WorkerHandle::default();

    run_test("basic task spawning", test_basic_task_spawning(&worker));
    run_test("performance", test_performance(&worker));
    run_test("sleep", test_worker_sleep(&worker));

    let _test_server = test_server::start_test_server();

    run_test("scalar subscription", test_scalar_subscription(&worker));
    run_test("archive subscription", test_archive_subscription(&worker));
    run_test("subscription cancellation", test_subscription_cancellation(&worker));
    run_test("fallible subscriptions", test_fallible_subscriptions(&worker));
    run_test("double consume stream", test_clone_subscription(&worker));

    run_test("archive async", test_archive_async(&worker));
    run_test("archive dropped request", test_archive_dropped_request(&worker));
    run_test("scalar async", test_scalar_async(&worker));
    run_test("scalar async dropped", test_scalar_async_dropped(&worker));
    run_test("async request intense", test_async_req_intense(&worker));

    run_test("pending connection retry", test_pending_connection_retry(&worker));

    run_test("try async with dead server", test_try_async_with_dead_server(&worker));

    run_test("timeout completes before timeout", test_timeout_completes_before_timeout(&worker));
    run_test("timeout expires", test_timeout_expires(&worker));

    log::info!("\nAll async worker integration tests passed successfully!");

    keyos_integration_test::pass();
}
