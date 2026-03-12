// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use server::{listen_and_connect, xous, CheckedConn, Server};

#[derive(server::Server)]
#[name = "test/disposable"]
pub struct DisposableServer;

pub struct DisposableServerHandle(CheckedConn<DisposablePermissions>);

impl Drop for DisposableServerHandle {
    fn drop(&mut self) { self.0.try_send_scalar(ShutdownDisposable).ok(); }
}

pub fn start_disposable_server() -> DisposableServerHandle {
    let server = DisposableServer;
    let pid = xous::current_pid().expect("current pid");
    DisposableServerHandle(listen_and_connect(server, pid).into())
}

#[derive(server::Message)]
#[response(u32)]
pub struct ScalarEcho(pub u32);

#[derive(server::Message, rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
#[response(u32)]
pub struct ArchiveEcho {
    pub value: u32,
}

#[derive(server::Message)]
struct ShutdownDisposable;

impl Server for DisposableServer {}

#[derive(Debug, Default, Clone, server::Permissions)]
#[server_name = "test/disposable"]
#[all_permissions]
pub struct DisposablePermissions;

impl server::BlockingScalarHandler<ScalarEcho> for DisposableServer {
    fn handle(&mut self, msg: ScalarEcho, _: xous::PID, _: &mut server::ServerContext<Self>) -> u32 { msg.0 }
}

impl server::ArchiveHandler<ArchiveEcho> for DisposableServer {
    fn handle(&mut self, msg: ArchiveEcho, _: xous::PID, _: &mut server::ServerContext<Self>) -> u32 {
        msg.value
    }
}

impl server::ScalarHandler<ShutdownDisposable> for DisposableServer {
    fn handle(
        &mut self,
        _msg: ShutdownDisposable,
        _sender: server::xous::PID,
        context: &mut server::ServerContext<Self>,
    ) {
        log::info!("shutting down disposable server");
        context.shutdown();
    }
}
