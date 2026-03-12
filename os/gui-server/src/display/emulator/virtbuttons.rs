// SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Virtual power button and home button routines.

use gui_server_api::consts::{VIRT_BUTTON_PHYS_ORIGIN_X, VIRT_BUTTON_PHYS_ORIGIN_Y};

use crate::display::emulator::consts::{VIRT_HOME_BUTTON_TOUCH_Y, VIRT_HOME_BUTTON_X};

pub(super) fn translate_virt_button_coords(x: u16, y: u16) -> (u16, u16) {
    let x = (x.saturating_sub(VIRT_HOME_BUTTON_X as u16)) + VIRT_BUTTON_PHYS_ORIGIN_X as u16;
    let y = (y.saturating_sub(VIRT_HOME_BUTTON_TOUCH_Y as u16)) + VIRT_BUTTON_PHYS_ORIGIN_Y as u16;

    (x, y)
}
