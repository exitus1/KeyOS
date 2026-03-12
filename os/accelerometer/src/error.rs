// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("OS error: {0:?}")]
    XousError(xous::Error),

    #[error("I2C error: {0:?}")]
    I2cError(i2c::I2cError),

    #[error("Unknown internal error")]
    UnknownError,
}

impl From<xous::Error> for Error {
    fn from(value: xous::Error) -> Self { Error::XousError(value) }
}

impl From<i2c::I2cError> for Error {
    fn from(value: i2c::I2cError) -> Self { Error::I2cError(value) }
}

impl From<()> for Error {
    fn from(_: ()) -> Self { Error::UnknownError }
}
