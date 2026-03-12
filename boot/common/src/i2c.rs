// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundationdevices.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::{
        get_pit,
        pins::{I2C_SCL, I2C_SDA},
        MASTER_CLOCK_SPEED,
    },
    atsama5d27::{
        pio::{Direction, Func, Pio},
        pmc::{PeripheralId, Pmc},
        twi::Twi,
    },
};

pub fn init_i2c() {
    let mut pmc = Pmc::new();
    pmc.enable_peripheral_clock(PeripheralId::Twi0);
    pmc.enable_peripheral_clock(PeripheralId::Pioc);

    // Do one clock cycle of SCL to reset all the possibly stuck slaves
    let mut scl = I2C_SCL;
    scl.set_func(Func::Gpio);
    scl.set_direction(Direction::Output);
    for _ in 0..1 {
        scl.set(false);
        for _ in 0..1000 {
            unsafe {
                core::arch::asm!("nop");
            }
        }
        scl.set(true);
    }

    let scl = I2C_SCL;
    scl.set_func(Func::E); // TWI
    let sda = I2C_SDA;
    sda.set_func(Func::E); // TWI
    let twi0 = Twi::twi0();
    twi0.init_master(MASTER_CLOCK_SPEED as usize, 100_000);

    let mut pit = get_pit();
    let mut touch_reset = Pio::pb2();
    touch_reset.set_func(Func::Gpio);
    touch_reset.set_direction(Direction::Output);
    touch_reset.set(false);
    pit.busy_wait_ms(MASTER_CLOCK_SPEED, 5);
    touch_reset.set(true);
    pit.busy_wait_ms(MASTER_CLOCK_SPEED, 50);
}
