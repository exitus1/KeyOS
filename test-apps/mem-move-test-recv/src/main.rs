// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use {xous::MessageEnvelope, xous_api_names::XousNames};

pub fn main() -> () {
    log_server::init_wait(env!("CARGO_CRATE_NAME")).unwrap();
    log::set_max_level(log::LevelFilter::Info);

    log::info!("Starting");

    let names = XousNames::new().expect("names");
    let sid = xous::create_server().expect("creating server");
    names.register_name(sid, "mem-move-test-recv").expect("register name");

    if let MessageEnvelope { body: xous::Message::Move(mem_msg), sender } =
        &xous::receive_message(sid).unwrap()
    {
        let len = mem_msg.buf.len();
        log::info!(
            "Received {} bytes of moved memory with id {} from PID {}",
            len,
            mem_msg.id,
            sender.pid().unwrap()
        );

        let buf_bytes = mem_msg.buf.as_slice::<u8>();
        let test_str = b"hello world";
        for offset in (0..len).step_by(4096) {
            assert_eq!(&buf_bytes[offset..offset + test_str.len()], test_str, "must contain expected data");
        }
    }

    log::info!("[+] Test successful");
}
