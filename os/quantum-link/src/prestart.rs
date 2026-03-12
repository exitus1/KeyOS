// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use quantum_link::messages::StartWithoutFilesystem;

use crate::FileSystem;

#[derive(server::Server)]
#[name = "os/ql-prestart"]
pub struct QuantumLinkPrestartServer;

impl server::Server for QuantumLinkPrestartServer {
    fn on_start(&mut self, context: &mut server::ServerContext<Self>) {
        FileSystem::default().subscribe_filesystem_events(context, fs::Location::AppData);
    }
}

impl server::ScalarEventHandler<fs::FileSystemEvent> for QuantumLinkPrestartServer {
    fn handle(
        &mut self,
        msg: fs::FileSystemEvent,
        _sender: xous::PID,
        context: &mut server::ServerContext<Self>,
    ) {
        if msg.location == fs::Location::AppData && msg.event_type == fs::FileSystemEventType::Mounted {
            context.shutdown();
        }
    }
}

impl server::BlockingScalarHandler<StartWithoutFilesystem> for QuantumLinkPrestartServer {
    fn handle(
        &mut self,
        _msg: StartWithoutFilesystem,
        _sender: xous::PID,
        context: &mut server::ServerContext<Self>,
    ) -> <StartWithoutFilesystem as server::BlockingScalar>::Response {
        context.shutdown();
    }
}
