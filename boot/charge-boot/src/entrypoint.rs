// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundationdevices.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use boot_common::{
    display::{init_display, init_lcdc},
    i2c::init_i2c,
    load_os_image_file,
    pins::init_pins,
};

use crate::{batt::init_batt, main_screen::show_main_screen, rgb::init_rgb};

pub fn entrypoint() {
    init_pins();
    init_i2c();
    init_batt();
    init_rgb();

    init_lcdc(|_| {});
    init_display();
    // XXX: Dummy load to initialize eMMC for the erase later
    unsafe {
        load_os_image_file(b"dummy".as_ptr(), true);
    }

    show_main_screen();
}

extern "C" {
    static mut _etext: u32;
}
