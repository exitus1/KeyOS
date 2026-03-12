// SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use num_derive::{FromPrimitive, ToPrimitive};

#[derive(
    Debug,
    Copy,
    Clone,
    FromPrimitive,
    ToPrimitive,
    Eq,
    Hash,
    PartialEq,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub enum Peripheral {
    Accelerometer = 0,
    Camera,
    AmbientLightSensor,
    RgbLedController,
    TouchController,
    Eeprom,
    FuelGauge,
    HapticDriver,
    UsbPortController,
    BatteryCharger,
}

impl Peripheral {
    pub const fn i2c_addr(&self) -> u8 {
        match self {
            Peripheral::Accelerometer => 0x15,
            Peripheral::Camera => 0x21,
            Peripheral::AmbientLightSensor => 0x29,
            Peripheral::RgbLedController => 0x34,
            Peripheral::TouchController => 0x38,
            Peripheral::Eeprom => 0x50,
            Peripheral::FuelGauge => 0x55,
            Peripheral::HapticDriver => 0x5A,
            Peripheral::UsbPortController => 0x61,
            Peripheral::BatteryCharger => 0x6A,
        }
    }
}
