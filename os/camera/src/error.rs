// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::PoisonError;

#[derive(Debug, thiserror::Error)]
pub enum CameraError {
    #[error("Xous error: {0:?}")]
    XousError(xous::Error),

    #[cfg(keyos)]
    #[error("Gpio API error: {0:?}")]
    GpioApiError(gpio::GpioApiError),

    #[cfg(keyos)]
    #[error("i2c error: {0:?}")]
    I2cError(i2c::I2cError),

    #[cfg(keyos)]
    #[error("ovm7690 error: {0:?}")]
    Ovm7690Error(ovm7690_rs::Ovm7690Error<i2c::I2cError>),

    #[error("Unknown internal error")]
    InternalError,

    #[error("gui-server error: (0:?)")]
    GuiServerError(#[from] gui_server_api::GuiServerError),

    #[cfg(not(keyos))]
    #[error("nokhwa error: (0:?)")]
    NokhwaError(#[from] nokhwa::NokhwaError),
}

impl From<xous::Error> for CameraError {
    fn from(value: xous::Error) -> Self { CameraError::XousError(value) }
}

#[cfg(keyos)]
impl From<i2c::I2cError> for CameraError {
    fn from(value: i2c::I2cError) -> Self { CameraError::I2cError(value) }
}

#[cfg(keyos)]
impl From<gpio::GpioApiError> for CameraError {
    fn from(value: gpio::GpioApiError) -> Self { CameraError::GpioApiError(value) }
}

#[cfg(keyos)]
impl From<ovm7690_rs::Ovm7690Error<i2c::I2cError>> for CameraError {
    fn from(value: ovm7690_rs::Ovm7690Error<i2c::I2cError>) -> Self { CameraError::Ovm7690Error(value) }
}

impl From<()> for CameraError {
    fn from(_: ()) -> Self { CameraError::InternalError }
}

impl From<PoisonError<std::sync::MutexGuard<'_, crate::camera::CameraServer>>> for CameraError {
    fn from(_value: PoisonError<std::sync::MutexGuard<'_, crate::camera::CameraServer>>) -> Self {
        Self::InternalError
    }
}
