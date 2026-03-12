// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use slint::{
    private_unstable_api::re_exports::{Image, ImageInner, Rgba8Pixel, SharedImageBuffer, SharedPixelBuffer},
    Color as SlintColor,
};
use tiny_skia::{GradientStop, LinearGradient, Paint, Point, Rect, SpreadMode, Transform};

use super::color_to_color;

fn get_stops() -> Vec<GradientStop> {
    vec![
        (0.00, 1.0, 0.0, 0.0, 1.0),
        (0.15, 1.0, 1.0, 0.0, 1.0),
        (0.30, 0.0, 1.0, 0.0, 1.0),
        (0.45, 0.0, 1.0, 1.0, 1.0),
        (0.65, 0.0, 0.0, 1.0, 1.0),
        (0.90, 1.0, 0.0, 1.0, 1.0),
        (1.00, 1.0, 0.0, 0.0, 1.0),
    ]
    .iter()
    .map(|a| GradientStop::new(a.0, tiny_skia::Color::from_rgba(a.1, a.2, a.3, a.4).unwrap()))
    .collect()
}

fn get_start_end(width: f32, height: f32, vertical: bool) -> (Point, Point) {
    if vertical {
        (Point::from_xy(width * 0.5, 0.0), Point::from_xy(width * 0.5, height))
    } else {
        (Point::from_xy(0.0, height * 0.5), Point::from_xy(width, height * 0.5))
    }
}

pub fn hue_slider(width: f32, height: f32, vertical: bool) -> Image {
    let w = width as u32;
    let h = height as u32;
    let mut pixel_buffer = SharedPixelBuffer::<Rgba8Pixel>::new(w, h);
    let mut pixmap = tiny_skia::PixmapMut::from_bytes(pixel_buffer.make_mut_bytes(), w, h).unwrap();
    let (start, end) = get_start_end(width, height, vertical);
    let mut paint = Paint::<'_> { anti_alias: true, ..Default::default() };
    paint.shader =
        LinearGradient::new(start, end, get_stops(), SpreadMode::Pad, Transform::identity()).unwrap();

    pixmap.fill_rect(Rect::from_xywh(0.0, 0.0, width, height).unwrap(), &paint, Transform::identity(), None);

    Image::from_rgba8_premultiplied(pixel_buffer)
}

pub fn color_palette(width: f32, height: f32, color: SlintColor) -> Image {
    let w = width as u32;
    let h = height as u32;
    let mut pixel_buffer = SharedPixelBuffer::<Rgba8Pixel>::new(w, h);
    let mut pixmap = tiny_skia::PixmapMut::from_bytes(pixel_buffer.make_mut_bytes(), w, h).unwrap();
    pixmap.fill(color_to_color(color));

    {
        let (start, end) = get_start_end(width, height, false);
        let mut paint = Paint::<'_> { anti_alias: true, ..Default::default() };

        paint.shader = LinearGradient::new(
            start,
            end,
            vec![
                GradientStop::new(0.00, tiny_skia::Color::from_rgba(1.0, 1.0, 1.0, 1.0).unwrap()),
                GradientStop::new(1.00, tiny_skia::Color::from_rgba(1.0, 1.0, 1.0, 0.0).unwrap()),
            ],
            SpreadMode::Pad,
            Transform::identity(),
        )
        .unwrap();

        pixmap.fill_rect(
            Rect::from_xywh(0.0, 0.0, width, height).unwrap(),
            &paint,
            Transform::identity(),
            None,
        );
    }
    {
        let (start, end) = get_start_end(width, height, true);
        let mut paint = Paint::<'_> { anti_alias: true, ..Default::default() };

        paint.shader = LinearGradient::new(
            start,
            end,
            vec![
                GradientStop::new(0.00, tiny_skia::Color::from_rgba(0.0, 0.0, 0.0, 0.0).unwrap()),
                GradientStop::new(1.00, tiny_skia::Color::from_rgba(0.0, 0.0, 0.0, 1.0).unwrap()),
            ],
            SpreadMode::Pad,
            Transform::identity(),
        )
        .unwrap();

        pixmap.fill_rect(
            Rect::from_xywh(0.0, 0.0, width, height).unwrap(),
            &paint,
            Transform::identity(),
            None,
        );
    }

    Image::from_rgba8_premultiplied(pixel_buffer)
}

trait BufData {
    fn data(&self) -> &[u8];
}

impl BufData for SharedImageBuffer {
    #[inline]
    fn data(&self) -> &[u8] {
        match self {
            Self::RGB8(buffer) => buffer.as_bytes(),
            Self::RGBA8(buffer) => buffer.as_bytes(),
            Self::RGBA8Premultiplied(buffer) => buffer.as_bytes(),
        }
    }
}

pub fn pick_color(source_image: Image, x: f32, y: f32) -> SlintColor {
    let x = x as usize;
    let y = y as usize;
    let w = source_image.size().width as usize;
    let h = source_image.size().height as usize;
    if x >= w || y >= h {
        return SlintColor::default();
    }

    let buffer = unsafe {
        let inner = std::mem::transmute::<Image, ImageInner>(source_image);
        inner.render_to_buffer(None)
    };

    if buffer.is_none() {
        SlintColor::default()
    } else {
        let buffer = buffer.unwrap();
        let src = buffer.data();
        let idx = 4 * (y * w + x);

        let r = src[idx];
        let g = src[idx + 1];
        let b = src[idx + 2];
        let a = src[idx + 3];

        SlintColor::from_argb_u8(a, r, g, b)
    }
}
