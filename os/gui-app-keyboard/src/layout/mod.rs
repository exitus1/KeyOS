// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundationdevices.com>
// SPDX-License-Identifier: GPL-3.0-or-later
pub mod alpha;
pub mod decimal;
pub mod numeric;

// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundationdevices.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use gui_server_api::consts::{KEYBOARD_TOP_BAR_MARGIN, SCREEN_WIDTH};

use crate::{
    key_slot::KeySlot,
    keyboard::{KEYS_TOP_MARGIN, KEY_V_MARGIN, ROW_HEIGHT},
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum LayoutType {
    #[default]
    Alphabetic,
    Numeric,
    Punctuation,
}

pub struct Row {
    pub gap: f32,
    pub key_slots: &'static [KeySlot],
}

pub struct KeyIter<'i> {
    x: f32,
    gap: f32,
    inner: core::slice::Iter<'i, KeySlot>,
}

impl<'i> Iterator for KeyIter<'i> {
    type Item = (core::ops::Range<f32>, &'i KeySlot);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(key) = self.inner.next() {
            let rx = self.x;
            self.x += key.width + self.gap;
            if !key.key.label.is_empty() || key.key.icon.is_some() {
                return Some((rx..rx + key.width, key));
            }
        }
        None
    }
}

impl Row {
    pub fn get_key(&self, id: u64) -> Option<&KeySlot> { self.key_slots.iter().find(|&s| s.key.id() == id) }

    pub fn keys_with_coords(&self) -> KeyIter<'_> {
        let gaps = self.gap * (self.key_slots.len() - 1) as f32; // count only inner gaps
        let inner_width = self.key_slots.iter().map(|s| s.width).sum::<f32>() + gaps;
        KeyIter { x: (SCREEN_WIDTH as f32 - inner_width) * 0.5, gap: self.gap, inner: self.key_slots.iter() }
    }
}

pub struct Layout {
    pub rows: &'static [Row],
}

impl Layout {
    pub fn get_key(&self, id: u64) -> Option<&KeySlot> { self.rows.iter().find_map(|row| row.get_key(id)) }

    pub fn rows_with_coords(&self) -> impl Iterator<Item = (core::ops::Range<f32>, &Row)> {
        (0..)
            .map(|row_idx| {
                let top = row_idx as f32 * ROW_HEIGHT
                    + KEYS_TOP_MARGIN
                    + KEYBOARD_TOP_BAR_MARGIN as f32
                    + KEY_V_MARGIN;
                top..(top + ROW_HEIGHT)
            })
            .zip(self.rows)
    }
}
