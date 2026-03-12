// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundationdevices.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{
    key_slot::KeySlot,
    keys::*,
    layout::{Layout, Row},
};

pub static LAYOUT_DECIMAL: Layout = Layout {
    rows: &[
        row(&[KeySlot::width(&key_1, 140.0), KeySlot::width(&key_2, 140.0), KeySlot::width(&key_3, 140.0)]),
        row(&[KeySlot::width(&key_4, 140.0), KeySlot::width(&key_5, 140.0), KeySlot::width(&key_6, 140.0)]),
        row(&[KeySlot::width(&key_7, 140.0), KeySlot::width(&key_8, 140.0), KeySlot::width(&key_9, 140.0)]),
        row(&[
            KeySlot::width(&key_period, 140.0),
            KeySlot::width(&key_0, 140.0),
            KeySlot::width(&key_backspace, 140.0),
        ]),
    ],
};

const fn row(key_slots: &'static [KeySlot]) -> Row { Row { gap: 15.0, key_slots } }
