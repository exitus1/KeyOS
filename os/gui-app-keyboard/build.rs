// SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use resvg::{tiny_skia, usvg};

const ICON_SIZE: usize = 24;
const ICONS: &[&str] = &["backspace", "caps", "shifted", "unshifted"];

const BG_WIDTH: usize = 480;
const BG_HEIGHT: usize = 306;

fn load_svg(p: &str, width: usize, height: usize) -> image::RgbaImage {
    println!("cargo:rerun-if-changed={p}");
    let tree = usvg::Tree::from_data(std::fs::read(p).unwrap().as_slice(), &Default::default()).unwrap();
    let original_size = tree.size();

    let mut buffer = vec![0u8; width * height * 4];
    let mut skia_buffer =
        tiny_skia::PixmapMut::from_bytes(buffer.as_mut_slice(), width as u32, height as u32).unwrap();
    resvg::render(
        &tree,
        tiny_skia::Transform::from_scale(
            width as f32 / original_size.width() as f32,
            height as f32 / original_size.height() as f32,
        ),
        &mut skia_buffer,
    );
    image::RgbaImage::from_raw(width as u32, height as u32, buffer).unwrap()
}

fn add_background(out_dir: &Path, assets: &mut File) {
    for theme in ["light", "dark"] {
        let bg_image = load_svg(&format!("assets/background-{theme}.svg"), BG_WIDTH, BG_HEIGHT);

        let mut bg_file = File::create(out_dir.join(format!("bg-{theme}.raw"))).unwrap();
        bg_file.write_all(&bg_image.into_raw()).unwrap();
    }

    writeln!(assets, "pub const BG_DARK_IMAGE: &[u8] = include_bytes!(\"bg-dark.raw\");").unwrap();
    writeln!(assets, "pub const BG_LIGHT_IMAGE: &[u8] = include_bytes!(\"bg-light.raw\");").unwrap();
    writeln!(assets, "pub const BG_IMAGE_WIDTH: usize = {BG_WIDTH};").unwrap();
    writeln!(assets, "pub const BG_IMAGE_HEIGHT: usize = {BG_HEIGHT};").unwrap();
}

fn add_icon(icon_name: &str, assets: &mut File) {
    let svg_path = format!("../../ui/ui/icons/{icon_name}.svg");
    let icon = load_svg(&svg_path, ICON_SIZE, ICON_SIZE);
    let icon_a: Vec<u8> = icon.pixels().map(|p| p[3]).collect();

    let const_name = icon_name.to_uppercase().replace('-', "_");

    writeln!(assets, "pub const {const_name}: [u8; {ICON_SIZE}*{ICON_SIZE}] = {icon_a:?};").unwrap();
}

fn main() {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let mut assets = File::create(out_dir.join("assets.rs")).unwrap();
    add_background(&out_dir, &mut assets);
    for icon in ICONS {
        add_icon(icon, &mut assets);
    }

    println!("cargo:rerun-if-changed=build.rs");
}
