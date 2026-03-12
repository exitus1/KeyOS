// SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

pub fn main() -> () {
    log_server::init_wait(env!("CARGO_CRATE_NAME")).unwrap();
    log::set_max_level(log::LevelFilter::Info);

    for _ in 0..10 {
        xous::yield_slice();
    }

    for _i in 0.. {
        // log::info!("Loop #{}", _i);

        for _ in 0..100_000 {
            core::hint::black_box(()); // nop
        }
    }
}
