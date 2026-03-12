// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use core::sync::atomic::{AtomicU32, Ordering};

/// Stack protection canary
#[no_mangle]
pub static __stack_chk_guard: AtomicU32 = AtomicU32::new(0);

/// Called by compiler-generated epilogues on mismatch.
#[no_mangle]
pub extern "C" fn __stack_chk_fail() -> ! {
    panic!("stack overflow detected");
}

#[no_mangle]
pub extern "C" fn __stack_chk_fail_local() -> ! { __stack_chk_fail() }

pub fn set_stack_guard(guard: u32) {
    let guard = guard & 0xffff_ff00; // Include at least one 0 byte
    __stack_chk_guard.store(guard, Ordering::Relaxed);
}
