// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundationdevices.com>
// SPDX-License-Identifier: GPL-3.0-or-later

/// Secure RAM (`SECURAM`) driver routines.

pub const HW_SECURAM_BASE: usize = 0xF804_4000;

/// Lower 4KB, auto-erasable.
const OFFSET_BUSRAM_LOWER: usize = 0x0000;

/// Size in bytes of the lower 4KB of `SECURAM`.
pub const SIZE_BUSRAM_LOWER: usize = 1024 * 4;

/// Higher 1KB not auto-erased.
const OFFSET_BUSRAM_HIGHER: usize = 0x1000;

/// Size in bytes of the higher 1KB of `SECURAM`.
pub const SIZE_BUSRAM_HIGHER: usize = 1024;

/// `BUREG` 256 bits auto-erased
const OFFSET_BUREG: usize = 0x1400;

/// Size in bytes of `BUREG`.
const SIZE_BUREG: usize = 256 / 8;

pub struct Securam {
    base_addr: u32,
}

impl Default for Securam {
    fn default() -> Self {
        Securam::new()
    }
}

impl Securam {
    #[inline]
    pub fn new() -> Self {
        Self {
            base_addr: HW_SECURAM_BASE as u32,
        }
    }

    #[inline]
    pub fn with_alt_base_addr(base_addr: u32) -> Self {
        Self { base_addr }
    }

    /// Returns the lower 4KB of `SECURAM`.
    #[inline]
    pub fn lower(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                (self.base_addr + OFFSET_BUSRAM_LOWER as u32) as *const u8,
                SIZE_BUSRAM_LOWER,
            )
        }
    }

    /// Returns the lower 4KB of `SECURAM`.
    #[inline]
    pub fn lower_mut(&mut self) -> &mut [u8] {
        unsafe {
            core::slice::from_raw_parts_mut(
                (self.base_addr + OFFSET_BUSRAM_LOWER as u32) as *mut u8,
                SIZE_BUSRAM_LOWER,
            )
        }
    }

    /// Returns the higher 1KB of `SECURAM`.
    #[inline]
    pub fn higher(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                (self.base_addr + OFFSET_BUSRAM_HIGHER as u32) as *const u8,
                SIZE_BUSRAM_HIGHER,
            )
        }
    }

    /// Returns the higher 1KB of `SECURAM`.
    #[inline]
    pub fn higher_mut(&mut self) -> &mut [u8] {
        unsafe {
            core::slice::from_raw_parts_mut(
                (self.base_addr + OFFSET_BUSRAM_HIGHER as u32) as *mut u8,
                SIZE_BUSRAM_HIGHER,
            )
        }
    }

    /// Returns the `BUREG` 256 bits of `SECURAM`.
    #[inline]
    pub fn bureg(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                (self.base_addr + OFFSET_BUREG as u32) as *const u8,
                SIZE_BUREG,
            )
        }
    }

    /// Returns the `BUREG` 256 bits of `SECURAM`.
    #[inline]
    pub fn bureg_mut(&mut self) -> &mut [u8] {
        unsafe {
            core::slice::from_raw_parts_mut(
                (self.base_addr + OFFSET_BUREG as u32) as *mut u8,
                SIZE_BUREG,
            )
        }
    }
}
