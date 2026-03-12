// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

#[derive(Debug, thiserror::Error)]
pub enum PowerManagerError {
    #[error("OS error: {0:?}")]
    XousError(xous::Error),

    #[cfg(keyos)]
    #[error("I2C error: {0:?}")]
    I2cError(i2c::I2cError),

    #[error("Unknown internal error")]
    UnknownError,
}

impl From<xous::Error> for PowerManagerError {
    fn from(value: xous::Error) -> Self { PowerManagerError::XousError(value) }
}

#[cfg(keyos)]
impl From<i2c::I2cError> for PowerManagerError {
    fn from(value: i2c::I2cError) -> Self { PowerManagerError::I2cError(value) }
}

impl From<()> for PowerManagerError {
    fn from(_: ()) -> Self { PowerManagerError::UnknownError }
}
