// SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundationdevices.com>
// SPDX-License-Identifier: MIT OR Apache-2.0

#[cfg(keyos)]
fn main() {
    use power_manager::api::PowerManagerApi;

    log_server::init_wait(env!("CARGO_CRATE_NAME")).unwrap();
    log::set_max_level(log::LevelFilter::Info);

    log::info!("RUNNING SECURE ELEMENT DEMO");

    let pmc = PowerManagerApi::new();
    pmc.enable_peripheral(atsama5d27::pmc::PeripheralId::Pioa).unwrap();
    pmc.enable_peripheral(atsama5d27::pmc::PeripheralId::Piob).unwrap();
    pmc.enable_peripheral(atsama5d27::pmc::PeripheralId::Pioc).unwrap();
    pmc.enable_peripheral(atsama5d27::pmc::PeripheralId::Piod).unwrap();
    pmc.enable_peripheral(atsama5d27::pmc::PeripheralId::Flexcom2).unwrap();
    pmc.enable_peripheral(atsama5d27::pmc::PeripheralId::Spi0).unwrap();
    pmc.enable_peripheral(atsama5d27::pmc::PeripheralId::Pit).unwrap();

    cryptoauthlib::init().unwrap();
    for _ in 0..3 {
        let info = cryptoauthlib::device_info().unwrap();
        log::info!("Device info: {:?}", info);
    }
    for _ in 0..3 {
        if cryptoauthlib::self_test().unwrap() {
            log::info!("Self test passed");
        } else {
            log::error!("Self test failed");
        }
    }

    log::info!("SECURE ELEMENT SUCCESS");
}

#[cfg(not(keyos))]
fn main() {}
