// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

#[derive(Debug, thiserror::Error, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub enum MassStorageEmulationError {
    #[error("OS error: {:?}", xous::Error::from_usize(*.0))]
    XousError(usize),
    #[error("Mass storage error: {0:?}")]
    MassStorageError(mass_storage::MassStorageError),
    #[error("Usb error: {0:?}")]
    UsbError(usb::error::UsbError),
    #[error("Other error")]
    Other,
}

impl From<xous::Error> for MassStorageEmulationError {
    fn from(value: xous::Error) -> Self { Self::XousError(value.to_usize()) }
}

impl From<mass_storage::MassStorageError> for MassStorageEmulationError {
    fn from(value: mass_storage::MassStorageError) -> Self { Self::MassStorageError(value) }
}

impl From<usb::error::UsbError> for MassStorageEmulationError {
    fn from(value: usb::error::UsbError) -> Self { Self::UsbError(value) }
}
