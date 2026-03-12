// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use slint::{Color as SlintColor, Image, Rgba8Pixel, SharedPixelBuffer};
use tiny_skia::{Color, Paint, PathBuilder, PixmapMut, Stroke, Transform};

pub fn loader(size: f32, tick_number: i32, stroke_color: SlintColor, active_color: SlintColor) -> Image {
    let tick = tick_number as usize % 8;
    let size_u32 = size as u32;

    let mut buffer = SharedPixelBuffer::<Rgba8Pixel>::new(size_u32, size_u32);
    let mut pixmap = PixmapMut::from_bytes(buffer.make_mut_bytes(), size_u32, size_u32).unwrap();
    pixmap.fill(Color::TRANSPARENT);

    let center = size / 2.0;
    let radius = size * 0.25;
    let segment_length = size * 0.20;
    let half_segment = segment_length / 2.0;
    let inner_radius = radius - half_segment;
    let outer_radius = radius + half_segment;

    let stroke_color_skia = Color::from_rgba8(
        stroke_color.red(),
        stroke_color.green(),
        stroke_color.blue(),
        stroke_color.alpha(),
    );

    let active_color_skia = Color::from_rgba8(
        active_color.red(),
        active_color.green(),
        active_color.blue(),
        active_color.alpha(),
    );

    let stroke = stroke_default();

    // Draw all segments in stroke color
    for i in 0..8 {
        let angle = i as f32 * std::f32::consts::PI / 4.0;
        let color = if i == tick { active_color_skia } else { stroke_color_skia };
        render_segment(&mut pixmap, center, inner_radius, outer_radius, angle, color, &stroke);
    }

    Image::from_rgba8_premultiplied(buffer)
}

#[inline]
fn render_segment(
    pixmap: &mut PixmapMut,
    center: f32,
    inner_radius: f32,
    outer_radius: f32,
    angle: f32,
    color: Color,
    stroke: &Stroke,
) {
    let mut path = PathBuilder::new();
    let cos = angle.cos();
    let sin = angle.sin();

    path.move_to(center + inner_radius * cos, center + inner_radius * sin);
    path.line_to(center + outer_radius * cos, center + outer_radius * sin);

    if let Some(path) = path.finish() {
        let mut paint = Paint::default();
        paint.anti_alias = true;
        paint.set_color(color);
        pixmap.stroke_path(&path, &paint, stroke, Transform::identity(), None);
    }
}

#[inline]
fn stroke_default() -> Stroke {
    let mut stroke = Stroke::default();
    stroke.width = 2.4;
    stroke
}
