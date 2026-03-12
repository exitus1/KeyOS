// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use xous::{MemoryFlags, MemoryMessage, Message};

pub fn main() -> () {
    const NUM_PAGES_SENT: usize = 3;

    log_server::init_wait(env!("CARGO_CRATE_NAME")).unwrap();
    log::set_max_level(log::LevelFilter::Info);

    log::info!("Connecting to the test memory receiver");
    let names = xous_api_names::XousNames::new().unwrap();
    let cid = names.request_connection_blocking("mem-move-test-recv").expect("connect");
    let mut buf = xous::map_memory(None, None, 4096 * NUM_PAGES_SENT, MemoryFlags::W).unwrap();
    let test_str = b"hello world";

    // Fill every page with test data
    for i in 0..NUM_PAGES_SENT {
        let offset = 4096 * i;
        buf.as_slice_mut()[offset..offset + test_str.len()].copy_from_slice(test_str);
    }

    log::info!("Sending buffer...");
    let mem_msg = Message::Move(MemoryMessage { id: 0, buf, valid: None, offset: None });
    xous::send_message(cid, mem_msg).expect("mem move");

    log::info!("Buffer sent, trying accessing moved memory");

    // Prove that the memory move is successful and safe

    #[cfg(keyos)]
    for i in 0..NUM_PAGES_SENT {
        assert!(
            matches!(xous::virt_to_phys(buf.as_ptr() as usize + 4096 * i), Err(xous::Error::BadAddress)),
            "the buffer pages must be unmapped now"
        );
    }

    log::info!("[+] Test successful");
}
