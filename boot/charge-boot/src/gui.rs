// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundationdevices.com>
// SPDX-License-Identifier: GPL-3.0-or-later

#![allow(dead_code)]

use {
    crate::batt::{self, batt_color},
    atsama5d27::display::FramebufDisplay,
    boot_common::{
        colors::{DarkPalette, UIColors},
        display::DISPLAY,
        fonts::SOURCE_CODE_PRO_FONT,
        gui::{text_width, Bounds, Component},
        theme::UISize,
    },
    embedded_graphics::{
        mono_font::MonoTextStyle,
        prelude::{Point, Size, *},
        primitives::{PrimitiveStyleBuilder, Rectangle, RoundedRectangle, StrokeAlignment},
        text::Text as EGText,
        Drawable,
    },
};

pub type DynComponent = boot_common::gui::DynComponent<BatteryIcon>;
pub type Page = boot_common::gui::Page<DynComponent>;

pub struct BatteryIcon {
    bounds: Bounds,
}

impl BatteryIcon {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self { bounds: Bounds { x, y, width, height } }
    }

    fn draw_outline(&self, display: &mut FramebufDisplay) {
        let bounds = self.bounds();
        let button_rect = Rectangle::new(
            Point::new(bounds.x, bounds.y),
            Size::new(bounds.width as u32, bounds.height as u32),
        );

        let style =
            PrimitiveStyleBuilder::new().stroke_color(DarkPalette::CONTENT_PRIMARY).stroke_width(5).build();
        RoundedRectangle::with_equal_corners(button_rect, Size::new_equal(UISize::SZ6 as u32))
            .into_styled(style)
            .draw(display)
            .ok();

        let right_pin_rect = Rectangle::new(
            Point::new(bounds.x + bounds.width, bounds.y + bounds.height / 4),
            Size::new(bounds.width as u32 / 15, bounds.height as u32 / 2),
        );

        RoundedRectangle::new(
            right_pin_rect,
            embedded_graphics::primitives::CornerRadii {
                top_left: Size::zero(),
                top_right: Size::new_equal(UISize::SZ2 as u32),
                bottom_right: Size::new_equal(UISize::SZ2 as u32),
                bottom_left: Size::zero(),
            },
        )
        .into_styled(style)
        .draw(display)
        .ok();
    }

    fn draw_charge_state_rect(&self, display: &mut FramebufDisplay, soc: i32) {
        let bounds = self.bounds();
        let width = (bounds.width - UISize::SZ4) * soc / 100;
        let charge_state_rect = Rectangle::new(
            Point::new(bounds.x + UISize::SZ2, bounds.y + UISize::SZ2),
            Size::new(width as u32, (bounds.height - UISize::SZ4) as u32),
        );
        let charge_state_style = PrimitiveStyleBuilder::new().fill_color(batt_color(soc as _)).build();

        RoundedRectangle::with_equal_corners(charge_state_rect, Size::new_equal(UISize::SZ4 as u32))
            .into_styled(charge_state_style)
            .draw(display)
            .ok();
    }

    fn draw_lightning(&self, display: &mut FramebufDisplay) {
        let bounds = self.bounds();

        let cx = bounds.x + bounds.width / 2;
        let cy = bounds.y + bounds.height / 2;
        let tri1 = embedded_graphics::primitives::Triangle::new(
            Point::new(cx + 15, cy - 70),
            Point::new(cx - 40, cy + 5),
            Point::new(cx, cy + 5),
        );
        let tri2 = embedded_graphics::primitives::Triangle::new(
            Point::new(cx - 15, cy + 70),
            Point::new(cx + 40, cy - 5),
            Point::new(cx, cy - 5),
        );

        let style = PrimitiveStyleBuilder::new()
            .fill_color(UIColors::PRIMARY_WHITE)
            .stroke_color(UIColors::PRIMARY_BLACK)
            .stroke_width(8)
            .stroke_alignment(StrokeAlignment::Outside)
            .build();

        tri1.clone().into_styled(style).draw(display).ok();
        tri2.into_styled(style).draw(display).ok();
        tri1.into_styled(PrimitiveStyleBuilder::new().fill_color(UIColors::PRIMARY_WHITE).build())
            .draw(display)
            .ok();
    }

    fn draw_text(&self, display: &mut FramebufDisplay, soc: u8) {
        let bounds = self.bounds();
        let mut text = [b' '; 4];
        if soc >= 100 {
            text[0] = b'1'
        }
        if soc > 10 {
            text[1] = b'0' + (soc / 10) % 10;
        }
        let font = SOURCE_CODE_PRO_FONT;
        text[2] = b'0' + soc % 10;
        text[3] = b'%';
        let text = str::from_utf8(text.as_slice()).unwrap();
        let x = bounds.x + (bounds.width / 2) - (text_width(text, &font) / 2);
        EGText::new(
            text,
            Point::new(x, bounds.y + bounds.height + UISize::SZ8),
            MonoTextStyle::new(&font, DarkPalette::CONTENT_PRIMARY),
        )
        .draw(display)
        .ok();
    }
}

impl Component for BatteryIcon {
    fn render(&self) {
        if let Some(display) = unsafe { (*core::ptr::addr_of_mut!(DISPLAY)).as_mut() } {
            let soc = batt::get_battery_soc();
            self.draw_outline(display);
            self.draw_charge_state_rect(display, soc as i32);
            if batt::is_charging() {
                self.draw_lightning(display);
            }
            self.draw_text(display, soc as u8);
        }
    }

    fn bounds(&self) -> &Bounds { &self.bounds }

    fn mut_bounds(&mut self) -> &mut Bounds { &mut self.bounds }
}
