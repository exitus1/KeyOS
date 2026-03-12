// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{error::Error, Status};

#[derive(Debug, server::Message, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[event(Status)]
pub struct StatusSubscribe;

#[derive(Debug, server::Message, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[response(Result<(), Error>)]
pub struct CreateBackup;

#[derive(Debug, server::Message, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[response(Result<(), Error>)]
pub struct CreateBackupFile {
    pub backup_path: String,
    pub location: fs::Location,
}

#[derive(Debug, server::Message, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[response(Result<(), Error>)]
pub struct RestoreBackup {
    pub backup_path: String,
    pub location: fs::Location,
}

#[derive(Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub enum RestoreProgress {
    NotFound,
    Downloading,
    Restoring,
    Restored,
    Error,
}

#[derive(Debug, server::Message, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[event(RestoreProgress)]
pub struct SubscribeRestoreProgress;
