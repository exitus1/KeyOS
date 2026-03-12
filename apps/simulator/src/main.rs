// SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: MIT OR Apache-2.0

use {
    simulator::{screengrab, settings, theme, MainWindow, SIMULATOR_DIR},
    slint::ComponentHandle,
    std::fs::create_dir_all,
};

gui_server_api::use_api!();

fn main() {
    //slint::platform::set_platform(Box::new(i_slint_backend_winit::Backend::new().unwrap())).unwrap();
    log_server::init_wait(env!("CARGO_CRATE_NAME")).unwrap_or_else(|error| {
        println!("Failed to initialize log server: {:?}", error);
    });

    log::set_max_level(log::LevelFilter::Info);

    create_dir_all(SIMULATOR_DIR).unwrap_or_else(|error| {
        log::warn!("Failed to create simulator directory: {:?}", error);
    });

    // Critical error, nothing can happen without a window
    let window = MainWindow::new().unwrap();

    screengrab::setup(&window);
    settings::setup(&window);
    theme::setup(&window);

    log::info!("Simulator starting");
    window.run().unwrap();
    GuiApiLight::default().shutdown().unwrap();
}
