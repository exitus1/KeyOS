// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use slint_keyos_platform::{
    app,
    gui_server_api::navigation::filepicker::{
        AllowedExtensions, AllowedLocations, Location, SelectFileOptions,
    },
    navigation::select_file,
};

app!("File Picker Demo");

use gui_permissions::GuiPermissions;

fn app_main(_cx: AppContext, ui: AppWindow) {
    log_server::init_wait(env!("CARGO_CRATE_NAME")).unwrap();
    log::set_max_level(log::LevelFilter::Info);

    ui.global::<Callbacks>().on_open_file_picker({
        let weak_ui = ui.as_weak();
        move || {
            let ui = weak_ui.unwrap();

            let options = SelectFileOptions::default()
                .with_hidden_allowed(ui.get_hidden_allowed())
                .with_dirs_allowed(ui.get_dirs_allowed())
                .with_allowed_locations(AllowedLocations::All)
                .with_allowed_extensions(AllowedExtensions::All)
                .with_search_allowed(ui.get_search_allowed());
            let result =
                if let Some(result) = select_file::<GuiPermissions>(options).expect("navigation result") {
                    result
                        .files()
                        .iter()
                        .map(|(path, location)| {
                            let location = match location {
                                Location::Internal => "<user>: ".to_string(),
                                Location::Airlock => "<airlock>: ".to_string(),
                                Location::External => "<usb>: ".to_string(),
                            };
                            format!("{}{}", location, path)
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                } else {
                    "<cancelled>".to_string()
                };

            ui.set_picker_result(result.into());
        }
    });

    ui.run().expect("UI running");
}
