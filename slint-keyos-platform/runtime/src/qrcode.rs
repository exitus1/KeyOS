// SPDX-FileCopyrightText: © 2023 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use core::cell::RefCell;

use slint::Model;
use slint::ModelRc;

const BORDER_RADIUS: f32 = 24.0;
const QUIET_ZONE: u32 = 2;

pub fn render(data: impl AsRef<[u8]>, dark_color: slint::Color, light_color: slint::Color) -> slint::Image {
    let code = qrcode::QrCode::new(data).expect("Failed to render QR Code");

    let colors = code.to_colors();
    let width = code.width();

    let mut renderer = qrcode::render::Renderer::<QrCodePixel>::new(&colors, width, QUIET_ZONE);

    renderer.dark_color(dark_color.into()).light_color(light_color.into()).build()
}

struct QrCodeCanvas {
    pixel_buffer: slint::SharedPixelBuffer<slint::Rgba8Pixel>,
    dark_pixel: slint::Rgba8Pixel,
    border_radius: f32,
}

impl qrcode::render::Canvas for QrCodeCanvas {
    type Image = slint::Image;
    type Pixel = QrCodePixel;

    fn new(width: u32, height: u32, dark_pixel: Self::Pixel, light_pixel: Self::Pixel) -> Self {
        let mut pixel_buffer = slint::SharedPixelBuffer::new(width, height);

        fill_rgba(&mut pixel_buffer, light_pixel.0);

        Self { pixel_buffer, dark_pixel: dark_pixel.0, border_radius: BORDER_RADIUS }
    }

    fn draw_dark_pixel(&mut self, x: u32, y: u32) {
        // These unwraps should not fail as they should not be bigger than the
        // width or height, and if they do it is a logic error.
        //
        // The Self::new function also checks when filling if width and height
        // fit on an usize.
        let x = usize::try_from(x).unwrap();
        let y = usize::try_from(y).unwrap();

        let width = usize::try_from(self.pixel_buffer.width()).unwrap();

        let slice = self.pixel_buffer.make_mut_slice();
        slice[(y * width) + x] = self.dark_pixel;
    }

    fn into_image(self) -> Self::Image {
        if self.border_radius > 0.0 {
            apply_rounded_corners(self.pixel_buffer, self.border_radius)
        } else {
            slint::Image::from_rgba8(self.pixel_buffer)
        }
    }
}

fn apply_rounded_corners(
    mut buffer: slint::SharedPixelBuffer<slint::Rgba8Pixel>,
    radius: f32,
) -> slint::Image {
    let width = buffer.width();
    let height = buffer.height();

    let mut mask = tiny_skia::Pixmap::new(width, height).expect("Failed to create mask pixmap");

    if let Some(rect) = tiny_skia::Rect::from_xywh(0.0, 0.0, width as f32, height as f32) {
        if let Some(path) = build_rounded_rect(&rect, radius) {
            mask.fill_path(
                &path,
                &tiny_skia::Paint {
                    shader: tiny_skia::Shader::SolidColor(tiny_skia::Color::WHITE),
                    ..Default::default()
                },
                tiny_skia::FillRule::Winding,
                tiny_skia::Transform::identity(),
                None,
            );
        }
    }

    let mask_pixels = mask.pixels();
    let buffer_slice = buffer.make_mut_slice();

    for i in 0..buffer_slice.len() {
        let mask_alpha = mask_pixels[i].alpha();
        let current_alpha = buffer_slice[i].a;
        buffer_slice[i].a = ((current_alpha as u16 * mask_alpha as u16) / 255) as u8;
    }

    slint::Image::from_rgba8(buffer)
}

fn build_rounded_rect(d: &tiny_skia::Rect, radius: f32) -> Option<tiny_skia::Path> {
    let mut pb = tiny_skia::PathBuilder::new();

    // Approximating a quarter circle with cubic bezier
    // See https://pomax.github.io/bezierinfo/#circles_cubic
    let b = (1.0 - 0.551785) * radius;

    pb.move_to(d.x() + radius, d.y()); // top
    pb.line_to(d.x() + d.width() - radius, d.y());
    pb.cubic_to(
        // top-right
        d.x() + d.width() - b,
        d.y(),
        d.x() + d.width(),
        d.y() + b,
        d.x() + d.width(),
        d.y() + radius,
    );
    pb.line_to(d.x() + d.width(), d.y() + d.height() - radius); // right
    pb.cubic_to(
        //  bottom-right
        d.x() + d.width(),
        d.y() + d.height() - b,
        d.x() + d.width() - b,
        d.y() + d.height(),
        d.x() + d.width() - radius,
        d.y() + d.height(),
    );
    pb.line_to(d.x() + radius, d.y() + d.height()); // bottom
    pb.cubic_to(
        //  bottom-left
        d.x() + b,
        d.y() + d.height(),
        d.x(),
        d.y() + d.height() - b,
        d.x(),
        d.y() + d.height() - radius,
    );
    pb.line_to(d.x(), d.y() + radius); // left
    pb.cubic_to(
        //  top-left
        d.x(),
        d.y() + b,
        d.x() + b,
        d.y(),
        d.x() + radius,
        d.y(),
    );
    pb.close();
    pb.finish()
}

#[derive(Clone, Copy)]
struct QrCodePixel(slint::Rgba8Pixel);

impl From<slint::Color> for QrCodePixel {
    fn from(c: slint::Color) -> Self { Self(slint::Rgba8Pixel::new(c.red(), c.green(), c.blue(), 255)) }
}

impl qrcode::render::Pixel for QrCodePixel {
    type Canvas = QrCodeCanvas;
    type Image = slint::Image;

    fn default_color(color: qrcode::types::Color) -> Self {
        Self(match color {
            qrcode::types::Color::Light => slint::Rgba8Pixel::new(255, 255, 255, 255),
            qrcode::types::Color::Dark => slint::Rgba8Pixel::new(0, 0, 0, 255),
        })
    }
}

fn fill_rgba(buf: &mut slint::SharedPixelBuffer<slint::Rgba8Pixel>, pixel: slint::Rgba8Pixel) {
    let width = usize::try_from(buf.width()).expect("image is too big for the platform");
    let height = usize::try_from(buf.height()).expect("image is too big for the platform");

    let slice = buf.make_mut_slice();

    for y in 0..height {
        for x in 0..width {
            slice[(y * width) + x] = pixel;
        }
    }
}

/// Encodes the data into a series of foundation_ur parts.
/// The provided model is an infinite iterator, so do not try to collect the values
pub fn encode_qr_parts(
    ur_type: impl AsRef<str>,
    data: impl AsRef<[u8]>,
    max_size: i32,
) -> ModelRc<slint::SharedString> {
    ModelRc::new(QrCodeParts::new(ur_type.as_ref().to_string(), data.as_ref().to_vec(), max_size))
}

struct QrCodeParts {
    encoder: RefCell<foundation_ur::Encoder<'static, 'static>>,
    ur_data: &'static [u8],
    ur_type: &'static str,
}

impl QrCodeParts {
    fn new(ur_type: String, data: Vec<u8>, max_size: i32) -> Self {
        let max_sequence_number = 100;
        let fragment_len = foundation_ur::max_fragment_len(
            ur_type.as_str(),
            max_sequence_number,
            max_size.try_into().unwrap_or(100),
        );

        // Leak the owned strings to obtain 'static references required by the encoder.
        // The memory is reclaimed in `Drop` where we turn the raw pointers back into `Box`es.
        let ur_type_box = ur_type.into_boxed_str();
        let data_box = data.into_boxed_slice();
        let ur_type_ref: &'static str = Box::leak(ur_type_box);
        let data_ref: &'static [u8] = Box::leak(data_box);

        let mut encoder: foundation_ur::Encoder<'static, 'static> = foundation_ur::Encoder::new();
        encoder.start(ur_type_ref, data_ref, fragment_len);

        Self { ur_data: data_ref, ur_type: ur_type_ref, encoder: RefCell::new(encoder) }
    }
}

impl Drop for QrCodeParts {
    fn drop(&mut self) {
        // safety: only the encoder is using the data and type
        unsafe {
            let _ = Box::from_raw(self.ur_data as *const [u8] as *mut [u8]);
            let _ = Box::from_raw(self.ur_type as *const str as *mut str);
        }
    }
}

impl Model for QrCodeParts {
    type Data = slint::SharedString;

    fn row_count(&self) -> usize { 100 }

    // always generates next part, ignore row index
    fn row_data(&self, _row: usize) -> Option<Self::Data> {
        let mut encoder = self.encoder.borrow_mut();
        let part = encoder.next_part().to_string().to_uppercase();
        Some(slint::SharedString::from(part))
    }

    fn model_tracker(&self) -> &dyn slint::ModelTracker {
        // no changes allowed.
        &()
    }
}

#[test]
fn test_encode_qr_parts() {
    use slint::Model;

    fn decode(label: &'static str, parts: &mut impl Iterator<Item = slint::SharedString>, data: &str) {
        let mut decoder = foundation_ur::Decoder::default();

        while !decoder.is_complete() {
            let part = parts.next().unwrap().to_lowercase();
            let ur = foundation_ur::UR::parse(&part).unwrap();
            decoder.receive(ur).unwrap();
        }

        let message = String::from_utf8(decoder.message().ok().flatten().unwrap().to_vec()).unwrap();

        assert_eq!(message, data, "{}", label);
    }

    let data = "Lorem ipsum dolor sit amet, consectetur adipiscing elit.
        Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.
        Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.
        Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur.
        Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";

    let parts = encode_qr_parts("bytes", data, 1000);
    let mut iter = parts.iter();

    decode("first", &mut iter, data);
    decode("second", &mut iter, data);

    iter.next();
    decode("third", &mut iter, data);
}
