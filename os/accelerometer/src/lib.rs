// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later
#![cfg(keyos)]

pub mod api;
pub mod error;
pub mod messages;

use server::{AsScalar, FromScalar};

mod implementation;

#[derive(Debug, Clone, Copy)]
/// Acceleration measurements. 1 bit is 1/1024 g.
/// Remember that the gravity vector is opposite to the accelerometer reading.
pub struct AccelerometerMeasurement {
    // Positive is UP, Negative is DOWN
    x: i16,
    // Positive is LEFT, negative is RIGHT
    y: i16,
    // Positive is towards the user, Negative is away from the user
    z: i16,
}

impl FromScalar<3> for AccelerometerMeasurement {
    fn from_scalar(value: [u32; 3]) -> Self { Self { x: value[0] as _, y: value[1] as _, z: value[2] as _ } }
}

impl AsScalar<3> for AccelerometerMeasurement {
    fn as_scalar(&self) -> [u32; 3] { [self.x as _, self.y as _, self.z as _] }
}

pub fn listen() { server::listen(implementation::AccelerometerServer::new().unwrap()) }
