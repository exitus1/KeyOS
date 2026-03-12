// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

pub use {
    response::Response,
    sleep::Sleep,
    subscription::Subscription,
    task::TaskHandle,
    timeout::{Timeout, TimeoutError},
    watch::StreamWatch,
};

mod implementation;
mod response;
mod sleep;
mod subscription;
mod task;
mod timeout;
mod watch;

use std::{
    future::IntoFuture,
    panic::Location,
    sync::{Arc, OnceLock},
    time::Duration,
};

use server::{xous, AsyncMessageInit, EventSubscriptionMessage};

use crate::implementation::{WorkerApiInner, WorkerEvent, WorkerServer};

/// A handle to the worker runtime.
///
/// On drop, will terminate the worker thread
/// cheaply cloneable api to the worker runtime
#[derive(Default, Debug, Clone)]
pub struct WorkerHandle {
    inner: OnceLock<Arc<WorkerApiInner>>,
}

impl WorkerHandle {
    pub fn spawn<I, T>(&self, f: I) -> TaskHandle<T>
    where
        I: IntoFuture<Output = T>,
        <I as IntoFuture>::IntoFuture: Send + 'static,
        T: Send + 'static,
    {
        let handle = Arc::downgrade(self.inner());
        let schedule = move |task| {
            if let Some(handle) = handle.upgrade() {
                handle.queue_event(WorkerEvent::Task { task });
            }
        };
        let (r, task) = async_task::spawn(f.into_future(), schedule);
        r.schedule();
        TaskHandle { task }
    }

    pub fn sleep(&self, duration: Duration) -> Sleep {
        let handle = Arc::downgrade(self.inner());
        Sleep::new(duration, handle)
    }

    pub fn timeout<F>(&self, future: F, duration: Duration) -> Timeout<F>
    where
        F: Future,
    {
        let handle = Arc::downgrade(self.inner());
        Timeout::new(future, duration, handle)
    }

    #[track_caller]
    pub fn subscribe_scalar<P, M>(&self, sub: M) -> Subscription<M::Event>
    where
        M: server::ScalarSubscription<Error = server::Infallible> + Send + 'static,
        P: server::CheckedPermissions + server::MessageAllowed<M>,
    {
        self.subscribe::<M, M::Event>(
            sub,
            <M as server::MessageId>::SERVER,
            subscribe_scalar_raw::<M>,
            server::decode_scalar_event::<M::Event>,
        )
    }

    pub fn try_subscribe_scalar<P, M>(&self, sub: M) -> TaskHandle<Result<Subscription<M::Event>, M::Error>>
    where
        M: server::ScalarSubscription + Send + 'static,
        M::Error: Send + 'static,
        M::Event: Send + 'static,
        P: server::CheckedPermissions + server::MessageAllowed<M>,
    {
        self.try_subscribe::<M, M::Event, M::Error>(
            sub,
            M::SERVER,
            subscribe_scalar_raw::<M>,
            server::decode_scalar_event::<M::Event>,
        )
    }

    pub fn subscribe_archive<P, M>(&self, sub: M) -> Subscription<M::Event>
    where
        M: server::ArchiveSubscription<Error = server::Infallible> + Send + 'static,
        P: server::CheckedPermissions + server::MessageAllowed<M>,
    {
        self.subscribe::<M, M::Event>(
            sub,
            M::SERVER,
            subscribe_archive_raw::<M>,
            server::decode_archive_event::<M::Event>,
        )
    }

    pub fn try_subscribe_archive<P, M>(&self, sub: M) -> TaskHandle<Result<Subscription<M::Event>, M::Error>>
    where
        M: server::ArchiveSubscription + Send + 'static,
        M::Error: Send + 'static,
        M::Event: Send + 'static,
        P: server::CheckedPermissions + server::MessageAllowed<M>,
    {
        self.try_subscribe::<M, M::Event, M::Error>(
            sub,
            M::SERVER,
            subscribe_archive_raw::<M>,
            server::decode_archive_event::<M::Event>,
        )
    }

    /// Send an Archive message asynchronously.
    #[track_caller]
    pub fn async_archive<P, M>(&self, msg: M) -> Response<M::Response>
    where
        M: server::Archive + Send + 'static,
        P: server::CheckedPermissions + server::MessageAllowed<M>,
    {
        self.async_request(msg, M::SERVER, send_archive_raw::<M>, |res, location| {
            let envelope = match res {
                Ok(envelope) => envelope,
                Err(e) => panic!("async_archive {location} {e:?}"),
            };
            match server::try_decode_archive_async_response::<M::Response>(envelope) {
                Ok(resp) => resp,
                Err(e) => panic!("async_archive decode {location} {e}"),
            }
        })
    }

    /// Send an Archive message asynchronously, retaining the error channel
    pub fn try_async_archive<P, M>(&self, msg: M) -> Response<Result<M::Response, xous::Error>>
    where
        M: server::Archive + Send + 'static,
        P: server::CheckedPermissions + server::MessageAllowed<M>,
    {
        self.async_request(msg, M::SERVER, send_archive_raw::<M>, |res, _| {
            let msg = res?;
            server::try_decode_archive_async_response::<M::Response>(msg)
                .map_err(|e| e.into_inner().into_xous())
        })
    }

    /// Send a Scalar message asynchronously.
    #[track_caller]
    pub fn async_scalar<P, M>(&self, msg: M) -> Response<M::Response>
    where
        M: server::BlockingScalar + Send + 'static,
        P: server::CheckedPermissions + server::MessageAllowed<M>,
    {
        self.async_request(msg, M::SERVER, send_scalar_raw, |res, location| {
            let envelope = match res {
                Ok(envelope) => envelope,
                Err(e) => panic!("async_scalar {location} {e:?}"),
            };
            server::decode_scalar_async_response::<M::Response>(envelope)
        })
    }

    /// Send a Scalar message asynchronously, retaining the error channel
    pub fn try_async_scalar<P, M>(&self, msg: M) -> Response<Result<M::Response, xous::Error>>
    where
        M: server::BlockingScalar + Send + 'static,
        P: server::CheckedPermissions + server::MessageAllowed<M>,
    {
        self.async_request(msg, M::SERVER, send_scalar_raw, |res, _| {
            let msg = res?;
            Ok(server::decode_scalar_async_response::<M::Response>(msg))
        })
    }

    pub fn watch_stream<S, T>(&self, sub: S, initial: T) -> StreamWatch<T>
    where
        T: Clone + Send + Sync + 'static,
        S: futures_lite::Stream + Send + 'static,
        S::Item: Into<T> + Send + 'static,
    {
        StreamWatch::from_stream(self, sub, initial)
    }
}

type SubInit<M, E> = fn(
    xous::CID,
    xous::CID,
    xous::MessageId,
    xous::MessageId,
    M,
) -> whence::Result<Result<(), E>, server::Error>;

type AsyncInit<M> = fn(xous::CID, xous::CID, xous::MessageId, M) -> whence::Result<(), server::Error>;

impl WorkerHandle {
    #[inline]
    fn inner(&self) -> &Arc<WorkerApiInner> { self.inner.get_or_init(|| Arc::new(WorkerApiInner::default())) }

    #[inline]
    fn subscribe<M, Event>(
        &self,
        msg: M,
        server_name: &'static str,
        sub: SubInit<M, server::Infallible>,
        decode: fn(xous::MessageEnvelope) -> Event,
    ) -> Subscription<Event>
    where
        M: Send + 'static,
        Event: 'static,
    {
        let (tx, rx) = async_channel::bounded(10);

        let mut state = Some((msg, tx));
        let init = Box::new(move |s: &mut WorkerServer| {
            let (cid, cid_remote, pid) = s.try_get_connection_from_name(server_name).unwrap()?;
            let (msg_id, cancel_msg_id) = s.get_available_subscription_ids()?;
            let (msg, tx) = state.take().unwrap();
            match sub(cid, cid_remote, msg_id, cancel_msg_id, msg).unwrap() {
                Ok(()) => (),
                Err(e) => match e {},
            };
            s.insert_subscription(msg_id, implementation::ActiveSubscription { tx, pid });
            Some(())
        });

        self.inner().queue_event(WorkerEvent::Register { init });

        Subscription { rx, decode }
    }

    #[inline]
    fn try_subscribe<M, Event, Err>(
        &self,
        msg: M,
        server_name: &'static str,
        sub: SubInit<M, Err>,
        decode: fn(xous::MessageEnvelope) -> Event,
    ) -> TaskHandle<Result<Subscription<Event>, Err>>
    where
        M: Send + 'static,
        Err: Send + 'static,
        Event: Send + 'static,
    {
        let this = self.clone();
        self.spawn(async move {
            let (tx, rx) = async_channel::bounded(10);
            let (result_tx, result_rx) = oneshot::channel();

            let mut state = Some((msg, tx, result_tx));
            let init = Box::new(move |s: &mut WorkerServer| {
                // TODO: probably should surface the xous error on the try variant
                let (cid, cid_remote, pid) = s.try_get_connection_from_name(server_name).unwrap()?;
                let (msg_id, cancel_msg_id) = s.get_available_subscription_ids()?;
                let (msg, tx, result_tx) = state.take().unwrap();
                // we are doing this regardless of the outcome
                // due to cancellation (via drop from handler server)
                // triggering the cleanup to evict the slot
                s.insert_subscription(msg_id, implementation::ActiveSubscription { tx, pid });
                let res = sub(cid, cid_remote, msg_id, cancel_msg_id, msg);
                let _ = result_tx.send(res);
                Some(())
            });

            this.inner().queue_event(WorkerEvent::Register { init });
            result_rx.await.unwrap().unwrap()?;
            Ok(Subscription { rx, decode })
        })
    }

    #[inline]
    #[track_caller]
    fn async_request<M, R>(
        &self,
        msg: M,
        server_name: &'static str,
        async_fn: AsyncInit<M>,
        decode: fn(Result<xous::MessageEnvelope, xous::Error>, &'static Location<'static>) -> R,
    ) -> Response<R>
    where
        M: Send + 'static,
        R: 'static,
    {
        let location = Location::caller();
        let (tx, rx) = oneshot::channel();

        let mut state = Some((msg, tx));
        let init = Box::new(move |s: &mut WorkerServer| {
            let (cid, cid_remote, pid) = match s.try_get_connection_from_name(server_name) {
                Ok(ids) => ids?,
                Err(e) => {
                    let (_, tx) = state.take().unwrap();
                    let _ = tx.send(Err(e));
                    return Some(());
                }
            };
            let msg_id = s.get_available_request_id()?;
            let (msg, tx) = state.take().unwrap();
            match async_fn(cid, cid_remote, msg_id, msg) {
                Ok(()) => {
                    s.insert_request(msg_id, implementation::PendingRequest { tx: Some(tx), pid });
                }
                Err(e) => {
                    let _ = tx.send(Err(e.into_inner().into_xous()));
                }
            }
            Some(())
        });

        self.inner().queue_event(WorkerEvent::Register { init });

        Response { rx, decode, location }
    }

    #[cfg(feature = "integration_test")]
    pub fn get_retry_timer_active(&self) -> bool {
        self.inner().conn.send_blocking_scalar(implementation::GetRetryTimerActive)
    }
}

fn send_archive_raw<M: server::Archive>(
    cid: xous::CID,
    remote_cid: xous::CID,
    msg_id: xous::MessageId,
    msg: M,
) -> whence::Result<(), server::Error> {
    AsyncMessageInit { cid: remote_cid, msg_id, msg }.send_archive(cid)
}

fn send_scalar_raw<M: server::BlockingScalar>(
    cid: xous::CID,
    remote_cid: xous::CID,
    msg_id: xous::MessageId,
    msg: M,
) -> whence::Result<(), server::Error> {
    AsyncMessageInit { cid: remote_cid, msg_id, msg }.send_scalar(cid)
}

fn subscribe_scalar_raw<M: server::ScalarSubscription>(
    cid: xous::CID,
    remote_cid: xous::CID,
    msg_id: xous::MessageId,
    cancel_msg_id: xous::MessageId,
    msg: M,
) -> whence::Result<Result<(), M::Error>, server::Error> {
    EventSubscriptionMessage { cid: remote_cid, msg_id, cancel_msg_id, msg }.send_scalar(cid)
}

fn subscribe_archive_raw<M: server::ArchiveSubscription>(
    cid: xous::CID,
    remote_cid: xous::CID,
    msg_id: xous::MessageId,
    cancel_msg_id: xous::MessageId,
    msg: M,
) -> whence::Result<Result<(), M::Error>, server::Error> {
    EventSubscriptionMessage { cid: remote_cid, msg_id, cancel_msg_id, msg }.send_archive(cid)
}

/// Code from https://doc.rust-lang.org/std/task/trait.Wake.html#examples
#[cfg(feature = "test_executor")]
pub mod test_executor {
    use std::future::Future;
    use std::sync::Arc;
    use std::task::{Context, Poll, Wake};
    use std::thread::{self, Thread};
    use std::time::{Duration, Instant};

    /// A waker that wakes up the current thread when called.
    struct ThreadWaker(Thread);

    impl Wake for ThreadWaker {
        fn wake(self: Arc<Self>) { self.0.unpark(); }
    }

    /// Run a future to completion on the current thread.
    pub fn block_on<T>(fut: impl Future<Output = T>) -> T {
        // Pin the future so it can be polled.
        let mut fut = Box::pin(fut);

        // Create a new context to be passed to the future.
        let t = thread::current();
        let waker = Arc::new(ThreadWaker(t)).into();
        let mut cx = Context::from_waker(&waker);

        // Run the future to completion.
        loop {
            match fut.as_mut().poll(&mut cx) {
                Poll::Ready(res) => return res,
                Poll::Pending => thread::park(),
            }
        }
    }

    pub fn block_timeout<T>(fut: impl Future<Output = T>, duration: Duration) -> Option<T> {
        let start = Instant::now();
        // Pin the future so it can be polled.
        let mut fut = Box::pin(fut);

        // Create a new context to be passed to the future.
        let t = thread::current();
        let waker = Arc::new(ThreadWaker(t)).into();
        let mut cx = Context::from_waker(&waker);

        // Run the future to completion.
        loop {
            let elapsed = start.elapsed();
            let remaining = duration.saturating_sub(elapsed);
            if remaining.is_zero() {
                return None;
            }
            match fut.as_mut().poll(&mut cx) {
                Poll::Ready(res) => return Some(res),
                Poll::Pending => thread::park_timeout(remaining),
            }
        }
    }
}
