// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundationdevices.com>
// SPDX-License-Identifier: GPL-3.0-or-later

pub mod assets {
    include!(concat!(env!("OUT_DIR"), "/assets.rs"));
}

use std::{rc::Rc, time::Instant};

use assets::{BG_IMAGE_HEIGHT, BG_IMAGE_WIDTH};
use gui_server_api::{
    consts::{DEFAULT_KEYBOARD_HEIGHT, KEYBOARD_TOP_BAR_MARGIN, SCREEN_HEIGHT, SCREEN_WIDTH},
    InputMessage, Key, KeyboardKind,
};
use tiny_skia::{Pixmap, PixmapMut, Rect, Transform};
use xous_api_ticktimer::TicktimerCallback;

use crate::{
    cache::with_cached_pixmap,
    keys::{KeyAction, KeyDef},
    layout::{
        alpha::{
            LAYOUT_ALPHA_LOWER, LAYOUT_ALPHA_NUMERIC, LAYOUT_ALPHA_PUNCTUATION, LAYOUT_ALPHA_UPPER,
            LAYOUT_ALPHA_UPPER_CAPS,
        },
        decimal::LAYOUT_DECIMAL,
        numeric::LAYOUT_NUMERIC,
        Layout, LayoutType,
    },
    overlay::OverlayCache,
    sliding::{CursorSliding, SlideEvent},
    HapticsApi,
};

pub const KEY_DEFAULT_WIDTH: f32 = 40.0;
pub const KEY_HEIGHT: f32 = 58.0;
pub const KEY_BORDER_RADIUS: f32 = 12.0;
pub const KEY_V_MARGIN: f32 = (ROW_HEIGHT - KEY_HEIGHT) / 2.0;

pub const ROW_HEIGHT: f32 = 74.0;
pub const KEYS_TOP_MARGIN: f32 = 4_f32;

pub const KEY_FONT_SCALE: f32 = 38.0;

const LONG_OVERLAY_DELAY: usize = 500;

pub struct KeyboardState {
    kind: KeyboardKind,
    shift_state: ShiftState,
    layout_type: LayoutType,
    layout: &'static Layout,

    overlay_cache: OverlayCache,

    show_overlay_long: bool,

    key_pressed: Option<PressedKey>,
    overlay_x: Option<i32>,
    overlay_char: Option<char>,
    key_pressed_pixmap: Option<Pixmap>,

    haptics_api: Rc<HapticsApi>,
    long_press_callback: TicktimerCallback,

    sliding: CursorSliding,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShiftState {
    #[default]
    Lowercase,
    Uppercase,
    CapsLock,
}

impl ShiftState {
    pub fn next(&self) -> Self {
        match self {
            ShiftState::Lowercase => ShiftState::Uppercase,
            ShiftState::Uppercase => ShiftState::CapsLock,
            ShiftState::CapsLock => ShiftState::Lowercase,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct PressedKey {
    id: u64,

    x: f32,
    y: f32,

    at: Instant,
}

impl KeyboardState {
    pub fn new(
        kind: KeyboardKind,
        haptics_api: Rc<HapticsApi>,
        long_press_callback: TicktimerCallback,
    ) -> Self {
        const _: () = assert!(SCREEN_WIDTH == BG_IMAGE_WIDTH);
        const _: () = assert!(DEFAULT_KEYBOARD_HEIGHT == BG_IMAGE_HEIGHT + KEYBOARD_TOP_BAR_MARGIN);
        // Create initial state
        let mut result = Self {
            kind,
            shift_state: Default::default(),
            layout_type: Default::default(),
            layout: &LAYOUT_ALPHA_LOWER,

            key_pressed: None,
            key_pressed_pixmap: None,

            overlay_cache: Default::default(),

            overlay_x: None,
            overlay_char: None,

            show_overlay_long: false,

            haptics_api,
            long_press_callback,

            sliding: CursorSliding::new(),
        };
        result.refresh_layout();
        result
    }

    pub fn set_kind(&mut self, kind: KeyboardKind) {
        self.kind = kind;
        self.cleanup();
        self.shift_state = Default::default();
        self.layout_type = Default::default();
        self.refresh_layout();
    }

    pub fn get_key(&self, click_x: f32, click_y: f32) -> Option<(&KeyDef, f32, f32)> {
        let (y, row) = space_filling_find(
            self.layout.rows_with_coords(),
            KEYBOARD_TOP_BAR_MARGIN as f32..SCREEN_HEIGHT as f32,
            click_y,
        )?;
        let (x, key) = space_filling_find(row.keys_with_coords(), 0.0..SCREEN_WIDTH as f32, click_x)?;
        Some((key.key, x, y))
    }

    pub fn get_key_def(&self, key_id: u64) -> Option<&KeyDef> {
        let slot = self.layout.get_key(key_id)?;
        Some(slot.key)
    }

    pub fn refresh_layout(&mut self) {
        self.layout = match self.kind {
            KeyboardKind::Alphanumeric | KeyboardKind::Password | KeyboardKind::Email => {
                match self.layout_type {
                    LayoutType::Alphabetic => match self.shift_state {
                        ShiftState::Lowercase => &LAYOUT_ALPHA_LOWER,
                        ShiftState::Uppercase => &LAYOUT_ALPHA_UPPER,
                        ShiftState::CapsLock => &LAYOUT_ALPHA_UPPER_CAPS,
                    },
                    LayoutType::Numeric => &LAYOUT_ALPHA_NUMERIC,
                    LayoutType::Punctuation => &LAYOUT_ALPHA_PUNCTUATION,
                }
            }
            KeyboardKind::Numbers => &LAYOUT_NUMERIC,
            KeyboardKind::Decimal => &LAYOUT_DECIMAL,
        };
    }

    pub fn shift_state_request(&mut self, shift_on: bool) {
        if shift_on {
            if self.shift_state == ShiftState::Lowercase {
                self.shift_state = ShiftState::Uppercase;
                self.refresh_layout();
            }
        } else {
            if self.shift_state == ShiftState::Uppercase {
                self.shift_state = ShiftState::Lowercase;
                self.refresh_layout();
            }
        }
    }

    pub fn on_pressed(&mut self, x: f32, y: f32) {
        let Some((key, x, y)) = self.get_key(x, y) else { return };
        let key = key.clone();
        let id = key.id();

        self.key_pressed = Some(PressedKey { id, x, y, at: Instant::now() });
        self.overlay_x = Some(x as i32);
        if key.overlay.len() > 1 {
            self.long_press_callback.request(LONG_OVERLAY_DELAY, InputMessage::Custom4 as usize, 0);
        }

        if key.on_released == KeyAction::Space {
            self.sliding.start(x, y);
        }

        self.haptics_api.vibrate(haptics::HapticPattern::SharpClick100);
    }

    fn cleanup(&mut self) -> (Option<char>, Option<PressedKey>) {
        self.show_overlay_long = false;
        self.overlay_x = None;

        let was_sliding = self.sliding.reset();

        let last_key = if was_sliding { None } else { self.key_pressed };
        self.key_pressed = None;
        self.key_pressed_pixmap = None;
        let overlay_char = if was_sliding { None } else { self.overlay_char };
        self.overlay_char = None;

        self.long_press_callback.cancel(InputMessage::Custom4 as usize);

        (overlay_char, last_key)
    }

    pub fn on_released(&mut self, x: f32, y: f32) -> Option<Key> {
        log::debug!("released {x}, {y}");

        let (overlay_char, last_key) = self.cleanup();

        if let Some(ch) = overlay_char {
            self.shift_state_request(false);
            return Some(Key::Char(ch as usize));
        }

        let key = last_key.and_then(|key| self.get_key_def(key.id));

        let Some(key) = key else {
            return None;
        };

        match key.on_released {
            KeyAction::Insert => {
                let result = key.label.chars().next().map(|c| Key::Char(c as usize));
                self.shift_state_request(false);
                result
            }
            KeyAction::None => None,
            KeyAction::Return => Some(Key::Char('\n' as usize)),
            KeyAction::Backspace => Some(Key::Backspace),
            KeyAction::Space => Some(Key::Char(' ' as usize)),
            KeyAction::Shift => {
                self.shift_state = self.shift_state.next();
                self.refresh_layout();
                None
            }
            KeyAction::ChangeLayer(layer_type) => {
                self.layout_type = layer_type;
                self.refresh_layout();
                None
            }
        }
    }

    pub fn on_moved(&mut self, x: f32, y: f32) -> Option<Key> {
        if let Some(event) = self.sliding.on_moved(x, y) {
            return match event {
                SlideEvent::Activated => {
                    self.long_press_callback.cancel(InputMessage::Custom4 as usize);
                    self.show_overlay_long = false;
                    self.overlay_x = None;
                    self.overlay_char = None;
                    self.key_pressed_pixmap = None;
                    self.haptics_api.click();
                    None
                }
                SlideEvent::EmitKey(key) => {
                    self.haptics_api.click();
                    Some(key)
                }
                SlideEvent::Idle => None,
            };
        }

        // --- Normal (non-sliding) move handling ---
        const HALF_HEIGHT: f32 = ROW_HEIGHT * 0.5;

        let Some(key_pressed) = &self.key_pressed else { return None };

        // check if moved away from pressed char
        if (key_pressed.y + HALF_HEIGHT - y).abs() > HALF_HEIGHT * 3.0 {
            // forget pressed key;
            let _ = self.cleanup();
        } else {
            // overlay shown, we need to select char from overlay
            self.overlay_x = Some(x as i32);
        }

        None
    }

    pub fn on_long_press(&mut self) {
        self.haptics_api.vibrate(haptics::HapticPattern::SharpClick100);
        self.show_overlay_long = true;
    }

    pub fn draw(&mut self, target: &mut PixmapMut<'_>) {
        while with_cached_pixmap(&self.layout, |layout_image| {
            target.data_mut().copy_from_slice(layout_image.data());
        })
        .is_none()
        {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        self.draw_overlay(target);
    }

    fn draw_overlay(&mut self, pixmap: &mut PixmapMut) {
        // Don't draw any key overlay while cursor sliding is active.
        if self.sliding.is_active() {
            return;
        }

        let Some(key_pressed) = self.key_pressed else {
            return;
        };
        // render overlay if necessary
        let Some(slot) = self.layout.get_key(key_pressed.id) else { return };

        let x = key_pressed.x;
        let y = key_pressed.y;
        let rect = &Rect::from_xywh(x, y, slot.width, KEY_HEIGHT).unwrap();

        let Some(mouse_x) = self.overlay_x else {
            return;
        };

        if slot.key.overlay == "" {
            let pressed = self.key_pressed_pixmap.get_or_insert_with(|| {
                let mut pressed = Pixmap::new(slot.width.ceil() as u32 + 2, KEY_HEIGHT as u32 + 2).unwrap();
                slot.draw(x.fract() + 1.0, y.fract() + 1.0, &mut pressed.as_mut(), true);
                pressed
            });
            pixmap.draw_pixmap(
                x as i32 - 1,
                y as i32 - 1,
                pressed.as_ref(),
                &Default::default(),
                Transform::identity(),
                None,
            );
        } else {
            let text = if self.show_overlay_long {
                slot.key.overlay
            } else {
                if let Some(c) = slot.key.label.chars().next() {
                    &slot.key.label[..c.len_utf8()]
                } else {
                    "?"
                }
            };
            let ch = crate::overlay::draw(&mut self.overlay_cache, &text, pixmap, rect, mouse_x);
            if let Some(ch) = ch {
                self.overlay_char = Some(ch);
            }
        }
    }
}

fn space_filling_find<'a, T>(
    iter: impl Iterator<Item = (core::ops::Range<f32>, &'a T)>,
    full_range: core::ops::Range<f32>,
    click: f32,
) -> Option<(f32, &'a T)> {
    let mut start = full_range.start;
    let mut iter = iter.peekable();
    while let Some((range, result)) = iter.next() {
        let end = if let Some((next_range, _)) = iter.peek() {
            (range.end + next_range.start) * 0.5
        } else {
            full_range.end
        };
        if (start..end).contains(&click) {
            return Some((range.start, result));
        }
        start = end;
    }
    None
}
