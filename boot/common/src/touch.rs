// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundationdevices.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use {
    atsama5d27::twi::Twi,
    ft3269::{Ft3269, TouchKind},
};

/// This offset is subtracted from the Y coordinate to make the touches more
/// user-friendly. The value is derived empirically from the users' feedback.
const TOUCH_Y_OFFSET: u16 = 30;

pub fn get_last_touch(ctp: &mut Ft3269<Twi>) -> Option<(u16, u16, TouchKind)> {
    let xh = ctp.read_reg_u8_by_addr(0x03).unwrap_or(0xff);
    let xl = ctp.read_reg_u8_by_addr(0x04).unwrap_or(0xff);
    let yh = ctp.read_reg_u8_by_addr(0x05).unwrap_or(0xff);
    let yl = ctp.read_reg_u8_by_addr(0x06).unwrap_or(0xff);

    let x = u16::from_be_bytes([xh & 0b1111, xl]);
    let kind = (xh >> 6) & 0b11;
    let y = u16::from_be_bytes([yh & 0b1111, yl]);
    let y = y.saturating_sub(TOUCH_Y_OFFSET);

    if kind != 0b11 {
        return Some((x, y, TouchKind::from(kind)));
    }

    None
}
