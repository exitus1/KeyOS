// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use {
    slint_keyos_platform::app,
    std::{rc::Rc, thread, time::Duration},
};

power_manager::use_api!();

app!("System Actions");

fn app_main(cx: AppContext, ui: AppWindow) {
    log_server::init_wait(env!("CARGO_CRATE_NAME")).unwrap();
    log::set_max_level(log::LevelFilter::Info);

    let settings = Rc::new(SettingsApi::default());
    ui.global::<Callbacks>().set_touch_offset(settings.get_touch_offset().0);

    ui.global::<Callbacks>().on_reboot({
        let gui = cx.gui.clone();
        move || {
            log::info!("Turning LCD off");
            gui.shutdown().ok();

            log::info!("Rebooting");
            thread::sleep(Duration::from_millis(500));
            let power_manager_api = PowerManagerApi::default();
            power_manager_api.reboot().ok();
        }
    });

    ui.global::<Callbacks>().on_shut_down({
        let gui = cx.gui.clone();
        move || {
            log::info!("Shutting down");
            gui.shutdown().ok();
        }
    });

    ui.global::<Callbacks>().on_crash_test(move || {
        log::info!("Crashing the app with a panic message");
        panic!("This application has crashed with a test panic message");
    });

    ui.global::<Callbacks>().set_debug_touch(settings.get_debug_touch().0);

    ui.global::<Callbacks>().on_set_debug_touch({
        let ui = ui.as_weak();
        let settings = settings.clone();
        move |debug| {
            settings.set_debug_touch(debug);
            let ui = ui.unwrap();
            ui.global::<Callbacks>().set_debug_touch(debug);
        }
    });

    ui.global::<Callbacks>().on_set_touch_offset({
        let ui = ui.as_weak();
        let settings = settings.clone();
        move |value| {
            settings.set_touch_offset(value);
            let ui = ui.unwrap();
            ui.global::<Callbacks>().set_touch_offset(value);
        }
    });

    ui.run().expect("UI running");
}
