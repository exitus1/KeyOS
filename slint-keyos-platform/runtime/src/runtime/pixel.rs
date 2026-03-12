// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

#[cfg(keyos)]
use std::arch::arm::{
    uint8x8_t, uint8x8x3_t, uint8x8x4_t, vaddl_u8, vdup_n_u8, vld1_u8, vld3_u8, vld4_u8, vmull_u8, vmvn_u8,
    vqadd_u8, vqaddq_u16, vqmovn_u16, vqsubq_u16, vrshrn_n_u16, vrshrq_n_u16, vst4_u8,
};

use slint::platform::software_renderer::{PremultipliedRgbaColor, TargetPixel};

/// KeyOS-specific color with blue and red channels swapped to match the hardware LCD.
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
#[cfg(keyos)]
pub struct KeyosPixel {
    pub blue: u8,
    pub green: u8,
    pub red: u8,
    pub alpha: u8,
}

/// Regular RGB pixel (hosted mode only)
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
#[cfg(not(keyos))]
pub struct KeyosPixel {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

impl TargetPixel for KeyosPixel {
    fn blend(&mut self, color: PremultipliedRgbaColor) {
        let a = (u8::MAX - color.alpha) as u16;
        self.red = (self.red as u16 * a / 255) as u8 + color.red;
        self.green = (self.green as u16 * a / 255) as u8 + color.green;
        self.blue = (self.blue as u16 * a / 255) as u8 + color.blue;
        self.alpha =
            (self.alpha as u16 + color.alpha as u16 - (self.alpha as u16 * color.alpha as u16) / 255) as u8;
    }

    fn from_rgb(r: u8, g: u8, b: u8) -> Self { Self { red: r, green: g, blue: b, alpha: 255 } }

    fn background() -> Self { Self { red: 0, green: 0, blue: 0, alpha: 0 } }

    #[cfg(keyos)]
    fn blend_slice(slice: &mut [Self], color: PremultipliedRgbaColor) {
        if color.alpha == u8::MAX {
            slice.fill(Self::from_rgb(color.red, color.green, color.blue))
        } else {
            let mut i = 0;
            let len = slice.len() as isize;
            unsafe {
                let cr = vdup_n_u8(color.red);
                let cg = vdup_n_u8(color.green);
                let cb = vdup_n_u8(color.blue);
                let ca = vdup_n_u8(color.alpha);
                let inv_a = vdup_n_u8(255 - color.alpha);
                while i < len - 8 {
                    let uint8x8x4_t(sb, sg, sr, sa) = vld4_u8(slice.as_ptr().offset(i) as *const u8);
                    let rr = vqadd_u8(vrshrn_n_u16(vmull_u8(sr, inv_a), 8), cr);
                    let rg = vqadd_u8(vrshrn_n_u16(vmull_u8(sg, inv_a), 8), cg);
                    let rb = vqadd_u8(vrshrn_n_u16(vmull_u8(sb, inv_a), 8), cb);
                    let ra = calculate_alpha(sa, ca);
                    vst4_u8(slice.as_mut_ptr().offset(i) as *mut u8, uint8x8x4_t(rb, rg, rr, ra));
                    i += 8;
                }
            }

            while i < len {
                slice[i as usize].blend(color);
                i += 1;
            }
        }
    }

    #[cfg(keyos)]
    fn blend_texture_slice_rgba(slice: &mut [Self], color: &[PremultipliedRgbaColor]) {
        let len = slice.len().min(color.len()) as isize;
        let mut i = 0;
        while i < len - 8 {
            unsafe {
                let uint8x8x4_t(sb, sg, sr, sa) = vld4_u8(slice.as_ptr().offset(i) as *const u8);
                let uint8x8x4_t(cr, cg, cb, ca) = vld4_u8(color.as_ptr().offset(i) as *const u8);
                let inv_a = vmvn_u8(ca);
                let rr = vqadd_u8(vrshrn_n_u16(vmull_u8(sr, inv_a), 8), cr);
                let rg = vqadd_u8(vrshrn_n_u16(vmull_u8(sg, inv_a), 8), cg);
                let rb = vqadd_u8(vrshrn_n_u16(vmull_u8(sb, inv_a), 8), cb);
                let ra = calculate_alpha(sa, ca);
                vst4_u8(slice.as_mut_ptr().offset(i) as *mut u8, uint8x8x4_t(rb, rg, rr, ra));
                i += 8;
            }
        }

        while i < len {
            slice[i as usize].blend(color[i as usize]);
            i += 1;
        }
    }

    #[cfg(keyos)]
    fn blend_texture_slice_rgb(slice: &mut [Self], color: &[slint::Rgb8Pixel]) {
        let len = slice.len().min(color.len()) as isize;
        let mut i = 0;
        unsafe {
            let ca = vdup_n_u8(255);
            while i < len - 9 {
                let uint8x8x3_t(cr, cg, cb) = vld3_u8(color.as_ptr().offset(i) as *const u8);
                vst4_u8(slice.as_mut_ptr().offset(i) as *mut u8, uint8x8x4_t(cb, cg, cr, ca));
                i += 8;
            }
        }

        while i < len {
            slice[i as usize] = Self::from_rgb(color[i as usize].r, color[i as usize].g, color[i as usize].b);
            i += 1;
        }
    }

    #[cfg(keyos)]
    fn blend_texture_slice_alpha(slice: &mut [Self], color: slint::Rgb8Pixel, alpha: &[u8]) {
        let len = slice.len().min(alpha.len()) as isize;
        let mut i = 0;
        unsafe {
            let cr = vdup_n_u8(color.r);
            let cg = vdup_n_u8(color.g);
            let cb = vdup_n_u8(color.b);
            while i < len - 8 {
                let uint8x8x4_t(sb, sg, sr, sa) = vld4_u8(slice.as_ptr().offset(i) as *const u8);
                let ca = vld1_u8(&alpha[i as usize] as *const u8);
                let inv_a = vmvn_u8(ca);
                let rr = vrshrn_n_u16(vqaddq_u16(vmull_u8(sr, inv_a), vmull_u8(cr, ca)), 8);
                let rg = vrshrn_n_u16(vqaddq_u16(vmull_u8(sg, inv_a), vmull_u8(cg, ca)), 8);
                let rb = vrshrn_n_u16(vqaddq_u16(vmull_u8(sb, inv_a), vmull_u8(cb, ca)), 8);
                let ra = calculate_alpha(sa, ca);
                vst4_u8(slice.as_mut_ptr().offset(i) as *mut u8, uint8x8x4_t(rb, rg, rr, ra));
                i += 8;
            }
        }

        while i < len {
            let c = PremultipliedRgbaColor::from(slint::Color::from_argb_u8(
                alpha[i as usize],
                color.r,
                color.g,
                color.b,
            ));
            slice[i as usize].blend(c);
            i += 1;
        }
    }
}

#[cfg(keyos)]
#[inline(always)]
fn calculate_alpha(sa: uint8x8_t, ca: uint8x8_t) -> uint8x8_t {
    unsafe { vqmovn_u16(vqsubq_u16(vaddl_u8(sa, ca), vrshrq_n_u16(vmull_u8(sa, ca), 8))) }
}
