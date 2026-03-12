// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use embedded_hal::i2c::I2c;
use rgb_led::RgbColor;
use {
    gpio::{GpioPin, PinSettings},
    i2c::Peripheral,
};

i2c::use_api!();
gpio::use_api!();

// 2 out of 4 LEDs are disabled permanently to conserve power.
const NUM_LEDS: usize = 2;

// Map for R,G,B channels for every LED
#[rustfmt::skip]
const CH_MAP: [[usize; 3]; NUM_LEDS] = [
//  [2, 1, 0],
    [2, 1, 0],
    [0, 1, 2],
//  [0, 1, 2]
];

const CONTROLLER_PERIPHERAL: Peripheral = Peripheral::RgbLedController;
const CONTROLLER_ADDR: u8 = CONTROLLER_PERIPHERAL.i2c_addr();
const LED_SCALING_REG_BASE: u8 = 0x4d;

fn led_scaling_reg(led_index: usize) -> u8 { LED_SCALING_REG_BASE + led_index as u8 * 3 }

const POWER_DISABLE_MSG: [u8; 2] = [
    0x00, // Power register
    0x00, // SSD: 0 Software shut down
];

const POWER_ENABLE_MSG: [u8; 2] = [
    0x00, // Power register
    0x01, // SSD: 1 (normal operation), PMS: 0 (8 bit PWM), OSC: 0 (16Mhz oscillator)
];

const CONFIG_REGISTERS: [[u8; 2]; 4] = [
    POWER_DISABLE_MSG,
    // Dim leds by 8x because the default max is very bright and uses a lot of power
    [
        0x6E, // Global current control
        0x20, // Iout = Iout(max) / 8
    ],
    // Enable phase delay on half the leds to reduce power supply ripple.
    [
        0x70, // Phase delay register
        0x82, // Phase delay enable, Group 2 (out4-9) delayed by 180 degrees
    ],
    // Enable spread spectrum to reduce EMI and only use the scaling registers for dimming
    [
        0x78, // Spread spectrum register
        0x70, // DCPWM: 0b011 (all channels), SSP: 1 (enable), RNG: 0 (+-5%), CLT: 0 (2ms cycle time)
    ],
];

pub struct Implementation {
    i2c: I2cPeripheral,
    enabled: bool,
}

impl Implementation {
    pub fn init() -> Self {
        let gpio_api = GpioApi::default();
        // Claim and activate the RGB driver's "enable" pin
        // Not used for shutdown, because I2C shutdown is just as efficient.
        log::debug!("Claiming RGB feedback controller SDB signal");
        gpio_api.claim_pin(GpioPin::LedDrvPwdnB, PinSettings::OutputHigh, false).unwrap();

        // Claim haptic feedback I2C interface
        log::debug!("Claiming RGB controller I2C peripheral");
        let i2c_api = I2cApi::default();
        let mut i2c = i2c_api.claim_peripheral(Peripheral::RgbLedController).unwrap();

        // Connect to and initialize the RGB LED driver chip
        log::debug!("Initializing IS31FL3205");

        for config in &CONFIG_REGISTERS {
            i2c.write(CONTROLLER_ADDR, config).unwrap();
        }

        // Make sure leftmost and rightmost LEDs are disabled.
        i2c.write(CONTROLLER_ADDR, &[led_scaling_reg(0), 0, 0, 0]).unwrap();
        i2c.write(CONTROLLER_ADDR, &[led_scaling_reg(3), 0, 0, 0]).unwrap();

        // Claim and activate the RGB LED driver's 5v charge pump "enable" pin
        // Will be constantly enabled because it consumes negligible current when not under load
        // Only done after setting up controller with I2C to prevent powerup LED flash.
        log::debug!("Claiming RGB LED 5v charge pump ENA signal");
        gpio_api.claim_pin(GpioPin::LedChgPmpEn, PinSettings::OutputHigh, false).unwrap();

        log::debug!("RGB LED controller initialized");

        Self { i2c, enabled: false }
    }

    pub fn set_all(&mut self, color: RgbColor) {
        log::trace!("Setting all LEDs to #{:02x}{:02x}{:02x}", color.r, color.g, color.b,);
        if let Err(e) = self.set_multiple(0..NUM_LEDS, color) {
            log::error!("Error setting all leds: {e:?}");
            return;
        }

        // set_all(BLACK) was called: disable the analog circuitry on the controller
        if color == RgbColor::BLACK && self.enabled {
            if let Err(e) = self.i2c.write(CONTROLLER_ADDR, &POWER_DISABLE_MSG) {
                log::error!("Error disabling controller: {e:?}");
                return;
            }
            self.enabled = false;
        }
    }

    pub fn set(&mut self, led: u8, color: RgbColor) {
        log::trace!("Setting LED #{} to #{:02x}{:02x}{:02x}", led, color.r, color.g, color.b,);
        if led as usize >= NUM_LEDS {
            log::error!("Led index {led:?} is out of range");
            return;
        }
        if let Err(e) = self.set_multiple((led as usize)..(led as usize + 1), color) {
            log::error!("Error setting led {led}: {e:?}");
            return;
        }
    }
}

impl Implementation {
    fn set_multiple(&mut self, leds: std::ops::Range<usize>, color: RgbColor) -> Result<(), i2c::I2cError> {
        let mut message = [0u8; (1 + NUM_LEDS * 3)];

        // leds.start is offset by 1, because we want to start at the 2nd physical LED,
        // the leftmost is permanently disabled.
        message[0] = led_scaling_reg(leds.start + 1);

        for (i, led) in leds.clone().enumerate() {
            message[i * 3 + 1 + CH_MAP[led][0]] = color.r;
            message[i * 3 + 1 + CH_MAP[led][1]] = color.g;
            message[i * 3 + 1 + CH_MAP[led][2]] = color.b;
        }

        self.i2c.write(CONTROLLER_ADDR, &message[0..(1 + leds.len() * 3)])?;

        // Enable the controller power if it is not enabled already.
        if !self.enabled {
            self.i2c.write(CONTROLLER_ADDR, &POWER_ENABLE_MSG)?;
            self.enabled = true;
        }

        Ok(())
    }
}
