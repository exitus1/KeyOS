// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundationdevices.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{
    key_slot::KeySlot,
    keys::*,
    layout::{Layout, Row},
};

pub static LAYOUT_ALPHA_LOWER: Layout = Layout {
    rows: &[
        row(&[
            KeySlot::new(&key_q),
            KeySlot::new(&key_w),
            KeySlot::new(&key_e),
            KeySlot::new(&key_r),
            KeySlot::new(&key_t),
            KeySlot::new(&key_y),
            KeySlot::new(&key_u),
            KeySlot::new(&key_i),
            KeySlot::new(&key_o),
            KeySlot::new(&key_p),
        ]),
        row(&[
            KeySlot::new(&key_a),
            KeySlot::new(&key_s),
            KeySlot::new(&key_d),
            KeySlot::new(&key_f),
            KeySlot::new(&key_g),
            KeySlot::new(&key_h),
            KeySlot::new(&key_j),
            KeySlot::new(&key_k),
            KeySlot::new(&key_l),
        ]),
        row(&[
            KeySlot::width(&key_shift_lowercase, 50.0),
            KeySlot::width(&key_empty, 7.0),
            KeySlot::new(&key_z),
            KeySlot::new(&key_x),
            KeySlot::new(&key_c),
            KeySlot::new(&key_v),
            KeySlot::new(&key_b),
            KeySlot::new(&key_n),
            KeySlot::new(&key_m),
            KeySlot::width(&key_empty, 7.0),
            KeySlot::width(&key_backspace, 50.0),
        ]),
        row(&bottom_row(&key_to_numeric)),
    ],
};

pub static LAYOUT_ALPHA_UPPER: Layout = Layout {
    rows: &[
        row(&[
            KeySlot::new(&key_Q),
            KeySlot::new(&key_W),
            KeySlot::new(&key_E),
            KeySlot::new(&key_R),
            KeySlot::new(&key_T),
            KeySlot::new(&key_Y),
            KeySlot::new(&key_U),
            KeySlot::new(&key_I),
            KeySlot::new(&key_O),
            KeySlot::new(&key_P),
        ]),
        row(&[
            KeySlot::new(&key_A),
            KeySlot::new(&key_S),
            KeySlot::new(&key_D),
            KeySlot::new(&key_F),
            KeySlot::new(&key_G),
            KeySlot::new(&key_H),
            KeySlot::new(&key_J),
            KeySlot::new(&key_K),
            KeySlot::new(&key_L),
        ]),
        row(&[
            KeySlot::width(&key_shift_uppercase, 50.0),
            KeySlot::width(&key_empty, 7.0),
            KeySlot::new(&key_Z),
            KeySlot::new(&key_X),
            KeySlot::new(&key_C),
            KeySlot::new(&key_V),
            KeySlot::new(&key_B),
            KeySlot::new(&key_N),
            KeySlot::new(&key_M),
            KeySlot::width(&key_empty, 7.0),
            KeySlot::width(&key_backspace, 50.0),
        ]),
        row(&bottom_row(&key_to_numeric)),
    ],
};

pub static LAYOUT_ALPHA_UPPER_CAPS: Layout = Layout {
    rows: &[
        row(&[
            KeySlot::new(&key_Q),
            KeySlot::new(&key_W),
            KeySlot::new(&key_E),
            KeySlot::new(&key_R),
            KeySlot::new(&key_T),
            KeySlot::new(&key_Y),
            KeySlot::new(&key_U),
            KeySlot::new(&key_I),
            KeySlot::new(&key_O),
            KeySlot::new(&key_P),
        ]),
        row(&[
            KeySlot::new(&key_A),
            KeySlot::new(&key_S),
            KeySlot::new(&key_D),
            KeySlot::new(&key_F),
            KeySlot::new(&key_G),
            KeySlot::new(&key_H),
            KeySlot::new(&key_J),
            KeySlot::new(&key_K),
            KeySlot::new(&key_L),
        ]),
        row(&[
            KeySlot::width(&key_shift_caps_lock, 50.0),
            KeySlot::width(&key_empty, 7.0),
            KeySlot::new(&key_Z),
            KeySlot::new(&key_X),
            KeySlot::new(&key_C),
            KeySlot::new(&key_V),
            KeySlot::new(&key_B),
            KeySlot::new(&key_N),
            KeySlot::new(&key_M),
            KeySlot::width(&key_empty, 7.0),
            KeySlot::width(&key_backspace, 50.0),
        ]),
        row(&bottom_row(&key_to_numeric)),
    ],
};

pub static LAYOUT_ALPHA_NUMERIC: Layout = Layout {
    rows: &[
        row(&[
            KeySlot::new(&key_1),
            KeySlot::new(&key_2),
            KeySlot::new(&key_3),
            KeySlot::new(&key_4),
            KeySlot::new(&key_5),
            KeySlot::new(&key_6),
            KeySlot::new(&key_7),
            KeySlot::new(&key_8),
            KeySlot::new(&key_9),
            KeySlot::new(&key_0),
        ]),
        row(&[
            KeySlot::new(&key_hyphen),
            KeySlot::new(&key_slash),
            KeySlot::new(&key_colon),
            KeySlot::new(&key_semicolon),
            KeySlot::new(&key_paren_left),
            KeySlot::new(&key_paren_right),
            KeySlot::new(&key_dollar),
            KeySlot::new(&key_ampersand),
            KeySlot::new(&key_at),
            KeySlot::new(&key_quote),
        ]),
        row(&[
            KeySlot::width(&key_to_punctuation, 100.0),
            KeySlot::width(&key_empty, 6.0),
            KeySlot::width(&key_period, 49.0),
            KeySlot::width(&key_comma, 49.0),
            KeySlot::width(&key_question, 49.0),
            KeySlot::width(&key_exclamation, 49.0),
            KeySlot::width(&key_apostrophe, 49.0),
            KeySlot::width(&key_empty, 7.0),
            KeySlot::width(&key_backspace, 50.0),
        ]),
        row(&bottom_row(&key_to_alpha)),
    ],
};

pub static LAYOUT_ALPHA_PUNCTUATION: Layout = Layout {
    rows: &[
        row(&[
            KeySlot::new(&key_bracket_left),
            KeySlot::new(&key_bracket_right),
            KeySlot::new(&key_brace_left),
            KeySlot::new(&key_brace_right),
            KeySlot::new(&key_hashtag),
            KeySlot::new(&key_percent),
            KeySlot::new(&key_caret),
            KeySlot::new(&key_asterisk),
            KeySlot::new(&key_plus),
            KeySlot::new(&key_equals),
        ]),
        row(&[
            KeySlot::new(&key_underscore),
            KeySlot::new(&key_backslash),
            KeySlot::new(&key_pipe),
            KeySlot::new(&key_tilde),
            KeySlot::new(&key_less_than),
            KeySlot::new(&key_greater_than),
            KeySlot::new(&key_euro),
            KeySlot::new(&key_pound_sterling),
            KeySlot::new(&key_yen),
            KeySlot::new(&key_bullet),
        ]),
        row(&[
            KeySlot::width(&key_to_numeric, 100.0),
            KeySlot::width(&key_empty, 6.0),
            KeySlot::width(&key_period, 49.0),
            KeySlot::width(&key_comma, 49.0),
            KeySlot::width(&key_question, 49.0),
            KeySlot::width(&key_exclamation, 49.0),
            KeySlot::width(&key_apostrophe, 49.0),
            KeySlot::width(&key_empty, 7.0),
            KeySlot::width(&key_backspace, 50.0),
        ]),
        row(&bottom_row(&key_to_alpha)),
    ],
};

const fn row(key_slots: &'static [KeySlot]) -> Row { Row { gap: 7.0, key_slots } }

const fn bottom_row(leftmost: &'static KeyDef) -> [KeySlot; 3] {
    [KeySlot::width(leftmost, 100.0), KeySlot::width(&key_space, 250.0), KeySlot::width(&key_done, 100.0)]
}
