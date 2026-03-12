// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundationdevices.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::{splash::colorway, verify::OS_VERSION},
    boot_common::{
        colors::UIColors, display::ArgbDisplay, fonts::SOURCE_CODE_PRO_FONT, get_pit, PB_HEIGHT, WIDTH,
    },
    core::str,
    embedded_graphics::{
        mono_font::MonoTextStyle,
        prelude::{DrawTarget, Point, Size},
        primitives::{PrimitiveStyleBuilder, Rectangle, StyledDrawable},
        text::Text as EGText,
        Drawable,
    },
    keyos::MASTER_CLOCK_SPEED,
};

pub(crate) static mut DISPLAY_PB_OVERLAY: Option<ArgbDisplay> = None;

const PB_WIDTH: u32 = 270;
const PB_Y: i32 = 0;
const PB_X: i32 = 105;
const PB_TITLE_Y: i32 = 30;

#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum ProgressBarMessage {
    None = 0,
    VerifyingMain,
    VerifyingCustom,
    VerifyingRecovery,
    VerificationFailed,
    LowBattery,
    LowBatteryCharging,
}

pub struct ProgressBar {
    percent: u32,
    message: [u8; 40],
}

impl ProgressBar {
    pub fn default() -> Self { Self { percent: 0, message: [0; 40] } }

    pub fn message_length(&self) -> usize {
        self.message.iter().position(|&byte| byte == 0).unwrap_or(self.message.len())
    }

    pub fn set_percent(&mut self, percent: u32) {
        self.percent = percent.min(100);
        self.render_bar();

        let mut pit = get_pit();
        pit.busy_wait_ms(MASTER_CLOCK_SPEED, 15);

        // Reset the watchdog when progress is updated
        boot_common::wdt_reset();
    }

    pub fn set_message(&mut self, message: ProgressBarMessage) {
        match message {
            ProgressBarMessage::None => self.format_static_message(""),
            ProgressBarMessage::VerifyingMain => self.format_version(true, false),
            ProgressBarMessage::VerifyingRecovery => self.format_version(false, false),
            ProgressBarMessage::VerifyingCustom => self.format_version(true, true),
            ProgressBarMessage::VerificationFailed => {
                self.format_static_message("Invalid firmware signature")
            }
            ProgressBarMessage::LowBattery => self.format_static_message("Low battery, please charge."),
            ProgressBarMessage::LowBatteryCharging => self.format_static_message("Low battery. Charging..."),
        }
        self.render_message();
    }

    fn format_static_message(&mut self, message: &str) {
        let message_bytes = message.as_bytes();
        let length = message_bytes.len().min(self.message.len());
        self.message.fill(0);
        self.message[..length].copy_from_slice(&message_bytes[..length]);
    }

    fn format_version(&mut self, is_main: bool, is_custom: bool) {
        let ver_title = if is_main {
            if is_custom {
                "Custom v"
            } else {
                "KeyOS v"
            }
        } else {
            "KeyOS Recovery v"
        };
        self.format_static_message(ver_title);

        let version = unsafe { (*core::ptr::addr_of!(OS_VERSION)).as_ref() };

        if let Some(version_bytes) = version {
            let current_len = self.message_length();
            let available_space = self.message.len().saturating_sub(current_len);
            let version_len = version_bytes.len().min(available_space);
            self.message[current_len..current_len + version_len]
                .copy_from_slice(&version_bytes[..version_len]);
        } else {
            // TODO: This should show an error screen instead, not just keep booting.
            let unknown_message =
                if is_main { "KeyOS <unknown version>" } else { "KeyOS Recovery <unknown version>" };
            self.format_static_message(unknown_message);
        }
    }

    fn render_message(&self) {
        let font = SOURCE_CODE_PRO_FONT;
        let height = font.character_size.height as i32;
        let pos_fill = Point::new(0, PB_TITLE_Y - height);
        let message_len = self.message_length();
        let text_x = (WIDTH as i32 / 2) - (font.character_size.width as i32 * message_len as i32) / 2;
        let pos_text = Point::new(text_x, PB_TITLE_Y);
        let color = match colorway() {
            fuse::Colorway::Dark => UIColors::PRIMARY_WHITE,
            fuse::Colorway::Light => UIColors::NEUTRAL_900,
        };

        if let Some(display) = unsafe { (*core::ptr::addr_of_mut!(DISPLAY_PB_OVERLAY)).as_mut() } {
            display
                .fill_solid(
                    &Rectangle::new(pos_fill, Size::new(WIDTH as u32, 2 * height as u32)),
                    UIColors::TRANSPARENT_BLACK,
                )
                .ok();

            // Safety: .message is only set by format_static_message, and format_version, both of
            // which operate with proper &str-s.
            let message_str = unsafe { str::from_utf8_unchecked(&self.message[..message_len]) };
            EGText::new(message_str, pos_text, MonoTextStyle::new(&font, color.into())).draw(display).ok();
        }
    }

    fn render_bar(&self) {
        let (bg_color, fg_color) = match colorway() {
            fuse::Colorway::Dark => (UIColors::NEUTRAL_800, UIColors::PRIMARY_WHITE),
            fuse::Colorway::Light => (UIColors::NEUTRAL_500, UIColors::NEUTRAL_950),
        };
        let pb_style = PrimitiveStyleBuilder::new().fill_color(bg_color.into()).build();

        let pb_fill_style = PrimitiveStyleBuilder::new().fill_color(fg_color.into()).build();

        if let Some(display) = unsafe { (*core::ptr::addr_of_mut!(DISPLAY_PB_OVERLAY)).as_mut() } {
            let rect = Rectangle::new(Point::new(PB_X, PB_Y), Size::new(PB_WIDTH, PB_HEIGHT));
            rect.draw_styled(&pb_style, display).ok();

            let bar_width = (PB_WIDTH * self.percent) / 100;
            let rect = Rectangle::new(Point::new(PB_X, PB_Y), Size::new(bar_width, PB_HEIGHT));
            rect.draw_styled(&pb_fill_style, display).ok();
        }
    }
}
