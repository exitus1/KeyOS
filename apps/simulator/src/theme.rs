// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: MIT OR Apache-2.0

use slint::ComponentHandle;

use crate::MainWindow;

settings::use_api!();

/// Set the system theme
pub fn set_system_theme(is_dark: bool) {
    let api = SettingsApi::default();
    let theme =
        if is_dark { settings::global::SystemTheme::Dark } else { settings::global::SystemTheme::Light };
    api.set_system_theme(theme);
}

/// Get the current system theme
pub fn get_system_theme() -> bool {
    let api = SettingsApi::default();
    matches!(api.get_system_theme(), settings::global::SystemTheme::Dark)
}

pub fn setup(window: &MainWindow) {
    // Initialize theme state
    let is_dark = get_system_theme();
    window.set_is_dark_theme(is_dark);

    // Subscribe to theme changes
    setup_theme_subscription(window);

    window.on_theme_set(move |is_dark| {
        set_system_theme(is_dark);
    });
}

fn setup_theme_subscription(window: &MainWindow) {
    let window_weak = window.as_weak();

    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_millis(500));
        let is_dark = get_system_theme();

        let window_weak_clone = window_weak.clone();
        slint::invoke_from_event_loop(move || {
            if let Some(window) = window_weak_clone.upgrade() {
                window.set_is_dark_theme(is_dark);
            }
        })
        .ok();
    });
}
