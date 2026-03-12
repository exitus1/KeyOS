// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use slint_keyos_platform_common::utils;

fn main() {
    let config = slint_build::CompilerConfiguration::new()
        .with_style("fluent-light".to_string())
        .with_library_paths(utils::library_paths());
    slint_build::compile_with_config("ui/slintbook.slint", config).unwrap();
}
