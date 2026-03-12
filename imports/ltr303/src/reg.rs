// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundationdevices.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use bitfield::bitfield;

#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
pub enum Register {
    Control = 0x80,
    MeasurementRate = 0x85,
    PartId = 0x86,
    ManufacturerId = 0x87,
    DataCh1Low = 0x88,
    DataCh1High = 0x89,
    DataCh0Low = 0x8a,
    DataCh0High = 0x8b,
    Status = 0x8c,
    InterruptSettings = 0x8f,
    ThresholdUpperLow = 0x97,
    ThresholdUpperHigh = 0x98,
    ThresholdLowerLow = 0x99,
    ThresholdLowerHigh = 0x9a,
    InterruptPersist = 0x9e,
}

#[derive(Debug, Copy, Clone)]
pub enum Gain {
    Gain1x = 0b000,
    Gain2x = 0b001,
    Gain4x = 0b010,
    Gain8x = 0b011,
    GainReserved1 = 0b100,
    GainReserved2 = 0b101,
    Gain48x = 0b110,
    Gain96x = 0b111,
}

bitfield! {
    pub struct Control(u8);
    impl Debug;
    pub active, set_active: 0;
    pub reset, set_reset: 1;
    pub gain, set_gain: 4, 2;
}

bitfield! {
    pub struct InterruptSettings(u8);
    impl Debug;
    pub enable, set_enable: 1;
    pub polarity, set_polarity: 2;
}
