// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: MIT OR Apache-2.0

pub mod screengrab;
pub mod settings;
pub mod theme;
slint::include_modules!();

gui_server_api::use_api!();

pub const SIMULATOR_DIR: &str = "../../simulator-files";
pub const SETTINGS_FILE: &str = "../../simulator-files/settings.json";
pub const DEP_SETTINGS_FILE: &str = "../../simulator-files/deprecated_settings.json";
const SCREENSHOTS_DIR: &str = "screenshots";
const GIF_DELAY_MS: u32 = 80;
