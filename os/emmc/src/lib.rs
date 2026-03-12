// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

#![cfg(keyos)]
use xous::keyos::TOTAL_FLASH_BLOCKS;

pub const BLOCK_SIZE: usize = 512;

pub mod api;
mod encryption;
pub mod error;
mod implementation;
pub mod messages;

crypto::use_api!();

const SD_BUFFER_BLOCKS: usize = 64;
const _: () = {
    if (SD_BUFFER_BLOCKS * BLOCK_SIZE) % 4096 != 0 {
        panic!("SD buffer size must be divisible by page size")
    }
};

pub fn listen() { server::listen(implementation::EmmcServer::new().unwrap()) }
