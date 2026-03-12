// SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::mem;

const STACK_SIZE: usize = 1024 * 1024 * 32;

fn f() -> u8 {
    let arr = [0xAA; STACK_SIZE];
    println!("Arr size: {}", mem::size_of_val(&arr));
    arr.iter().sum()
}

pub fn main() -> () {
    log_server::init_wait(env!("CARGO_CRATE_NAME")).unwrap();
    log::set_max_level(log::LevelFilter::Info);

    log::info!("Sum: {}", f());

    loop {
        xous::yield_slice();
    }
}
