// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use fs::Location;
use num_traits::FromPrimitive;
use server::{AsScalar, FromScalar};

use crate::recovery::RecoveryState;

pub const DOWNGRADE_NOT_ALLOWED_MSG: &str =
    "The selected firmware is older than the installed firmware. Rollback is not supported.";

#[derive(Debug, Clone, server::Message, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[response(())]
pub struct ReadArchive {
    pub path: String,
    pub location: Location,
}

#[derive(Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub enum ReadArchiveError {
    /// The recovery achieved is not a tar or corrupted.
    UnsupportedFormat,
    /// The recovery achieved is a tar, but some of the required files are missing.
    MissingRequiredFiles,
    /// The firmware file is valid, but installation is denied due to rollback policy.
    DowngradeNotAllowed,
    /// An unknown error occurred while reading the archive.
    InternalError(String),
}

#[derive(Debug, Clone, server::Message, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[response(ArchiveState)]
pub struct GetArchiveState;

#[derive(Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub enum ArchiveState {
    None,

    Ok {
        /// KeyOS version
        version: String,
        /// Number of apps to be recovered
        num_apps: usize,
        /// Number of assets to be recovered
        num_assets: usize,
        /// Whether the archive is already existing in the system
        is_existing: bool,
        os_binary: String,
        has_recovery_or_bootloader: bool,
    },

    Error(ReadArchiveError),
}

impl From<crate::recovery::ArchiveState> for ArchiveState {
    fn from(value: crate::recovery::ArchiveState) -> Self {
        match value {
            crate::recovery::ArchiveState::None => ArchiveState::None,
            crate::recovery::ArchiveState::ValidArchive(crate::recovery::ValidArchive {
                version,
                apps,
                assets,
                os_binary,
                bootloader,
                ..
            }) => ArchiveState::Ok {
                version,
                num_apps: apps.len(),
                num_assets: assets.len(),
                has_recovery_or_bootloader: bootloader.is_some() || os_binary == "recovery.bin",
                os_binary,
                is_existing: false,
            },
            crate::recovery::ArchiveState::CopiedArchive(crate::recovery::ValidArchive {
                version,
                apps,
                assets,
                os_binary,
                bootloader,
                ..
            }) => ArchiveState::Ok {
                version,
                num_apps: apps.len(),
                num_assets: assets.len(),
                has_recovery_or_bootloader: bootloader.is_some() || os_binary == "recovery.bin",
                os_binary,
                is_existing: true,
            },
            crate::recovery::ArchiveState::Error(error) => ArchiveState::Error(error),
        }
    }
}

#[derive(Debug, Copy, Clone, num_derive::FromPrimitive, num_derive::ToPrimitive)]
#[repr(u32)]
pub enum ProgressKind {
    ArchiveRead = 0,
    ArchiveCopy,
    AppBinVerify,
    Extracting,
    RebootCountdown,

    #[doc(hidden)]
    Unknown,
}

#[derive(Debug, Copy, Clone)]
pub struct Progress {
    pub kind: ProgressKind,
    pub is_completed: bool,
    pub is_error: bool,
    pub progress: f32,
}

impl AsScalar<4> for Progress {
    fn as_scalar(&self) -> [u32; 4] {
        [
            self.kind as u32,
            if self.is_completed { 1 } else { 0 },
            if self.is_error { 1 } else { 0 },
            self.progress.to_bits(),
        ]
    }
}

impl FromScalar<4> for Progress {
    fn from_scalar([kind, is_completed, is_error, progress]: [u32; 4]) -> Self {
        Self {
            kind: ProgressKind::from_u32(kind).unwrap_or(ProgressKind::Unknown),
            is_completed: is_completed != 0,
            is_error: is_error != 0,
            progress: f32::from_bits(progress),
        }
    }
}

#[derive(Debug, Clone, server::Message, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[event(Progress)]
pub struct SubscribeProgress;

#[derive(Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub enum AppBinVerificationState {
    None,
    Invalid(String),
    Valid {
        hash: String,
        version: String,
        build_date: String,
        fw_file_hash: [u8; 32],
        fw_file_total_size: usize,
    },
    Copied,
}

impl From<&RecoveryState> for AppBinVerificationState {
    fn from(value: &RecoveryState) -> Self {
        match value {
            RecoveryState::None => AppBinVerificationState::None,
            RecoveryState::Invalid(err) => AppBinVerificationState::Invalid(err.clone()),
            RecoveryState::Valid { hash, version, build_date, fw_file_hash, fw_file_total_size, .. } => {
                AppBinVerificationState::Valid {
                    hash: hash.clone(),
                    version: version.clone(),
                    build_date: build_date.clone(),
                    fw_file_hash: *fw_file_hash,
                    fw_file_total_size: *fw_file_total_size,
                }
            }
            RecoveryState::Copied { .. } => AppBinVerificationState::Copied,
        }
    }
}

#[derive(Debug, Clone, server::Message, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[response(AppBinVerificationState)]
pub struct GetAppBinVerificationState;

#[derive(Debug, server::Message)]
pub struct CopyArchive;

#[derive(Debug, server::Message)]
pub struct StartRecovery;

#[derive(Debug, Clone, server::Message, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[response(Option<String>)]
pub struct GetLastError;
