// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use {
    atsama5d27::{
        pio::{Direction, Func, Pio},
        twi::Twi,
    },
    embedded_graphics::{pixelcolor::Rgb888, prelude::RgbColor},
};

const NUM_LEDS: usize = 4;

// Map for R,G,B channels for every LED
#[rustfmt::skip]
const CH_MAP: [[usize; 3]; NUM_LEDS] = [
    [2, 1, 0],
    [2, 1, 0],
    [0, 1, 2],
    [0, 1, 2]
];

const CONTROLLER_ADDR: u8 = 0x34;

const POWER_DISABLE_MSG: [u8; 2] = [
    0x00, // Power register
    0x00, // SSD: 0 Software shut down
];

const POWER_ENABLE_MSG: [u8; 2] = [
    0x00, // Power register
    0x01, // SSD: 1 (normal operation), PMS: 0 (8 bit PWM), OSC: 0 (16Mhz oscillator)
];

const CONFIG_REGISTERS: [[u8; 2]; 5] = [
    POWER_DISABLE_MSG,
    // Dim leds by 4x because the default max is very bright.
    [
        0x6E, // Global current control
        0x40, // Iout = Iout(max) / 4
    ],
    // Enable phase delay on half the leds to reduce power supply ripple.
    [
        0x70, // Phase delay register
        0x82, // Phase delay enable, Group 2 (out4-9) delayed by 180 degrees
    ],
    // Enable spread spectrum to reduce EMI and only use the scaling registers for dimming
    [
        0x78, // Spread spectrum register
        0x70, /* DCPWM: 0b011 (all channels), SSP: 1 (enable), RNG: 0 (+-5%), CLT: 0 (2ms cycle
               * time) */
    ],
    POWER_ENABLE_MSG,
];

pub fn init_rgb() {
    let mut led_drv_pwdn_b = Pio::pa22();
    led_drv_pwdn_b.set_direction(Direction::Output);
    led_drv_pwdn_b.set_func(Func::Gpio);
    led_drv_pwdn_b.set(true);
    let mut led_chgpmp_en = Pio::pd23();
    led_chgpmp_en.set_direction(Direction::Output);
    led_chgpmp_en.set_func(Func::Gpio);
    led_chgpmp_en.set(true);

    let i2c = Twi::twi0();
    for config in &CONFIG_REGISTERS {
        i2c.write_bytes(CONTROLLER_ADDR, config).ok();
    }
}

pub fn rgb_set_multiple(leds: core::ops::Range<usize>, color: Rgb888) {
    if leds.start >= NUM_LEDS || leds.end > NUM_LEDS || leds.end < leds.start {
        return;
    }

    let mut message = [0u8; (1 + NUM_LEDS * 3)];
    // Current scaling registers are 0x4d-0x59
    message[0] = 0x4d + (leds.start * 3) as u8;

    for (i, led) in leds.clone().enumerate() {
        message[i * 3 + 1 + CH_MAP[led][0]] = color.r();
        message[i * 3 + 1 + CH_MAP[led][1]] = color.g();
        message[i * 3 + 1 + CH_MAP[led][2]] = color.b();
    }

    let i2c = Twi::twi0();
    i2c.write_bytes(CONTROLLER_ADDR, &message[0..(1 + leds.len() * 3)]).ok();
}
