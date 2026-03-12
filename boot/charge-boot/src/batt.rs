// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundationdevices.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use {
    atsama5d27::{
        pio::{Direction, Func, Pio},
        twi::Twi,
    },
    boot_common::pins::BC_CD,
    embedded_graphics::pixelcolor::Rgb888,
};

const BQ24157_ADDRESS: u8 = 0x6A;
const TUSB320RWBR_ADDRESS: u8 = 0x61;
const TUSB320RWBR_MODE_SELECT_ADDRESS: u8 = 0x0A;
const TUSB320RWBR_MODE_SELECT_MASK: u8 = 0xcf;
const TUSB320RWBR_MODE_SELECT_OFFSET: u8 = 4;
const TUSB320RWBR_MODE_SELECT_UFP: u8 = 0b01 << TUSB320RWBR_MODE_SELECT_OFFSET; // UFP (Upstream Facing Port) mode

const BQ27421_ADDR: u8 = 0x55;
const BQ27421_SOC_REG_ADDR: u8 = 0x1C;

// Fallback value of the SoC in case of an error.
// It's chosen to be higher than the low battery threshold to keep the device bootable if
// gas gauge chip gives an error
const DEFAULT_SOC: u8 = 0x1; // 100%

pub const THRESHOLD_ORANGE: u8 = 50;
pub const THRESHOLD_GREEN: u8 = 73;
pub const THRESHOLD_CHARGE_STOP: u8 = 75;

pub(crate) fn get_battery_soc() -> u16 {
    let i2c = Twi::twi0();
    let mut buf = [DEFAULT_SOC, 0x00];

    if i2c.write_read_bytes(BQ27421_ADDR, &[BQ27421_SOC_REG_ADDR], &mut buf).is_ok() {
        u16::from_le_bytes(buf)
    } else {
        DEFAULT_SOC as u16
    }
}

/// Initializes battery charger chip.
pub fn init_batt() {
    let i2c = Twi::twi0();

    let mut bc_cd = BC_CD; // BC_CD is battery charger disable pin
    bc_cd.set_func(Func::Gpio);
    bc_cd.set_direction(Direction::Output);

    for (reg, val) in keyos::batt::CHARGER_CONFIG_DUMP {
        let tx_buf = [reg, val];
        i2c.write_bytes(BQ24157_ADDRESS, &tx_buf).ok();
    }

    bc_cd.set(false);

    // Configure the USB Type-C port controller
    // to force it into current sink mode (UFP).
    let mut reg_buf = [0x00];
    if i2c.write_read_bytes(TUSB320RWBR_ADDRESS, &[TUSB320RWBR_MODE_SELECT_ADDRESS], &mut reg_buf).is_ok() {
        // Clear the "mode select" bits and set UFP mode
        reg_buf[0] &= TUSB320RWBR_MODE_SELECT_MASK;
        reg_buf[0] |= TUSB320RWBR_MODE_SELECT_UFP;

        // Write the updated register value back
        i2c.write_bytes(TUSB320RWBR_ADDRESS, &[TUSB320RWBR_MODE_SELECT_ADDRESS, reg_buf[0]]).ok();
    }
}

pub fn stop_charging() {
    let mut bc_cd = BC_CD;
    bc_cd.set(true);
}

pub fn start_charging() {
    let mut bc_cd = BC_CD;
    bc_cd.set(false);
}

pub(crate) fn is_charging() -> bool {
    let chg_stat_pin = Pio::pa26(); // BC_STAT, active low (charging)
    chg_stat_pin.set_direction(Direction::Input);
    chg_stat_pin.set_func(Func::Gpio);
    !chg_stat_pin.get()
}

pub fn batt_color(soc: u8) -> Rgb888 {
    if soc < THRESHOLD_ORANGE {
        Rgb888::new(255, 0, 0) // Red
    } else if soc < THRESHOLD_GREEN {
        Rgb888::new(255, 120, 0) // Orange
    } else {
        Rgb888::new(0, 200, 0) // Green
    }
}
