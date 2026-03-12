// SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::io::{Read, Write};

use security::{FirmwareTimestamp, PinEntryMode, Seed};

bt::use_api!();
fs::use_api!();
security::use_api!();

fn main() {
    log_server::init_wait(env!("CARGO_CRATE_NAME")).unwrap();
    log::set_max_level(log::LevelFilter::Info);

    log::info!("Waiting for bluetooth to boot to get device ID");
    let mut bt_api = BluetoothApi::default();
    while !bt_api.state().unwrap().is_booted() {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    log::info!("Bluetooth booted");
    let security = Security::default();

    let device_id = security.device_id().unwrap();
    log::info!("device id: {device_id:?}");

    let firmware_timestamp = security.firmware_timestamp();
    log::info!("firmware timestamp: {:?}", firmware_timestamp);

    // Test PIN validation: this should fail with a short PIN
    log::info!("Testing PIN validation with short PIN (4 chars)...");
    match security.set_seed_and_pin(Seed::Twelve([11; 16]), "1234".to_string(), PinEntryMode::Pin) {
        Ok(_) => log::error!("Short PIN was unexpectedly accepted!"),
        Err(security::PinError::TooShort) => log::info!("Short PIN correctly rejected: PIN too short"),
        Err(e) => log::info!("Short PIN rejected with different error: {}", e),
    }

    // Now use a valid PIN (6+ characters)
    log::info!("Setting seed and PIN with valid 6-character PIN...");
    security.set_seed_and_pin(Seed::Twelve([11; 16]), "123456".to_string(), PinEntryMode::Pin).unwrap();

    let seed_final = security.seed_fingerprint().unwrap();
    log::info!("seed fingerprint: {:?}", seed_final);

    let is_pin_set = security.is_pin_set().unwrap();
    log::info!("pin is set after setting pin: {is_pin_set:?}");

    let msg: [u8; 32] = [
        0xb5, 0x58, 0xf1, 0xe8, 0x44, 0x98, 0xc4, 0xdf, 0x7a, 0xfa, 0xa4, 0xaa, 0x46, 0x06, 0x04, 0x96, 0x2e,
        0x5f, 0x9a, 0x0f, 0x8b, 0x37, 0x00, 0xdc, 0x41, 0x6f, 0x8b, 0x3b, 0x3b, 0xbf, 0xb8, 0x4f,
    ];
    let mac = security.keycard_authenticity_mac(msg).unwrap();
    // Calculated based on predefined message and key values.
    let expected_mac: [u8; 32] = [
        0xda, 0x99, 0x96, 0x0f, 0xe9, 0x2e, 0x75, 0x90, 0xb8, 0x65, 0x86, 0xd1, 0xed, 0xe8, 0xd8, 0x73, 0x27,
        0xa8, 0x92, 0x6e, 0x84, 0xf8, 0x00, 0xcb, 0x8f, 0xcb, 0x15, 0x48, 0x2c, 0x1d, 0x23, 0xe9,
    ];
    if mac == expected_mac {
        log::info!("keycard authenticity MAC matches expected value");
    } else {
        log::info!("keycard authenticity MAC does not match expected value, got result {mac:?}");
    }

    let result = security.log_in("123456".to_string());
    log::info!("attempted login with correct pin, got result {result:?}");

    let logged_in = security.logged_in();
    log::info!("logged in: {logged_in:?}");

    // Test change_pin with short PIN (should fail)
    log::info!("Testing change_pin with short PIN (4 chars)...");
    match security.change_pin("4321".to_string(), Some(Seed::Twelve([11; 16])), PinEntryMode::Pin) {
        Ok(_) => log::error!("Short PIN change was unexpectedly accepted!"),
        Err(security::PinError::TooShort) => log::info!("Short PIN change correctly rejected: PIN too short"),
        Err(e) => log::info!("Short PIN change rejected with different error: {}", e),
    }

    // Now change to a valid PIN
    log::info!("Changing to valid 6-character PIN...");
    security.change_pin("654321".to_string(), Some(Seed::Twelve([11; 16])), PinEntryMode::Pin).unwrap();

    let logged_in = security.logged_in();
    log::info!("logged in after pin change: {logged_in:?}");

    let is_pin_set = security.is_pin_set().unwrap();
    log::info!("pin is set after pin change: {is_pin_set:?}");

    security.log_out();

    let result = security.log_in("123456".to_string());
    log::info!("attempted login with old (unchanged => incorrect) pin, got result {result:?}");

    let result = security.log_in("654321".to_string());
    log::info!("attempted login with new (changed => correct) pin, got result {result:?}");

    let logged_in = security.logged_in();
    log::info!("logged in: {logged_in:?}");

    let seed = security.seed().unwrap();
    log::info!("seed: {:?}", seed);

    let device_id = security.device_id().unwrap();
    log::info!("device id: {device_id:?}");

    security.set_firmware_timestamp(FirmwareTimestamp([3; 4])).unwrap();
    let firmware_timestamp = security.firmware_timestamp().unwrap();
    log::info!("firmware timestamp after update: {:?}", firmware_timestamp.0);

    security.set_seed_and_pin(Seed::TwentyFour([0; 32]), "123456".to_string(), PinEntryMode::Pin).unwrap();

    let result = security.log_in("654321".to_string());
    log::info!("attempted login with old incorrect pin, got result {result:?}");

    let result = security.log_in("123456".to_string());
    log::info!("attempted login with new correct pin, got result {result:?}");

    let seed = security.seed().unwrap();
    log::info!("seed before change: {:?}", seed);

    security.set_seed(Seed::TwentyFour([12; 32])).unwrap();
    let seed = security.seed().unwrap();
    log::info!("seed after change: {:?}", seed);

    let words = security.security_words("123").unwrap();
    log::info!("security words: {} {}", words[0], words[1]);

    let factory_reset_counter = security.factory_reset_counter();
    log::info!("factory reset counter: {factory_reset_counter:?}");

    let sig = security.sign_with_security_check_key([12; 32]).unwrap();
    log::info!("signed with security check key: {sig:?}");

    let sig = security.sign_with_fido_key([12; 32]).unwrap();
    log::info!("signed with fido key: {sig:?}");

    security.log_out();
    let result = security.seed().is_ok();
    log::info!("seed after logout is OK: {result:?}");

    security.set_firmware_timestamp(FirmwareTimestamp([103; 4])).unwrap();
    let firmware_timestamp = security.firmware_timestamp().unwrap();
    log::info!("should be able to set firmware timestamp after logout: {firmware_timestamp:?}");

    if should_reset() {
        log::info!("performing factory reset");
        security.lockout(security::LockoutOptions::erase_aes_keys()).unwrap();
    }
}

/// Alternate between resetting and not resetting by using the file system.
fn should_reset() -> bool {
    let fs = FileSystem::default();
    let mut reset_file = fs
        .open_file(
            "/keyos/reset_test.txt",
            fs::Location::System,
            fs::OpenFlags { read: true, write: true, create: true },
        )
        .unwrap();
    let mut buf = String::new();
    reset_file.read_to_string(&mut buf).unwrap();
    if buf.is_empty() {
        // File just created, write something in it and reset.
        reset_file.write_all("reset".as_bytes()).unwrap();
        true
    } else {
        // File already created, remove it and do not reset.
        assert_eq!(buf, "reset");
        drop(reset_file);
        fs.remove("/keyos/reset_test.txt", fs::Location::System).unwrap();
        false
    }
}
