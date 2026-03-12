// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use slint::{Image, Rgba8Pixel, SharedPixelBuffer};
use tiny_skia::{
    BlendMode, Color, FillRule, GradientStop, LinearGradient, Mask, Paint, Path, PathBuilder, Point,
    SpreadMode, Stroke, Transform,
};

#[derive(Debug, Clone, Copy)]
pub struct PricePoint {
    pub price: u32,
    pub timestamp: u64,
    pub is_pad: bool,
}

pub fn draw_graph(data: &[PricePoint], w: u32, h: u32, max_height: u32, is_dark_mode: bool) -> Image {
    let mut pixel_buffer = SharedPixelBuffer::<Rgba8Pixel>::new(w, h);
    let mut pixmap = tiny_skia::PixmapMut::from_bytes(pixel_buffer.make_mut_bytes(), w, h).unwrap();
    pixmap.fill(Color::TRANSPARENT);

    if data.len() < 2 {
        return Image::from_rgba8_premultiplied(pixel_buffer);
    }

    const SHADOW_HEIGHT: usize = 6;
    const BOTTOM_VERTICAL_MARGIN: u32 = 6;
    let bg_color = if is_dark_mode {
        Color::from_rgba8(0x11, 0x11, 0x11, 0xff)
    } else {
        Color::from_rgba8(0xf6, 0xf6, 0xf6, 0xff)
    };
    let fg_color = Color::from_rgba8(0xbf, 0x75, 0x5f, 0xff);
    let stale_fg_color = match is_dark_mode {
        true => Color::from_rgba8(0x5a, 0x59, 0x5a, 0xff), // #5A595A
        false => Color::from_rgba8(0x95, 0x93, 0x94, 0xff), // #959394
    };
    let shadow = Color::from_rgba8(0x11, 0x11, 0x11, 0x0d);

    // Normalize the price data into 0..1 range
    let prices: Vec<u32> = data.iter().map(|point| point.price).collect();
    let max_value = *prices.iter().max().unwrap();
    let min_value = *prices.iter().min().unwrap();
    let is_constant = max_value == min_value;
    let normalized: Vec<f32> = prices
        .iter()
        .map(|&v| {
            if max_value == min_value {
                0.0
            } else {
                (v - min_value) as f32 / (max_value - min_value) as f32
            }
        })
        .collect();

    // Scale data to fit within the max_height
    let scaled: Vec<f32> = normalized.iter().map(|&v| v * max_height as f32).collect();

    // Calculate time range for x-axis spacing
    let min_timestamp = data.first().unwrap().timestamp;
    let max_timestamp = data.last().unwrap().timestamp;
    let time_range = max_timestamp.saturating_sub(min_timestamp);
    let num_points = data.len() as f32;

    let mut pb_line = PathBuilder::new();
    let mut pb_line_shadow = PathBuilder::new();
    let mut pb_line_stale = PathBuilder::new();
    let mut pb_line_shadow_stale = PathBuilder::new();
    let mut pb_fill_fresh = PathBuilder::new();
    let mut pb_fill_stale = PathBuilder::new();

    let max_y = h.saturating_sub(BOTTOM_VERTICAL_MARGIN);
    let mut points = Vec::with_capacity(data.len());
    for (i, value) in scaled.iter().enumerate() {
        let timestamp = data[i].timestamp;
        let time_offset = timestamp.saturating_sub(min_timestamp);
        let x = if time_range > 0 {
            (time_offset as f32 / time_range as f32) * w as f32
        } else {
            (i as f32 / (num_points - 1.0)) * w as f32
        };
        let adjusted_value = if is_constant { max_y as f32 - h as f32 * 0.25 } else { *value };
        let shadow_offset = (SHADOW_HEIGHT.saturating_sub(1)) as f32;
        let y = max_y as f32 - adjusted_value;
        let y_shadow = max_y as f32 - (adjusted_value - shadow_offset).max(0.0);
        points.push((x, y, y_shadow));
    }

    let mut last_segment_stale = None;
    for i in 1..points.len() {
        let (prev_x, prev_y, prev_y_shadow) = points[i - 1];
        let (x, y, y_shadow) = points[i];
        let is_pad_segment = data[i - 1].is_pad || data[i].is_pad;
        let fill = if is_pad_segment { &mut pb_fill_stale } else { &mut pb_fill_fresh };
        fill.move_to(prev_x, h as f32);
        fill.line_to(prev_x, prev_y);
        fill.line_to(x, y);
        fill.line_to(x, h as f32);
        fill.close();

        let start_new_run = last_segment_stale.map_or(true, |last| last != is_pad_segment);
        if is_pad_segment {
            if start_new_run {
                pb_line_stale.move_to(prev_x, prev_y);
                pb_line_shadow_stale.move_to(prev_x, prev_y_shadow);
            }
            pb_line_stale.line_to(x, y);
            pb_line_shadow_stale.line_to(x, y_shadow);
        } else {
            if start_new_run {
                pb_line.move_to(prev_x, prev_y);
                pb_line_shadow.move_to(prev_x, prev_y_shadow);
            }
            pb_line.line_to(x, y);
            pb_line_shadow.line_to(x, y_shadow);
        }
        last_segment_stale = Some(is_pad_segment);
    }

    let border_path = rounded_bottom_path(w as f32, h as f32, 16.0, 0.0);
    let graph_path = pb_line.finish();
    let shadow_path = pb_line_shadow.finish();
    let graph_stale_path = pb_line_stale.finish();
    let shadow_stale_path = pb_line_shadow_stale.finish();
    let fill_path_fresh = pb_fill_fresh.finish();
    let fill_path_stale = pb_fill_stale.finish();
    let mut paint_line = Paint::default();
    paint_line.set_color(fg_color);
    let mut paint_line_stale = Paint::default();
    paint_line_stale.set_color(stale_fg_color);
    let mut paint_line_shadow = Paint::default();
    paint_line_shadow.set_color(shadow);
    let mut paint_line_shadow_stale = Paint::default();
    let mut paint_fill_stale = Paint::default();
    paint_line_shadow_stale.set_color(shadow);
    let mut paint_fill = Paint::default();

    let (start, end) =
        (Point::from_xy(w as f32 / 2.0, h as f32), Point::from_xy(w as f32 / 2.0, (h - max_height) as f32));

    let stale_stops = if is_dark_mode {
        [(0.0, Color::from_rgba8(0x86, 0x83, 0x85, 0x33)), (1.0, Color::from_rgba8(0x45, 0x44, 0x44, 0xff))]
    } else {
        [(0.0, Color::from_rgba8(0x86, 0x83, 0x85, 0x33)), (1.0, Color::from_rgba8(0x23, 0x1f, 0x20, 0xff))]
    };
    let fresh_stops = [
        (0.0, Color::from_rgba8(0xD6, 0x8B, 0x6E, 0x00)), // 0% alpha
        (1.0, Color::from_rgba8(0xD6, 0x8B, 0x6E, 0xbf)), // 75% alpha
    ];

    let stale_stops = stale_stops.into_iter().map(|(pos, color)| GradientStop::new(pos, color)).collect();
    paint_fill_stale.shader =
        LinearGradient::new(start, end, stale_stops, SpreadMode::Pad, Transform::identity()).unwrap();
    paint_fill_stale.shader.apply_opacity(1.0);
    paint_fill_stale.blend_mode = BlendMode::SourceOver;

    let fresh_stops = fresh_stops.into_iter().map(|(pos, color)| GradientStop::new(pos, color)).collect();
    paint_fill.shader =
        LinearGradient::new(start, end, fresh_stops, SpreadMode::Pad, Transform::identity()).unwrap();
    paint_fill.shader.apply_opacity(1.0);
    paint_fill.blend_mode = BlendMode::SourceOver;

    let mut graph_stroke = Stroke::default();
    graph_stroke.width = 2.0;
    let mut shadow_stroke = Stroke::default();
    shadow_stroke.width = SHADOW_HEIGHT as f32;

    let mut mask = Mask::new(w, h).unwrap();
    mask.fill_path(&border_path, FillRule::EvenOdd, false, Transform::identity());

    let mut paint_inside = Paint::default();
    paint_inside.set_color(bg_color);
    pixmap.fill_path(&border_path, &paint_inside, FillRule::EvenOdd, Transform::identity(), None);

    if let Some(fill_path_stale) = fill_path_stale {
        pixmap.fill_path(
            &fill_path_stale,
            &paint_fill_stale,
            FillRule::EvenOdd,
            Transform::identity(),
            Some(&mask),
        );
    }
    if let Some(fill_path_fresh) = fill_path_fresh {
        pixmap.fill_path(
            &fill_path_fresh,
            &paint_fill,
            FillRule::EvenOdd,
            Transform::identity(),
            Some(&mask),
        );
    }
    if let Some(graph_path) = graph_path {
        pixmap.stroke_path(&graph_path, &paint_line, &graph_stroke, Transform::identity(), Some(&mask));
    }
    if let Some(shadow_path) = shadow_path {
        pixmap.stroke_path(
            &shadow_path,
            &paint_line_shadow,
            &shadow_stroke,
            Transform::identity(),
            Some(&mask),
        );
    }
    if let Some(graph_stale_path) = graph_stale_path {
        pixmap.stroke_path(
            &graph_stale_path,
            &paint_line_stale,
            &graph_stroke,
            Transform::identity(),
            Some(&mask),
        );
    }
    if let Some(shadow_stale_path) = shadow_stale_path {
        pixmap.stroke_path(
            &shadow_stale_path,
            &paint_line_shadow_stale,
            &shadow_stroke,
            Transform::identity(),
            Some(&mask),
        );
    }

    Image::from_rgba8_premultiplied(pixel_buffer)
}

fn rounded_bottom_path(width: f32, height: f32, border_radius: f32, stroke_width: f32) -> Path {
    let z = stroke_width * 0.5; // little shift for the border
    let width = width - z; // Shrink a bit to let all border inside the box (stroke rendered )
    let height = height - z;
    let b = 0.448 * border_radius;
    let mut pb = PathBuilder::new();
    if border_radius > 0.0 {
        pb.move_to(0.0, z); // top
        pb.line_to(width, z);
        pb.line_to(width, height - border_radius); // right
        pb.cubic_to(
            //  bottom-right
            width,
            height - b,
            width - b,
            height,
            width - border_radius,
            height,
        );
        pb.line_to(border_radius + z, height); // bottom
        pb.cubic_to(
            //  bottom-left
            b,
            height,
            z,
            height - b,
            z,
            height - border_radius,
        );
        pb.line_to(z, border_radius); // left
    }
    pb.close();

    pb.finish().unwrap()
}
