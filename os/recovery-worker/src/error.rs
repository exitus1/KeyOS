// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

#[derive(Debug, Clone, Copy, thiserror::Error, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub enum RecoveryWorkerError {
    #[error("OS error: {:?}", xous::Error::from_usize(*.0))]
    Xous(usize),

    #[error("fs error: {0:?}")]
    Fs(#[from] fs::Error),

    #[error("access denied")]
    AccessDenied,

    #[error("Other error")]
    Other,
}

impl From<xous::Error> for RecoveryWorkerError {
    fn from(value: xous::Error) -> Self { RecoveryWorkerError::Xous(value.to_usize()) }
}
