// SPDX-License-Identifier: GPL-3.0-or-later
//
// Identical shape to the Bitcoin app's build.rs: compile the Slint module tree
// rooted at ui/app.slint, with router + translations.

use slint_keyos_platform_build::{compile_options, CompileOptions};

fn main() {
    compile_options(CompileOptions {
        module_path: "ui/app.slint",
        include_slint: true,
        include_router: true,
        include_translations: true,
        include_time_localization: false,
    });
}
