// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use server::{AsScalar, FromScalar};

#[derive(Debug, thiserror::Error, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub enum DmaError {
    #[error("OS error: {:?}", xous::Error::from_usize(*.0))]
    XousError(usize),

    #[error("No free channels left to allocate")]
    NoFreeChannels,

    #[error("Unknown internal error")]
    UnknownError,

    // These are errors that can't really happen through the API.
    #[error("Invalid parameter")]
    InvalidParameter,

    #[error("The source or destination address points to somewhere it shouldn't")]
    InvalidAddress,

    #[error("The length or address was not aligned to data word size")]
    InvalidAlignment,

    #[error("The lent buffer didn't contain contigous pages (use POPULATE)")]
    BufferNotContiguous,

    #[error("The transfer is already running")]
    AlreadyRunning,
}

impl From<xous::Error> for DmaError {
    fn from(value: xous::Error) -> Self { DmaError::XousError(value.to_usize()) }
}

impl From<()> for DmaError {
    fn from(_: ()) -> Self { DmaError::UnknownError }
}

impl AsScalar<2> for DmaError {
    fn as_scalar(&self) -> [u32; 2] {
        match self {
            DmaError::XousError(e) => [1, *e as u32],
            DmaError::NoFreeChannels => [2, 0],
            DmaError::UnknownError => [3, 0],
            DmaError::InvalidParameter => [4, 0],
            DmaError::InvalidAddress => [5, 0],
            DmaError::InvalidAlignment => [6, 0],
            DmaError::BufferNotContiguous => [7, 0],
            DmaError::AlreadyRunning => [8, 0],
        }
    }
}

impl FromScalar<2> for DmaError {
    fn from_scalar(value: [u32; 2]) -> Self {
        match value[0] {
            1 => DmaError::XousError(value[1] as _),
            2 => DmaError::NoFreeChannels,
            3 => DmaError::UnknownError,
            4 => DmaError::InvalidParameter,
            5 => DmaError::InvalidAddress,
            6 => DmaError::InvalidAlignment,
            7 => DmaError::BufferNotContiguous,
            8 => DmaError::AlreadyRunning,
            _ => DmaError::UnknownError,
        }
    }
}
