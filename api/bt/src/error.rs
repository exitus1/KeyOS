// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use server::{AsScalar, FromScalar};

#[derive(Debug, Copy, Clone, thiserror::Error, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub enum BluetoothError {
    #[error("OS error: {:?}", xous::Error::from_usize(*.0))]
    XousError(usize),

    #[error("Message is too long")]
    MessageTooLong,

    #[error("BLE send buffer overflow, packet rejected")]
    BlePacketRejected,

    #[error("Unknown internal error")]
    UnknownError,

    #[cfg(keyos)]
    #[error("GPIO error: {0:?}")]
    GpioApiError(gpio::GpioApiError),

    #[cfg(keyos)]
    #[error("Communication protocol error")]
    SpiProtocolError,

    #[cfg(keyos)]
    #[error("Communication protocol error: timeout")]
    SpiTimeout,

    #[cfg(keyos)]
    #[error("SPI error")]
    SpiError(spi::SpiError),

    #[error("BLE is in an inappropriate state for the command")]
    InvalidState,

    #[cfg(keyos)]
    #[error("Unable to verify BLE controller's firmware")]
    UnverifiedFirmware,

    #[cfg(keyos)]
    #[error("Crypto error")]
    Crypto,

    #[cfg(keyos)]
    #[error("Random Generator error")]
    Random,
}

impl From<xous::Error> for BluetoothError {
    fn from(value: xous::Error) -> Self { BluetoothError::XousError(value.to_usize()) }
}

impl From<()> for BluetoothError {
    fn from(_: ()) -> Self { BluetoothError::UnknownError }
}

impl From<usize> for BluetoothError {
    fn from(value: usize) -> Self { Self::from_scalar([value as u32, 0]) }
}

impl From<BluetoothError> for usize {
    fn from(value: BluetoothError) -> usize { <BluetoothError as AsScalar<2>>::as_scalar(&value)[0] as usize }
}

#[cfg(keyos)]
impl From<postcard::Error> for BluetoothError {
    fn from(value: postcard::Error) -> Self {
        log::debug!("Postcard error: {value:?}");
        BluetoothError::SpiProtocolError
    }
}

#[cfg(keyos)]
impl From<spi::SpiError> for BluetoothError {
    fn from(value: spi::SpiError) -> Self { BluetoothError::SpiError(value) }
}

#[cfg(keyos)]
impl From<gpio::GpioApiError> for BluetoothError {
    fn from(e: gpio::GpioApiError) -> Self { BluetoothError::GpioApiError(e) }
}

impl AsScalar<2> for BluetoothError {
    fn as_scalar(&self) -> [u32; 2] {
        match self {
            BluetoothError::XousError(e) => [0, *e as u32],
            BluetoothError::MessageTooLong => [1, 0],
            BluetoothError::BlePacketRejected => [2, 0],
            BluetoothError::UnknownError => [3, 0],
            #[cfg(keyos)]
            BluetoothError::GpioApiError(e) => [4, AsScalar::<1>::as_scalar(e)[0]],
            #[cfg(keyos)]
            BluetoothError::SpiProtocolError => [5, 0],
            #[cfg(keyos)]
            BluetoothError::SpiTimeout => [6, 0],
            #[cfg(keyos)]
            BluetoothError::SpiError(e) => [7, AsScalar::<1>::as_scalar(e)[0]],
            BluetoothError::InvalidState => [8, 0],
            #[cfg(keyos)]
            BluetoothError::UnverifiedFirmware => [9, 0],
            #[cfg(keyos)]
            BluetoothError::Crypto => [10, 0],
            #[cfg(keyos)]
            BluetoothError::Random => [11, 0],
        }
    }
}

impl FromScalar<2> for BluetoothError {
    fn from_scalar(value: [u32; 2]) -> Self {
        match value[0] {
            0 => BluetoothError::XousError(value[1] as usize),
            1 => BluetoothError::MessageTooLong,
            2 => BluetoothError::BlePacketRejected,
            3 => BluetoothError::UnknownError,
            #[cfg(keyos)]
            4 => BluetoothError::GpioApiError(gpio::GpioApiError::from_scalar([value[1]])),
            #[cfg(keyos)]
            5 => BluetoothError::SpiProtocolError,
            #[cfg(keyos)]
            6 => BluetoothError::SpiTimeout,
            #[cfg(keyos)]
            7 => BluetoothError::SpiError(spi::SpiError::from_scalar([value[1]])),
            8 => BluetoothError::InvalidState,
            #[cfg(keyos)]
            9 => BluetoothError::UnverifiedFirmware,
            #[cfg(keyos)]
            10 => BluetoothError::Crypto,
            #[cfg(keyos)]
            11 => BluetoothError::Random,
            _ => BluetoothError::UnknownError,
        }
    }
}
