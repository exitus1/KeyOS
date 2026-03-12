// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use rgb_led::RgbColor;

pub struct Implementation;

impl Implementation {
    pub fn init() -> Implementation { Implementation }

    pub fn set_all(&mut self, _color: RgbColor) {}

    pub fn set(&mut self, _led: u8, _color: RgbColor) {}
}
