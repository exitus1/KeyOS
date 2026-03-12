// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundationdevices.com>
// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]

use {atsama5d27::dma::BIG_TRANSFER_CHUNK_SIZE, boot_common::PB_OVERLAY_DMA_ADDR};

mod assets;
mod batt;
mod boot_screen;
mod entrypoint;
mod gui;
mod memzero;
mod menu_page;
mod progress_bar;
mod securam;
mod splash;
mod system_error_page;
mod system_errors;
mod verify;

pub static mut PROGRESS_BAR: Option<progress_bar::ProgressBar> = None;

pub(crate) const FIRMWARE_LOAD_BASE_ADDR: usize = 0x20000000;

// Entrypoint of the loader binary and its reset vectors
const FIRMWARE_JUMP_ADDR: usize = FIRMWARE_LOAD_BASE_ADDR + cosign2::Header::DEFAULT_SIZE;

// This is the split point for zeroing memory, we start by zeroing everything that will be
// used for the display, and after initing display, we zero the rest.
// It has to be well-aligned because of how DMA transfers work for big ranges
const DISPLAY_MEMORY_START: usize = PB_OVERLAY_DMA_ADDR & !(BIG_TRANSFER_CHUNK_SIZE * 4 - 1);

// This field is replaced with a secret value when building the bootloader
static EXTRA_ENTROPY: [u8; 32] = *b"extra_entropy_replaced_by_xtask_";
// Before jumping out from the bootloader, the EXTRA_ENTROPY field is replaced with
// this one to hide the actual value from subsequent stages.
static DEFAULT_EXTRA_ENTROPY: [u8; 32] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x19, 0xd6, 0x68, 0x9c, 0x08, 0x5a, 0xe1, 0x65, 0x83, 0x1e, 0x93, 0x4f,
    0xf7, 0x63, 0xae, 0x46, 0xa2, 0xa6, 0xc1, 0x72, 0xb3, 0xf1, 0xb6, 0x0a, 0x8c, 0xe2, 0x6f,
];

#[repr(C)]
#[derive(Copy, Clone)]
pub enum BrickMessage {
    MainImageUnverified = 0,
    RecoveryImageUnverified,
}

#[no_mangle]
pub extern "C" fn ffi_bootloader_entrypoint() { entrypoint::entrypoint(); }

#[no_mangle]
pub extern "C" fn ffi_set_progress_bar(curr: u32, total: u32) {
    if let Some(pb) = unsafe { (*core::ptr::addr_of_mut!(PROGRESS_BAR)).as_mut() } {
        let percent = curr as u64 * 100 / total as u64;
        pb.set_percent(percent as u32);
    }
}

#[derive(Copy, Clone)]
pub(crate) enum BootImageKind {
    Main,
    UpdatedMain,
    Recovery,
}

static mut BOOT_IMAGE_KIND: BootImageKind = BootImageKind::Main;

pub(crate) fn selected_boot_image_kind() -> BootImageKind { unsafe { BOOT_IMAGE_KIND } }

pub(crate) fn select_updated_image() {
    unsafe {
        BOOT_IMAGE_KIND = BootImageKind::UpdatedMain;
    }
}

pub(crate) fn select_recovery_image() {
    unsafe {
        BOOT_IMAGE_KIND = BootImageKind::Recovery;
    }
}

/// Hardened equality check that prevents compiler optimization of redundant comparisons
#[inline(always)]
pub(crate) fn hardened_eq<TA, TB>(a: TA, b: TB) -> bool
where
    TA: core::cmp::PartialEq<TB>,
{
    // First check with black_box on left operand
    if !(core::hint::black_box(&a) == &b) {
        return false;
    }

    boot_common::random::delay();

    // Second check with black_box on right operand
    if !(a == core::hint::black_box(b)) {
        return false;
    }

    true
}
