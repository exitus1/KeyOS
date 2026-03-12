// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundationdevices.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use atsama5d27::{
    pio::{Direction, Func, Pio, PioA, PioC, PioD},
    pmc::{PeripheralId, Pmc},
};

pub const I2C_SDA: Pio<PioC, 27> = Pio::pc27();
pub const I2C_SCL: Pio<PioC, 28> = Pio::pc28();

pub const BC_CD: Pio<PioD, 20> = Pio::pd20();

pub const PWR_BTN: Pio<PioA, 25> = Pio::pa25();

pub fn init_pins() {
    let mut pmc = Pmc::new();
    pmc.enable_peripheral_clock(PeripheralId::Pioa);
    pmc.enable_peripheral_clock(PeripheralId::Piob);
    pmc.enable_peripheral_clock(PeripheralId::Pioc);
    pmc.enable_peripheral_clock(PeripheralId::Piod);

    let power_btn = PWR_BTN;
    power_btn.set_direction(Direction::Input);
    power_btn.set_func(Func::Gpio);
}
