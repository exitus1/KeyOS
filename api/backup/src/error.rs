// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later
use server::xous;

#[derive(Debug, thiserror::Error, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub enum Error {
    #[error("xous error: {:?}", xous::Error::from_usize(*.0))]
    Xous(usize),

    #[error("invalid backup file")]
    InvalidBackupFile,

    #[error(transparent)]
    CryptoError(#[from] crypto::error::CryptoError),

    #[error(transparent)]
    AccessDenied(#[from] security::AccessDenied),

    #[error(transparent)]
    Fs(#[from] fs::Error),

    #[error("io error: {0}")]
    Io(String),
}

impl From<xous::Error> for Error {
    fn from(e: xous::Error) -> Self { Error::Xous(e.to_usize()) }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self { Error::Io(e.to_string()) }
}
