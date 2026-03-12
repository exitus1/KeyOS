// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later
#![cfg(keyos)]
pub mod api;
pub mod error;
pub mod messages;

use server::{AsScalar, FromScalar};

mod implementation;

#[derive(Debug, server::Message, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[event(AmbientLightMeasurement)]
struct AmbientLightSubscribe;

#[derive(Debug, Clone, Copy)]
pub struct AmbientLightMeasurement {
    /// The visible ambient light level in logical linear units
    measurement: u16,
}

impl FromScalar<1> for AmbientLightMeasurement {
    fn from_scalar(value: [u32; 1]) -> Self { Self { measurement: value[0] as u16 } }
}

impl AsScalar<1> for AmbientLightMeasurement {
    fn as_scalar(&self) -> [u32; 1] { [self.measurement as u32] }
}

pub fn listen() { server::listen(implementation::AmbientLightServer::new().unwrap()) }
