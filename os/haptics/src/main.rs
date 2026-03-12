// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use log::debug;

#[cfg(keyos)]
mod atsama5d2;
#[cfg(keyos)]
use atsama5d2::Implementation;

#[cfg(not(keyos))]
mod hosted;
use haptics::messages::*;
#[cfg(not(keyos))]
use hosted::Implementation;
use server::{ScalarHandler, Server};

#[derive(server::Server)]
#[name = "os/haptics"]
pub struct HapticsServer {
    implementation: Implementation,
}

impl HapticsServer {
    pub fn new() -> Self { Self { implementation: Implementation::init() } }
}

impl Server for HapticsServer {}

impl ScalarHandler<Vibrate> for HapticsServer {
    fn handle(
        &mut self,
        Vibrate(pattern): Vibrate,
        _sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) {
        debug!("Vibrating the pattern: {:?}", pattern);
        self.implementation.vibrate(pattern)
    }
}

fn main() {
    log_server::init_wait(env!("CARGO_CRATE_NAME")).unwrap();
    log::set_max_level(log::LevelFilter::Info);

    xous::set_thread_priority(xous::ThreadPriority::System3).unwrap();

    server::listen(HapticsServer::new())
}
