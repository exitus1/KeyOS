// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later
#![cfg(keyos)]

#[cfg(keyos)]
use std::rc::Rc;

use server::ScalarEventSubscriber;
use server::Server;

use crate::error::RecoveryWorkerError;

pub mod error;

pub mod api;
pub mod messages;
mod recovery;
mod system_info;
mod utils;

pub use messages::*;

use crate::recovery::{ArchiveState as RecoveryArchiveState, RecoveryState};

crypto::use_api!();
fs::use_api!();

#[macro_export]
macro_rules! use_api {
    () => {
        mod recovery_worker_permissions {
            use recovery_worker::messages::*;
            #[derive(Clone, Default, server::Permissions)]
            #[server_name = "os/recovery-worker"]
            pub struct RecoveryWorkerPermissions;
        }
        type RecoveryWorkerApi =
            recovery_worker::api::RecoveryWorkerApi<recovery_worker_permissions::RecoveryWorkerPermissions>;
    };
}

pub fn listen() { server::listen(RecoveryWorkerServer::new().unwrap()) }

#[derive(Default, server::Server)]
#[name = "os/recovery-worker"]
pub struct RecoveryWorkerServer {
    #[cfg(keyos)]
    progress_subscriber: Option<Rc<ScalarEventSubscriber<Progress>>>,
    #[cfg(keyos)]
    archive_state: RecoveryArchiveState,
    #[cfg(keyos)]
    os_binary_state: RecoveryState,
    #[cfg(keyos)]
    bootloader_state: RecoveryState,
    #[cfg(keyos)]
    fs: FileSystem,
    #[cfg(keyos)]
    crypto: CryptoApi,
    #[cfg(keyos)]
    last_error: Option<String>,
}

impl Server for RecoveryWorkerServer {}

impl RecoveryWorkerServer {
    pub fn new() -> Result<Self, RecoveryWorkerError> { Ok(Self::default()) }
}
