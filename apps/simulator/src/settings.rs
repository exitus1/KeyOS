// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: MIT OR Apache-2.0

use {
    crate::{gui_permissions::GuiPermissions, MainWindow, Settings, DEP_SETTINGS_FILE, SETTINGS_FILE},
    anyhow::Context,
    slint::ComponentHandle,
};

pub fn setup(window: &MainWindow) {
    let settings = match read_settings_file() {
        Ok(s) => s,
        Err(error) => {
            log::warn!("Failed to read settings file: {}", error);
            let s = Settings { scale: 1.0 };
            write_settings_file(&s).unwrap_or_else(|error| {
                log::warn!("Failed to write default settings file: {}", error);
            });
            s
        }
    };

    set_scale(settings.scale);
    window.set_settings(settings.into());

    window.on_scale_set({
        let window = window.clone_strong();
        move |scale_factor| {
            set_scale(scale_factor);
            change_setting(&window, |settings| settings.scale = scale_factor);
        }
    });
}

pub fn change_setting(window: &MainWindow, f: impl Fn(&mut Settings)) {
    let mut settings: Settings = window.get_settings();
    f(&mut settings);
    write_settings_file(&settings).unwrap_or_else(|error| {
        log::warn!("Failed to write settings file: {}", error);
    });
    window.set_settings(settings);
}

pub fn read_settings_file() -> anyhow::Result<Settings> {
    let settings_string = std::fs::read_to_string(SETTINGS_FILE).context("Error reading settings file")?;

    serde_json::from_str(settings_string.as_str()).map_err(|error| {
        log::warn!(
            "Unable to parse settings: {}\nThis may be due to settings structure changes. Moving old settings file to {}.",
            error,
            DEP_SETTINGS_FILE
        );
        std::fs::rename(SETTINGS_FILE, DEP_SETTINGS_FILE).unwrap_or_else(|error| {
            log::warn!("Unable to move old settings file: {}", error);
        });

        anyhow::Error::new(error).context("Error parsing settings file")
    })
}

pub fn write_settings_file(settings: &Settings) -> anyhow::Result<()> {
    let settings_string = serde_json::to_string(&settings).context("Could not serialize settings")?;
    std::fs::write(SETTINGS_FILE, settings_string.as_str()).context("Could not write settings file")?;
    Ok(())
}

pub fn set_scale(scale_factor: f32) {
    gui_server_api::simulator::SimulatorApi::<GuiPermissions>::default()
        .set_scale_factor(scale_factor)
        .unwrap_or_else(|error| {
            log::warn!("Could not set scale factor: {:?}", error);
        })
}
