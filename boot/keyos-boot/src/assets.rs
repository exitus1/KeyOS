// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundationdevices.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::verify::Sha256,
    core::sync::atomic::{AtomicUsize, Ordering},
    cosign2::Sha256 as _,
    keyos::BOOT_SPLASH_PHYS_ADDR,
};

include!(concat!(env!("OUT_DIR"), "/assets_metadata.rs"));

// Define the ImageInfo struct to match the C structure
#[repr(C)]
struct ImageInfo {
    filename: *const u8,
    dest: *mut u8,
}

extern "C" {
    fn load_sdcard(image: *const ImageInfo, max_size: u32, show_progress: bool, size: *mut u32) -> i32;
}

/// Holds the last successfully loaded asset to prevent reloading the same asset
static LAST_LOADED_ASSET: AtomicUsize = AtomicUsize::new(0);

pub(crate) fn load_and_verify_asset(asset: &Asset) -> bool {
    let last_loaded_asset = LAST_LOADED_ASSET.load(Ordering::Relaxed);
    if last_loaded_asset != 0 && last_loaded_asset == asset as *const _ as usize {
        return true;
    }

    let image_info = ImageInfo { dest: BOOT_SPLASH_PHYS_ADDR as *mut u8, filename: asset.fs_name.as_ptr() };

    // Load the asset image from the filesystem
    let mut _loaded_size = 0;
    let result = unsafe { load_sdcard(&image_info, asset.size as u32, false, &mut _loaded_size) };

    if result != 0 {
        return false;
    }

    let asset_contents =
        unsafe { core::slice::from_raw_parts(BOOT_SPLASH_PHYS_ADDR as *const u8, asset.size) };

    // Verify the loaded asset
    let hash = Sha256::new().hash(asset_contents);

    // Security: potentially non-constant-time comparison is okay here because the hash is
    // public
    let res = hash == asset.hash;

    if res {
        LAST_LOADED_ASSET.store(asset as *const _ as usize, Ordering::Relaxed);
    }

    res
}
