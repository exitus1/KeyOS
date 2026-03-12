// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

mod implementation;

#[cfg(keyos)]
use implementation::start_server;

#[cfg(keyos)]
fn main() {
    log_server::init_wait(env!("CARGO_CRATE_NAME")).unwrap();
    log::set_max_level(log::LevelFilter::Info);

    xous::set_thread_priority(xous::ThreadPriority::System6).unwrap();

    start_server()
}

// This server is not needed in hosted mode
#[cfg(not(keyos))]
pub fn main() {}
