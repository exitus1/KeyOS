// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("OS error: {0:?}")]
    XousError(xous::Error),

    #[cfg(keyos)]
    #[error("GPIO error: {0:?}")]
    GpioApiError(gpio::GpioApiError),

    #[cfg(keyos)]
    #[error("I2C error: {0:?}")]
    I2cError(i2c::I2cError),

    #[cfg(keyos)]
    #[error("ltr303 error: {0:?}")]
    Is31fl32xxError(ltr303::Ltr303Error<i2c::I2cError>),

    #[error("Unknown internal error")]
    UnknownError,
}

impl From<xous::Error> for Error {
    fn from(value: xous::Error) -> Self { Error::XousError(value) }
}

#[cfg(keyos)]
impl From<i2c::I2cError> for Error {
    fn from(value: i2c::I2cError) -> Self { Error::I2cError(value) }
}

#[cfg(keyos)]
impl From<gpio::GpioApiError> for Error {
    fn from(value: gpio::GpioApiError) -> Self { Error::GpioApiError(value) }
}

#[cfg(keyos)]
impl From<ltr303::Ltr303Error<i2c::I2cError>> for Error {
    fn from(value: ltr303::Ltr303Error<i2c::I2cError>) -> Self { Error::Is31fl32xxError(value) }
}

impl From<()> for Error {
    fn from(_: ()) -> Self { Error::UnknownError }
}
