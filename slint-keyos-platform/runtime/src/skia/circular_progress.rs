// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use slint::{Color as SlintColor, Image, Rgba8Pixel, SharedPixelBuffer};
use tiny_skia::{Color, FillRule, Mask, Paint, PathBuilder, PixmapMut, Transform};

const NUM_SEGMENTS: usize = 10;

pub fn circular_progress(
    size: f32,
    tick_number: i32,
    stroke_color: SlintColor,
    active_color: SlintColor,
    thickness: f32,
) -> Image {
    let size_u32 = size as u32;
    let tick = tick_number as usize % NUM_SEGMENTS;

    let mut buffer = SharedPixelBuffer::<Rgba8Pixel>::new(size_u32, size_u32);
    let mut pixmap = PixmapMut::from_bytes(buffer.make_mut_bytes(), size_u32, size_u32).unwrap();
    pixmap.fill(Color::TRANSPARENT);

    let center_x = size / 2.0;
    let center_y = size / 2.0;
    let radius = size / 2.0;

    // Draw background donut
    draw_donut(&mut pixmap, center_x, center_y, radius, thickness, stroke_color, None);

    // Calculate angles for the active segment
    let segment_angle = 360.0 / NUM_SEGMENTS as f32;
    let gap_angle = 4.0; // Fixed 4 degree gap between segments
    let arc_angle = segment_angle - gap_angle;

    let start_angle = tick as f32 * segment_angle + gap_angle / 2.0;
    let end_angle = start_angle + arc_angle;

    // Create mask for the active segment with rounded ends
    let mask = create_segment_mask(size_u32, center_x, center_y, radius, thickness, start_angle, end_angle);

    // Draw active segment
    draw_donut(&mut pixmap, center_x, center_y, radius, thickness, active_color, Some(&mask));

    Image::from_rgba8_premultiplied(buffer)
}

fn draw_donut(
    pixmap: &mut PixmapMut,
    center_x: f32,
    center_y: f32,
    radius: f32,
    thickness: f32,
    color: SlintColor,
    mask: Option<&Mask>,
) {
    let mut pb = PathBuilder::new();
    pb.push_circle(center_x, center_y, radius);
    pb.push_circle(center_x, center_y, radius * (1.0 - thickness));
    let path = pb.finish().unwrap();

    let mut paint = Paint::default();
    paint.set_color(Color::from_rgba8(color.red(), color.green(), color.blue(), color.alpha()));

    pixmap.fill_path(&path, &paint, FillRule::EvenOdd, Transform::identity(), mask);
}

fn create_segment_mask(
    size: u32,
    center_x: f32,
    center_y: f32,
    radius: f32,
    thickness: f32,
    start_angle: f32,
    end_angle: f32,
) -> Mask {
    let mut mask = Mask::new(size, size).unwrap();

    // Convert to radians and adjust for starting at top (-90 degrees)
    let start_rad = (start_angle - 90.0).to_radians();
    let end_rad = (end_angle - 90.0).to_radians();

    let mut pb = PathBuilder::new();

    // Move to center
    pb.move_to(center_x, center_y);

    // Line to start of arc
    let start_x = center_x + radius * start_rad.cos();
    let start_y = center_y + radius * start_rad.sin();
    pb.line_to(start_x, start_y);

    // Arc to end point
    let arc_angle = end_angle - start_angle;
    let steps = ((arc_angle / 3.0).ceil() as i32).max(10);
    let step_angle = (end_rad - start_rad) / steps as f32;

    for i in 1..=steps {
        let angle = start_rad + step_angle * i as f32;
        let x = center_x + radius * angle.cos();
        let y = center_y + radius * angle.sin();
        pb.line_to(x, y);
    }

    // Back to center
    pb.close();

    // Add circles for rounded ends
    let cap_radius = radius * thickness / 2.0;
    let mid_radius = radius - cap_radius;

    let start_cap_x = center_x + mid_radius * start_rad.cos();
    let start_cap_y = center_y + mid_radius * start_rad.sin();
    pb.push_circle(start_cap_x, start_cap_y, cap_radius);

    let end_cap_x = center_x + mid_radius * end_rad.cos();
    let end_cap_y = center_y + mid_radius * end_rad.sin();
    pb.push_circle(end_cap_x, end_cap_y, cap_radius);

    let path = pb.finish().unwrap();
    mask.fill_path(&path, FillRule::Winding, true, Transform::identity());

    mask
}
