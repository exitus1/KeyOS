// SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: MIT OR Apache-2.0

use gui_server::Gui;

#[cfg(keyos)]
fn main() {
    log_server::init_wait(env!("CARGO_CRATE_NAME")).unwrap();
    log::set_max_level(log::LevelFilter::Info);

    xous::set_thread_priority(xous::ThreadPriority::System3).unwrap();

    server::listen(Gui::new().expect("initialize gui server"))
}

#[cfg(not(keyos))]
fn main() {
    log_server::init_wait(env!("CARGO_CRATE_NAME")).unwrap();
    log::set_max_level(log::LevelFilter::Info);

    std::thread::Builder::new()
        .name("KeyOS GUI thread".to_string())
        .spawn(move || server::listen(Gui::new().expect("initialize gui server")))
        .expect("Spawn gui thread");

    gui_server::open_window()
}
