// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundationdevices.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::{
        assets::{load_and_verify_asset, Asset},
        splash::show_image_overlay,
    },
    arrayvec::ArrayVec,
    boot_common::gui::{Bounds, Component, TextMessage},
};

pub type DynComponent = boot_common::gui::DynComponent<QrCode>;
pub type Page = boot_common::gui::Page<DynComponent>;

pub struct QrCode {
    bounds: Bounds,
    asset: &'static Asset,
}

impl QrCode {
    pub fn new(asset: &'static Asset, x: i32, y: i32, width: i32, height: i32) -> Self {
        Self { asset, bounds: Bounds::new(x, y, width, height) }
    }
}

impl Component for QrCode {
    #[allow(clippy::vec_init_then_push)] // Saving code space by avoiding vec![]
    fn render(&self) {
        if load_and_verify_asset(self.asset) {
            show_image_overlay(
                self.bounds.x as u16,
                self.bounds.y as u16,
                self.bounds.width as u16,
                self.bounds.height as u16,
            );
        } else {
            TextMessage::new(
                self.bounds.x,
                self.bounds.y,
                self.bounds.width,
                ArrayVec::try_from(&["<Error loading QR code>"] as &[_]).unwrap(),
                true,
            )
            .render();
        }
    }

    fn bounds(&self) -> &Bounds { &self.bounds }

    fn mut_bounds(&mut self) -> &mut Bounds { &mut self.bounds }
}
