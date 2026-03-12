// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use xous::Error;
use {
    num_derive::{FromPrimitive, ToPrimitive},
    num_traits::{FromPrimitive, ToPrimitive},
};

#[derive(
    Debug, Copy, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, FromPrimitive, ToPrimitive,
)]
pub enum SpiError {
    AlreadyClaimed = 1,
    PeripheralNotClaimed,
    AccessDenied,
    MessageTooLong,
    Timeout,
    InternalError,
    DoubleSelect,
    PeripheralNotSelected,
    DmaError,
    InvalidPeripheral,
    St25r95,
    InvalidWordSize,
}

impl eh_1::spi::Error for SpiError {
    fn kind(&self) -> eh_1::spi::ErrorKind { eh_1::spi::ErrorKind::Other }
}

impl From<xous::Error> for SpiError {
    fn from(_value: Error) -> Self { SpiError::InternalError }
}

impl From<atsama5d27::spi::SpiError> for SpiError {
    fn from(value: atsama5d27::spi::SpiError) -> Self {
        match value {
            atsama5d27::spi::SpiError::Error => SpiError::InternalError,
            atsama5d27::spi::SpiError::Timeout => SpiError::Timeout,
        }
    }
}

impl server::AsScalar<1> for SpiError {
    fn as_scalar(&self) -> [u32; 1] { [self.to_u32().unwrap()] }
}

impl server::FromScalar<1> for SpiError {
    fn from_scalar([value]: [u32; 1]) -> Self { Self::from_u32(value).unwrap_or(Self::InternalError) }
}

impl From<SpiError> for usize {
    fn from(value: SpiError) -> Self { value.to_usize().unwrap() }
}

impl From<usize> for SpiError {
    fn from(value: usize) -> Self { SpiError::from_usize(value).unwrap_or(SpiError::InternalError) }
}

impl From<dma::error::DmaError> for SpiError {
    fn from(value: dma::error::DmaError) -> Self {
        log::error!("DMA Error: {value:?}");
        Self::DmaError
    }
}

impl From<SpiError> for st25r95::Error {
    fn from(_: SpiError) -> Self { st25r95::Error::Spi }
}
