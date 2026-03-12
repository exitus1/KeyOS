// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

#[cfg(keyos)]
use {
    emmc::{api::EmmcApi, BLOCK_SIZE},
    std::{thread, time::Duration},
};

#[cfg(keyos)]
pub fn main() -> () {
    const UNUSED_BLOCK_IDX: u32 = 0x7a50;
    log_server::init_wait(env!("CARGO_CRATE_NAME")).unwrap();
    log::set_max_level(log::LevelFilter::Info);

    log::info!("Starting");

    thread::sleep(Duration::from_secs(1));

    let mut buf = [0; BLOCK_SIZE];
    let mut emmc_api = EmmcApi::default();
    emmc_api.read_blocks(0, &mut buf).expect("read eMMC block");

    // Print the buffer
    log::info!("Read buffer:");
    for line in buf.chunks_exact(32) {
        log::info!("{:02x?}", line);
    }

    buf.fill(0);
    let str = b"hello world";
    buf[..str.len()].copy_from_slice(str);

    log::info!("Write buffer:");
    for line in buf.chunks_exact(32) {
        log::info!("{:02x?}", line);
    }

    emmc_api.write_blocks(UNUSED_BLOCK_IDX, &buf).expect("write block");
    let mut buf2 = [0; BLOCK_SIZE];
    emmc_api.read_blocks(UNUSED_BLOCK_IDX, &mut buf2).expect("read block");
    log::info!("Read buffer 2:");
    for line in buf2.chunks_exact(32) {
        log::info!("{:02x?}", line);
    }

    assert_eq!(buf, buf2, "must read back the same content that was written");

    log::info!("Test successful");

    // Do nothing
    loop {
        xous::yield_slice();
    }
}

#[cfg(not(keyos))]
pub fn main() -> () {}
