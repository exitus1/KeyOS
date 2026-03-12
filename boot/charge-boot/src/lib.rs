// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundationdevices.com>
// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]

mod batt;
mod entrypoint;
mod flash_erase;
mod gui;
mod main_screen;
mod rgb;

#[no_mangle]
pub extern "C" fn ffi_bootloader_entrypoint() { entrypoint::entrypoint(); }

#[no_mangle]
pub extern "C" fn ffi_set_progress_bar(_curr: u32, _total: u32) {}
