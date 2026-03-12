// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use {
    atsama5d27::spi::{BitsPerTransfer, ChipSelect},
    num_derive::{FromPrimitive, ToPrimitive},
    num_traits::{FromPrimitive, ToPrimitive},
    server::{AsScalar, FromScalar},
};

// There's a minimal clock speed!
// No lower that 1.79 MHz (2.0 MHz is the max per datasheet).
// Speed lower than 1.79 MHz results in tv(SO) timing (80ns) mismatch that leads to the
// loss of MSB on MOSI line
const NFC_SPI_CLOCK_SPEED_HZ: u32 = 2_000_000;

const LCD_SPI_FREQ_HZ: u32 = 10_000_000;

// nRF52805 SPIS max frequency is 8MHz
// The actual maximum data rate depends on the master's CLK to MISO and MOSI setup and hold timings.
// High bit rates may require GPIOs to be set as High Drive.
const BLE_SPI_FREQ_HZ: u32 = 4_000_000;

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
    Nfc = 0,
    Lcd = 1,
    Ble = 2,
}

impl Peripheral {
    pub fn cs(&self) -> ChipSelect {
        match self {
            Peripheral::Nfc => ChipSelect::Cs0,
            Peripheral::Lcd => ChipSelect::Cs2,
            Peripheral::Ble => ChipSelect::Cs1,
        }
    }

    pub fn bit_per_transfer(&self) -> BitsPerTransfer {
        match self {
            Peripheral::Nfc => BitsPerTransfer::Bits8,
            Peripheral::Lcd => BitsPerTransfer::Bits9,
            Peripheral::Ble => BitsPerTransfer::Bits8,
        }
    }

    pub fn bitrate(&self) -> u32 {
        match self {
            Peripheral::Nfc => NFC_SPI_CLOCK_SPEED_HZ,
            Peripheral::Lcd => LCD_SPI_FREQ_HZ,
            Peripheral::Ble => BLE_SPI_FREQ_HZ,
        }
    }

    pub fn dlybs(&self) -> u32 {
        match self {
            Peripheral::Nfc => 0,
            Peripheral::Lcd => 0,
            Peripheral::Ble => 300,
        }
    }
}

impl AsScalar<1> for Peripheral {
    fn as_scalar(&self) -> [u32; 1] { [self.to_u32().unwrap()] }
}

impl FromScalar<1> for Peripheral {
    fn from_scalar([value]: [u32; 1]) -> Self { Self::from_u32(value).unwrap_or(Peripheral::Nfc) }
}
