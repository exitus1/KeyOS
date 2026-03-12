// SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

pub fn main() -> () {
    log_server::init_wait(env!("CARGO_CRATE_NAME")).unwrap();
    log::set_max_level(log::LevelFilter::Info);

    unsafe {
        let r0: u32 = 0xDEADBEEF;
        core::arch::asm!(
            "mov r0, {}",
            "svc 0",
            in(reg) r0,
        )
    }
}
