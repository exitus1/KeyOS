// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundationdevices.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use embedded_graphics::{
    image::ImageRaw,
    mono_font::{
        mapping::{StrGlyphMapping, ASCII},
        DecorationDimensions, MonoFont,
    },
    prelude::*,
};

pub const SOURCE_CODE_PRO_FONT: MonoFont = MonoFont {
    image: ImageRaw::new(include_bytes!("../assets/fonts/source_code_pro_14x24.raw"), 16 * 14),
    glyph_mapping: &ASCII,
    character_size: Size::new(14, 24),
    character_spacing: 0,
    baseline: 0,
    underline: DecorationDimensions::default_underline(24),
    strikethrough: DecorationDimensions::default_strikethrough(13),
};

pub const ICON_FONT: MonoFont = MonoFont {
    image: ImageRaw::new(include_bytes!("../assets/fonts/icon_font.raw"), 24 * 5),
    glyph_mapping: &StrGlyphMapping::new("ipSPL", 0),
    character_size: Size::new(24, 24),
    character_spacing: 0,
    baseline: 0,
    underline: DecorationDimensions::default_underline(24),
    strikethrough: DecorationDimensions::default_strikethrough(13),
};
