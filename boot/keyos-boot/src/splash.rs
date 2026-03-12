// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundationdevices.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::{assets, batt},
    atsama5d27::{
        lcdc::{ColorMode, LayerConfig, Lcdc, LcdcLayerId},
        pmc::{PeripheralId, Pmc},
        sfc::Sfc,
    },
    boot_common::{
        FB_PB_OVERLAY_ADDR, HEIGHT, PB_OVERLAY_DMA_ADDR, PB_OVERLAY_HEIGHT, SPLASH_DMA_ADDR, WIDTH,
    },
    keyos::BOOT_SPLASH_PHYS_ADDR,
};

// This layer is used during the boot process to be shown behind a progress bar overlay
const SPLASH_LAYER: LcdcLayerId = LcdcLayerId::Ovr1;
// This layer is used after the boot process to be shown above the main UI
const SPLASH_LAYER_AFTER_BOOT: LcdcLayerId = LcdcLayerId::Base;
const PB_OVERLAY_LAYER: LcdcLayerId = LcdcLayerId::Ovr2;
const IMAGE_OVERLAY_LAYER: LcdcLayerId = LcdcLayerId::Ovr1;

pub(crate) fn init_splash_layers(lcdc: &mut Lcdc) {
    load_splash();

    set_splash_to_layer(lcdc, SPLASH_LAYER);

    // Initialize progress bar overlay
    let width = WIDTH as u16;
    let height = PB_OVERLAY_HEIGHT as u16;
    let x = 0;
    let y = HEIGHT as u16 - height;
    lcdc.set_window_pos(PB_OVERLAY_LAYER, x, y);
    lcdc.set_window_size(PB_OVERLAY_LAYER, width, height);
    lcdc.update_attribute(PB_OVERLAY_LAYER);
    lcdc.update_layer(
        &LayerConfig::new(PB_OVERLAY_LAYER, FB_PB_OVERLAY_ADDR, PB_OVERLAY_DMA_ADDR, PB_OVERLAY_DMA_ADDR),
        || (),
    );
    lcdc.enable_layer(PB_OVERLAY_LAYER);
    lcdc.set_rgb_mode_input(PB_OVERLAY_LAYER, ColorMode::Rgba8888);
    lcdc.set_blender_iterated_color_enable(PB_OVERLAY_LAYER, true);
    lcdc.set_blender_use_iterated_color(PB_OVERLAY_LAYER, true);
    lcdc.set_blender_local_alpha_enable(PB_OVERLAY_LAYER, true);
    lcdc.update_overlay_attributes_enable(PB_OVERLAY_LAYER);
    lcdc.update_attribute(PB_OVERLAY_LAYER);
}

fn set_splash_to_layer(lcdc: &mut Lcdc, layer: LcdcLayerId) {
    lcdc.disable_layer(layer);

    // Initialize splash screen overlay
    lcdc.set_window_pos(layer, 0, 0);
    lcdc.set_window_size(layer, WIDTH as u16, HEIGHT as u16);
    lcdc.update_attribute(layer);
    lcdc.update_layer(
        &LayerConfig::new(layer, BOOT_SPLASH_PHYS_ADDR, SPLASH_DMA_ADDR, SPLASH_DMA_ADDR),
        || (),
    );
    lcdc.enable_layer(layer);
    lcdc.set_rgb_mode_input(layer, ColorMode::Argb8888);
}

pub(crate) fn show_splash_layer() {
    let lcdc = Lcdc::new(WIDTH as u16, HEIGHT as u16);
    lcdc.enable_layer(SPLASH_LAYER);
}

pub(crate) fn hide_splash_layer() {
    let lcdc = Lcdc::new(WIDTH as u16, HEIGHT as u16);
    lcdc.disable_layer(SPLASH_LAYER);
}

pub(crate) fn show_progress_layer() {
    let lcdc = Lcdc::new(WIDTH as u16, HEIGHT as u16);
    lcdc.enable_layer(PB_OVERLAY_LAYER);
    lcdc.set_rgb_mode_input(PB_OVERLAY_LAYER, ColorMode::Rgba8888);
    lcdc.set_blender_iterated_color_enable(PB_OVERLAY_LAYER, true);
    lcdc.set_blender_use_iterated_color(PB_OVERLAY_LAYER, true);
    lcdc.set_blender_local_alpha_enable(PB_OVERLAY_LAYER, true);
    lcdc.update_overlay_attributes_enable(PB_OVERLAY_LAYER);
    lcdc.update_attribute(PB_OVERLAY_LAYER);
}

pub(crate) fn hide_progress_layer() {
    let lcdc = Lcdc::new(WIDTH as u16, HEIGHT as u16);
    lcdc.disable_layer(PB_OVERLAY_LAYER);
}

pub(crate) fn set_layers_after_boot() {
    load_splash();
    let mut lcdc = Lcdc::new(WIDTH as u16, HEIGHT as u16);
    lcdc.disable_layer(PB_OVERLAY_LAYER);
    set_splash_to_layer(&mut lcdc, SPLASH_LAYER_AFTER_BOOT);
}

pub(crate) fn set_layers_for_recovery() {
    load_splash();
    let mut lcdc = Lcdc::new(WIDTH as u16, HEIGHT as u16);
    set_splash_to_layer(&mut lcdc, SPLASH_LAYER);
}

pub(crate) fn load_splash() {
    let mut pmc = Pmc::new();
    pmc.enable_peripheral_clock(PeripheralId::Sfc);

    let is_low_battery = matches!(batt::check_low_battery(), batt::BatteryState::Low);

    let splash_asset = match (colorway(), is_low_battery) {
        (fuse::Colorway::Dark, true) => &assets::ASSET_LOWDARK,
        (fuse::Colorway::Dark, false) => &assets::ASSET_DARK,
        (fuse::Colorway::Light, true) => &assets::ASSET_LOWLIGHT,
        (fuse::Colorway::Light, false) => &assets::ASSET_LIGHT,
    };

    if !assets::load_and_verify_asset(splash_asset) {
        loop {
            unsafe {
                core::arch::asm!("wfi");
            }
        }
    }
}

pub(crate) fn show_image_overlay(x: u16, y: u16, w: u16, h: u16) {
    let lcdc = Lcdc::new(w, h);

    lcdc.set_window_pos(IMAGE_OVERLAY_LAYER, x, y);
    lcdc.set_window_size(IMAGE_OVERLAY_LAYER, w, h);
    lcdc.update_attribute(IMAGE_OVERLAY_LAYER);
    lcdc.update_layer(
        &LayerConfig::new(IMAGE_OVERLAY_LAYER, BOOT_SPLASH_PHYS_ADDR, SPLASH_DMA_ADDR, SPLASH_DMA_ADDR),
        || (),
    );
    lcdc.enable_layer(IMAGE_OVERLAY_LAYER);
    lcdc.set_rgb_mode_input(IMAGE_OVERLAY_LAYER, ColorMode::Argb8888);
}

pub(crate) fn hide_image_overlay() {
    let lcdc = Lcdc::new(WIDTH as u16, HEIGHT as u16);
    lcdc.disable_layer(IMAGE_OVERLAY_LAYER);
}

pub(crate) fn colorway() -> fuse::Colorway { fuse::get_colorway(&Sfc::new()).unwrap_or(fuse::Colorway::Dark) }
