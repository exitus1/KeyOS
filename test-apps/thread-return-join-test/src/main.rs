// SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

pub fn main() {
    log_server::init_wait(env!("CARGO_CRATE_NAME")).unwrap();
    log::set_max_level(log::LevelFilter::Info);

    log::info!("Starting");

    let thread = xous::create_thread_0(|| {
        // Give at least 2 slices to the main thread otherwise this thread will terminate before
        // the main thread gets a chance to call `wait_thread` thus producing no return value
        for _ in 0..2 {
            xous::yield_slice();
        }

        42
    })
    .expect("create thread");
    let res = xous::wait_thread(thread).expect("join thread");
    log::debug!("Returned from thread: {:?}", res);

    assert!(matches!(res, xous::Result::Scalar1(42)), "Unexpected return value from the wait_thread()");
    log::info!("Success");
}
