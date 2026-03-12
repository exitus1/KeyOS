// SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use sandbox_test_worker::{TESTS, WORKER_SID};

pub fn main() {
    log_server::init_wait(env!("CARGO_CRATE_NAME")).unwrap();
    log::set_max_level(log::LevelFilter::Info);

    log::info!("Started");

    let server = xous::create_server_with_sid(WORKER_SID, 0..0xff).unwrap();
    let step = xous::receive_message(server).unwrap().body.id();
    let test = &TESTS[step];
    log::info!("Executing test: {}", test.name);
    (test.worker_fn)(server);
    log::info!("Exiting");
}
