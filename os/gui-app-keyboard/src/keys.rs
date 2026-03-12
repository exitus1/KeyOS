// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundationdevices.com>
// SPDX-License-Identifier: GPL-3.0-or-later

#![allow(non_upper_case_globals)]

// All key definitions

use std::hash::{DefaultHasher, Hash as _, Hasher as _};

use crate::{
    colors::KeyStyle,
    drawing::Icon,
    keyboard::{assets::*, KEY_FONT_SCALE},
    layout::LayoutType,
};

// Lowercase
pub const key_a: KeyDef = KeyDef::new_overlay("a", "aàáâäæãåā");
pub const key_b: KeyDef = KeyDef::new_overlay("b", "b");
pub const key_c: KeyDef = KeyDef::new_overlay("c", "cçćčċ");
pub const key_d: KeyDef = KeyDef::new_overlay("d", "dďð");
pub const key_e: KeyDef = KeyDef::new_overlay("e", "eèéêëẽēėę");
pub const key_f: KeyDef = KeyDef::new_overlay("f", "f");
pub const key_g: KeyDef = KeyDef::new_overlay("g", "gğġ");
pub const key_h: KeyDef = KeyDef::new_overlay("h", "hħ");
pub const key_i: KeyDef = KeyDef::new_overlay("i", "ìíîïĩīıiį");
pub const key_j: KeyDef = KeyDef::new_overlay("j", "j");
pub const key_k: KeyDef = KeyDef::new_overlay("k", "kķ");
pub const key_l: KeyDef = KeyDef::new_overlay("l", "łļlľ");
pub const key_m: KeyDef = KeyDef::new_overlay("m", "m");
pub const key_n: KeyDef = KeyDef::new_overlay("n", "ñnńņň");
pub const key_o: KeyDef = KeyDef::new_overlay("o", "òóôöœøõoō");
pub const key_p: KeyDef = KeyDef::new_overlay("p", "p");
pub const key_q: KeyDef = KeyDef::new_overlay("q", "q");
pub const key_r: KeyDef = KeyDef::new_overlay("r", "rř");
pub const key_s: KeyDef = KeyDef::new_overlay("s", "sßşșśš");
pub const key_t: KeyDef = KeyDef::new_overlay("t", "tțťþ");
pub const key_u: KeyDef = KeyDef::new_overlay("u", "ùúûüuũūűů");
pub const key_v: KeyDef = KeyDef::new_overlay("v", "v");
pub const key_w: KeyDef = KeyDef::new_overlay("w", "wŵ");
pub const key_x: KeyDef = KeyDef::new_overlay("x", "x");
pub const key_y: KeyDef = KeyDef::new_overlay("y", "yýŷÿ");
pub const key_z: KeyDef = KeyDef::new_overlay("z", "zźžż");

// Uppercase
pub const key_A: KeyDef = KeyDef::new_overlay("A", "AÀÁÂÄÆÃÅĀ");
pub const key_B: KeyDef = KeyDef::new_overlay("B", "B");
pub const key_C: KeyDef = KeyDef::new_overlay("C", "CÇĆČĊ");
pub const key_D: KeyDef = KeyDef::new_overlay("D", "DĎÐ");
pub const key_E: KeyDef = KeyDef::new_overlay("E", "EÈÉÊËẼĒĖĘ");
pub const key_F: KeyDef = KeyDef::new_overlay("F", "F");
pub const key_G: KeyDef = KeyDef::new_overlay("G", "GĞĠ");
pub const key_H: KeyDef = KeyDef::new_overlay("H", "HĦ");
pub const key_I: KeyDef = KeyDef::new_overlay("I", "ÌÍÎÏĨĪİIĮ");
pub const key_J: KeyDef = KeyDef::new_overlay("J", "J");
pub const key_K: KeyDef = KeyDef::new_overlay("K", "KĶ");
pub const key_L: KeyDef = KeyDef::new_overlay("L", "ŁĻLĽ");
pub const key_M: KeyDef = KeyDef::new_overlay("M", "M");
pub const key_N: KeyDef = KeyDef::new_overlay("N", "ÑNŃŅŇ");
pub const key_O: KeyDef = KeyDef::new_overlay("O", "ÒÓÔÖŒØÕOŌ");
pub const key_P: KeyDef = KeyDef::new_overlay("P", "P");
pub const key_Q: KeyDef = KeyDef::new_overlay("Q", "Q");
pub const key_R: KeyDef = KeyDef::new_overlay("R", "RŘ");
pub const key_S: KeyDef = KeyDef::new_overlay("S", "SẞŚŠŞȘ");
pub const key_T: KeyDef = KeyDef::new_overlay("T", "TȚŤÞ");
pub const key_U: KeyDef = KeyDef::new_overlay("U", "ÙÚÛÜUŨŪŰŮ");
pub const key_V: KeyDef = KeyDef::new_overlay("V", "V");
pub const key_W: KeyDef = KeyDef::new_overlay("W", "WŴ");
pub const key_X: KeyDef = KeyDef::new_overlay("X", "X");
pub const key_Y: KeyDef = KeyDef::new_overlay("Y", "YÝŶŸ");
pub const key_Z: KeyDef = KeyDef::new_overlay("Z", "ZŹŽŻ");

// Numbers
pub const key_0: KeyDef = KeyDef::new_overlay("0", "");
pub const key_1: KeyDef = KeyDef::new_overlay("1", "");
pub const key_2: KeyDef = KeyDef::new_overlay("2", "");
pub const key_3: KeyDef = KeyDef::new_overlay("3", "");
pub const key_4: KeyDef = KeyDef::new_overlay("4", "");
pub const key_5: KeyDef = KeyDef::new_overlay("5", "");
pub const key_6: KeyDef = KeyDef::new_overlay("6", "");
pub const key_7: KeyDef = KeyDef::new_overlay("7", "");
pub const key_8: KeyDef = KeyDef::new_overlay("8", "");
pub const key_9: KeyDef = KeyDef::new_overlay("9", "");

// Punctuation
pub const key_space: KeyDef = KeyDef::new_action(" ", KeyAction::Space, KeyStyle::Normal);
pub const key_comma: KeyDef = KeyDef::new_overlay(",", "");
pub const key_period: KeyDef = KeyDef::new_overlay(".", "");
pub const key_question: KeyDef = KeyDef::new_overlay("?", "");
pub const key_exclamation: KeyDef = KeyDef::new_overlay("!", "");
pub const key_semicolon: KeyDef = KeyDef::new_overlay(";", "");
pub const key_colon: KeyDef = KeyDef::new_overlay(":", "");
pub const key_apostrophe: KeyDef = KeyDef::new_overlay("'", "");
pub const key_quote: KeyDef = KeyDef::new_overlay("\"", "");
pub const key_hyphen: KeyDef = KeyDef::new_overlay("-", "");
pub const key_underscore: KeyDef = KeyDef::new_overlay("_", "");
pub const key_slash: KeyDef = KeyDef::new_overlay("/", "");
pub const key_backslash: KeyDef = KeyDef::new_overlay("\\", "");
pub const key_pipe: KeyDef = KeyDef::new_overlay("|", "");
pub const key_tilde: KeyDef = KeyDef::new_overlay("~", "");
pub const key_equals: KeyDef = KeyDef::new_overlay("=", "");
pub const key_plus: KeyDef = KeyDef::new_overlay("+", "");
pub const key_asterisk: KeyDef = KeyDef::new_overlay("*", "");
pub const key_ampersand: KeyDef = KeyDef::new_overlay("&", "");
pub const key_caret: KeyDef = KeyDef::new_overlay("^", "");
pub const key_percent: KeyDef = KeyDef::new_overlay("%", "");
pub const key_dollar: KeyDef = KeyDef::new_overlay("$", "");
pub const key_hashtag: KeyDef = KeyDef::new_overlay("#", "");
pub const key_at: KeyDef = KeyDef::new_overlay("@", "");
pub const key_paren_left: KeyDef = KeyDef::new_overlay("(", "");
pub const key_paren_right: KeyDef = KeyDef::new_overlay(")", "");
pub const key_bracket_left: KeyDef = KeyDef::new_overlay("[", "");
pub const key_bracket_right: KeyDef = KeyDef::new_overlay("]", "");
pub const key_brace_left: KeyDef = KeyDef::new_overlay("{", "");
pub const key_brace_right: KeyDef = KeyDef::new_overlay("}", "");
pub const key_less_than: KeyDef = KeyDef::new_overlay("<", "");
pub const key_greater_than: KeyDef = KeyDef::new_overlay(">", "");
pub const key_euro: KeyDef = KeyDef::new_overlay("€", "");
pub const key_pound_sterling: KeyDef = KeyDef::new_overlay("£", "");
pub const key_yen: KeyDef = KeyDef::new_overlay("¥", "");
pub const key_bullet: KeyDef = KeyDef::new_overlay("•", "");

// Special keys
pub const key_done: KeyDef =
    KeyDef::new_action("Done", KeyAction::Return, KeyStyle::Cta).with_font_scale(32.0);
pub const key_backspace: KeyDef = KeyDef::new_icon(&BACKSPACE, KeyAction::Backspace, KeyStyle::Accent);

pub const key_shift_lowercase: KeyDef = KeyDef::new_icon(&UNSHIFTED, KeyAction::Shift, KeyStyle::Accent);
pub const key_shift_uppercase: KeyDef = KeyDef::new_icon(&SHIFTED, KeyAction::Shift, KeyStyle::Accent);
pub const key_shift_caps_lock: KeyDef = KeyDef::new_icon(&CAPS, KeyAction::Shift, KeyStyle::Accent);

// Layer change keys
pub const key_to_numeric: KeyDef = KeyDef::new_layer_label("123", LayoutType::Numeric);
pub const key_to_punctuation: KeyDef = KeyDef::new_layer_label("#+=", LayoutType::Punctuation);
pub const key_to_alpha: KeyDef = KeyDef::new_layer_label("abc", LayoutType::Alphabetic);

// Empty key
pub const key_empty: KeyDef = KeyDef::empty();

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    #[default]
    None,
    Insert,
    Return,
    Backspace,
    Space,
    ChangeLayer(LayoutType),
    Shift,
}

// The Key defines the behavior
#[derive(Clone, Debug, PartialEq)]
pub struct KeyDef {
    pub label: &'static str,
    pub overlay: &'static str, // A list of overlay characters
    pub icon: Option<&'static Icon>,
    pub style: KeyStyle,
    pub on_released: KeyAction,
    pub font_scale: f32,
}

impl KeyDef {
    pub const fn empty() -> Self {
        Self {
            label: "",
            overlay: "",
            icon: None,
            style: KeyStyle::Normal,
            on_released: KeyAction::None,
            font_scale: KEY_FONT_SCALE,
        }
    }

    pub const fn new(label: &'static str) -> Self {
        Self { label, style: KeyStyle::Normal, on_released: KeyAction::Insert, ..Self::empty() }
    }

    pub const fn new_overlay(label: &'static str, overlay: &'static str) -> Self {
        Self { overlay: if overlay.is_empty() { label } else { overlay }, ..Self::new(label) }
    }

    pub const fn new_action(label: &'static str, action: KeyAction, color: KeyStyle) -> Self {
        Self { label, style: color, on_released: action, ..Self::empty() }
    }

    pub const fn new_icon(icon: &'static Icon, action: KeyAction, color: KeyStyle) -> Self {
        Self { icon: Some(icon), style: color, on_released: action, ..Self::empty() }
    }

    pub const fn new_layer_label(label: &'static str, layer: LayoutType) -> Self {
        Self { label, style: KeyStyle::Accent, on_released: KeyAction::ChangeLayer(layer), ..Self::empty() }
    }

    pub const fn with_font_scale(self, font_scale: f32) -> Self { Self { font_scale, ..self } }

    pub fn id(&self) -> u64 {
        let mut s = DefaultHasher::new();
        if let Some(icon) = self.icon {
            icon.hash(&mut s);
        }
        self.label.hash(&mut s);
        s.finish()
    }
}
