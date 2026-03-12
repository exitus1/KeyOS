// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later
#[cfg(keyos)]
mod implementation;

#[cfg(keyos)]
pub fn main() {
    log_server::init_wait(env!("CARGO_CRATE_NAME")).unwrap();
    log::set_max_level(log::LevelFilter::Info);

    xous::set_thread_priority(xous::ThreadPriority::System6).unwrap();

    let spi_server = implementation::SpiServer::init();
    server::listen(spi_server)
}

// This server is not needed in hosted mode
#[cfg(not(keyos))]
pub fn main() {}
