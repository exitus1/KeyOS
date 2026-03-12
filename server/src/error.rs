// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later
use rkyv::rancor;

#[derive(Debug)]
pub enum Error {
    /// syscall error
    Xous(xous::Error),
    /// ipc error
    Ipc(rancor::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Xous(e) => write!(f, "{e:?}"),
            Self::Ipc(e) => write!(f, "{e}"),
        }
    }
}

impl Error {
    #[inline]
    pub fn into_xous(self) -> xous::Error {
        match self {
            Error::Xous(e) => e,
            Error::Ipc(_) => xous::Error::InternalError,
        }
    }

    #[inline]
    pub fn into_rancor(self) -> rancor::Error {
        match self {
            Error::Xous(e) => rancor::Source::new(XousError(e)),
            Error::Ipc(e) => e,
        }
    }
}

impl From<xous::Error> for Error {
    #[inline]
    fn from(e: xous::Error) -> Self { Self::Xous(e) }
}

impl From<rancor::Error> for Error {
    #[inline]
    fn from(e: rancor::Error) -> Self { Self::Ipc(e) }
}

impl PartialEq<xous::Error> for Error {
    fn eq(&self, other: &xous::Error) -> bool { matches!(self, Self::Xous(e) if e == other) }
}

#[repr(transparent)]
struct XousError(xous::Error);

impl std::fmt::Debug for XousError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{:?}", self.0) }
}

impl std::fmt::Display for XousError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{:?}", self.0) }
}

impl std::error::Error for XousError {}

const _: () = {
    #[cfg(target_pointer_width = "32")]
    const PTR_SIZE: usize = 4;
    #[cfg(target_pointer_width = "64")]
    const PTR_SIZE: usize = 8;

    #[cfg(debug_assertions)]
    const ERROR_SIZE: usize = PTR_SIZE * 2;
    #[cfg(not(debug_assertions))]
    const ERROR_SIZE: usize = PTR_SIZE;

    assert!(std::mem::size_of::<xous::Error>() == PTR_SIZE);
    assert!(std::mem::size_of::<whence::Error<xous::Error>>() == PTR_SIZE * 2);

    assert!(std::mem::size_of::<Error>() == ERROR_SIZE);
    assert!(std::mem::size_of::<whence::Error<Error>>() == ERROR_SIZE + PTR_SIZE);
};
