// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

#[cfg(keyos)]
pub fn main() -> () {
    log_server::init_wait(env!("CARGO_CRATE_NAME")).unwrap();
    log::set_max_level(log::LevelFilter::Info);

    log::info!("Starting");

    // Try to access a privileged CP15 register to cause an UNDEF exception
    unsafe {
        let var = 0;
        core::arch::asm!(
            "mcr p15, 0, {}, c13, c0, 1",
            in(reg) var,
        )
    }
}

#[cfg(not(keyos))]
pub fn main() -> () {}
