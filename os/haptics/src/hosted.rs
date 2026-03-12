// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use haptics::HapticPattern;

pub struct Implementation;

impl Implementation {
    pub fn init() -> Implementation { Implementation }

    pub fn vibrate(&mut self, _pattern: HapticPattern) {}
}
