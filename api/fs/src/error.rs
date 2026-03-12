// SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};

#[derive(
    Debug, Clone, Copy, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, ToPrimitive, FromPrimitive,
)]
pub enum Error {
    Io = 1,
    FileNotOpen,
    InvalidBufferLength,
    InvalidPath,
    InvalidOperation,
    FileNotFound,
    FileAlreadyExists,
    FileInUse,
    NotADirectory,
    NoMedia,
    OutOfMemory,
    AccessDenied,
    InternalError,
}

impl server::AsScalar<1> for Error {
    fn as_scalar(&self) -> [u32; 1] { [self.to_u32().unwrap()] }
}

impl server::FromScalar<1> for Error {
    fn from_scalar([value]: [u32; 1]) -> Self { Self::from_u32(value).unwrap_or(Self::InternalError) }
}

impl From<Error> for usize {
    fn from(value: Error) -> Self { value.to_usize().unwrap() }
}

impl From<usize> for Error {
    fn from(value: usize) -> Self { Self::from_usize(value).unwrap_or(Self::InternalError) }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{self:?}") }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        match e.kind() {
            std::io::ErrorKind::NotFound => Error::FileNotFound,
            std::io::ErrorKind::PermissionDenied => Error::AccessDenied,
            std::io::ErrorKind::InvalidFilename => Error::InvalidPath,
            std::io::ErrorKind::Unsupported => Error::InvalidOperation,
            std::io::ErrorKind::ResourceBusy => Error::FileInUse,
            std::io::ErrorKind::OutOfMemory => Error::OutOfMemory,
            _ => Error::Io,
        }
    }
}

impl From<Error> for std::io::Error {
    fn from(value: Error) -> Self {
        std::io::Error::new(
            match value {
                Error::Io => std::io::ErrorKind::Other,
                Error::FileNotOpen => std::io::ErrorKind::BrokenPipe,
                Error::InvalidBufferLength => std::io::ErrorKind::InvalidInput,
                Error::InvalidPath => std::io::ErrorKind::InvalidFilename,
                Error::InvalidOperation => std::io::ErrorKind::Unsupported,
                Error::FileNotFound => std::io::ErrorKind::NotFound,
                Error::FileAlreadyExists => std::io::ErrorKind::AlreadyExists,
                Error::FileInUse => std::io::ErrorKind::ResourceBusy,
                Error::NotADirectory => std::io::ErrorKind::NotADirectory,
                Error::NoMedia => std::io::ErrorKind::NotFound,
                Error::OutOfMemory => std::io::ErrorKind::OutOfMemory,
                Error::InternalError => std::io::ErrorKind::Other,
                Error::AccessDenied => std::io::ErrorKind::PermissionDenied,
            },
            value,
        )
    }
}

impl From<xous::Error> for Error {
    fn from(value: xous::Error) -> Self {
        match value {
            xous::Error::OutOfMemory => Self::OutOfMemory,
            xous::Error::AccessDenied => Self::AccessDenied,
            _ => Self::InternalError,
        }
    }
}
