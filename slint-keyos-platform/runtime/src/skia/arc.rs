// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use slint::{Color as SlintColor, Image, Rgba8Pixel, SharedPixelBuffer};
use tiny_skia::{FillRule, Mask, PathBuilder, Rect, Transform};

use super::color_to_paint_dist;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Corner {
    TL,
    TR,
    BR,
    BL,
}

impl Corner {
    fn get_corner(a: f32) -> Corner {
        let a = (a + 360.0) % 360.0;
        match a {
            a if a >= 0.0 && a < 90.0 => Corner::BR,
            a if a >= 90.0 && a < 180.0 => Corner::BL,
            a if a >= 180.0 && a < 270.0 => Corner::TL,
            a if a >= 270.0 && a < 360.0 => Corner::TR,
            _ => Corner::TR,
        }
    }

    fn next_corner(&self) -> Corner {
        match &self {
            Corner::TL => Corner::TR,
            Corner::TR => Corner::BR,
            Corner::BR => Corner::BL,
            Corner::BL => Corner::TL,
        }
    }

    fn corner_point(&self, w: f32, h: f32) -> (f32, f32) {
        match &self {
            Corner::TL => (0.0, 0.0),
            Corner::TR => (w, 0.0),
            Corner::BL => (0.0, h),
            Corner::BR => (w, h),
        }
    }

    fn distance_to(&self, x: f32, y: f32, w: f32, h: f32) -> f32 {
        let (a, b) = self.corner_point(w, h);
        ((x - a).abs().powi(2) + (y - b).abs().powi(2)).sqrt()
    }
}

fn create_arc_mask(r: f32, width: f32, height: f32, angle_start: f32, angle_end: f32, thikness: f32) -> Mask {
    let mut mask = Mask::new(width as u32, height as u32).unwrap();
    if (angle_end - angle_start).abs() >= 360.0 {
        // full circle - expose everythinh
        let path = {
            let mut pb = PathBuilder::new();
            pb.push_oval(Rect::from_xywh(0.0, 0.0, width, height).unwrap());
            pb.finish().unwrap()
        };
        mask.fill_path(&path, FillRule::Winding, false, Transform::identity());
        return mask;
    }
    if angle_end == angle_start {
        // hide everything
        return mask;
    }

    let clip_path = {
        let x = width / 2.0;
        let y = height / 2.0;

        let point_on_arc =
            |ar: f32| -> (f32, f32) { ((x + r * ar.cos()).round(), (y + r * ar.sin()).round()) };

        let mut pb = PathBuilder::new();
        pb.move_to(x, y); // center of the circle
        let mut corner = Corner::get_corner(angle_start);

        let (sx, sy) = point_on_arc(angle_start.to_radians());
        let (ex, ey) = point_on_arc(angle_end.to_radians());

        pb.line_to(sx, sy);
        let mut skip = (angle_end - angle_start).abs() > 270.0;

        loop {
            let (sx, sy) = corner.corner_point(width, height);
            pb.line_to(sx, sy);
            let dist_to_end = corner.distance_to(ex, ey, width, height);
            if dist_to_end <= r && !skip {
                break;
            }
            corner = corner.next_corner();
            skip = false;
        }

        pb.line_to(ex, ey);
        pb.close();

        let small_r = r * (1.0 - thikness) / 2.0;

        let sr = angle_start.to_radians();
        let er = angle_end.to_radians();

        // round edges
        let small_center_x = x + (r - small_r) * sr.cos();
        let small_center_y = y + (r - small_r) * sr.sin();
        pb.push_circle(small_center_x, small_center_y, small_r);

        let small_center_x = x + (r - small_r) * er.cos();
        let small_center_y = y + (r - small_r) * er.sin();
        pb.push_circle(small_center_x, small_center_y, small_r);

        pb.finish().unwrap()
    };

    mask.fill_path(&clip_path, FillRule::Winding, true, Transform::default());

    mask
}

pub fn doughnut(
    width: f32,
    height: f32,
    angle_start: f32,
    angle_end: f32,
    thikness: f32,
    color: SlintColor,
) -> Image {
    let x = width / 2.0;
    let y = height / 2.0;
    let r = f32::min(x, y);
    let dist = (angle_end - angle_start).abs();
    let paint = color_to_paint_dist(color, dist);

    let mut pixel_buffer = SharedPixelBuffer::<Rgba8Pixel>::new(width as u32, height as u32);
    let mut pixmap =
        tiny_skia::PixmapMut::from_bytes(pixel_buffer.make_mut_bytes(), width as u32, height as u32).unwrap();
    pixmap.fill(tiny_skia::Color::TRANSPARENT);

    let circle = {
        let mut pb = PathBuilder::new();
        pb.push_circle(x, y, r);
        pb.push_circle(x, y, r * thikness);
        pb.finish().unwrap()
    };

    let mut mask: Option<&Mask> = None;
    let m: Mask;
    if dist > 0.0 {
        // rotate 90deg counterclock - doughnut starts from the top side, not right side
        m = create_arc_mask(r, width, height, angle_start - 90.0, angle_end - 90.0, thikness);
        mask = Some(&m);
    };

    pixmap.fill_path(&circle, &paint, FillRule::EvenOdd, Default::default(), mask);

    Image::from_rgba8_premultiplied(pixel_buffer)
}
