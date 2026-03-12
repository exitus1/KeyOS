// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later
use std::thread;
use std::time::Duration;

use keyos_integration_test::{assert_eq, pass};
use settings::global::SystemTheme;

settings::use_api!();

fn main() {
    log_server::init_wait(env!("CARGO_CRATE_NAME")).unwrap();
    log::set_max_level(log::LevelFilter::Debug);

    thread::sleep(Duration::from_secs(1));

    let tester = Tester::new();

    tester.run("global", |s| {
        let worker = worker::WorkerHandle::default();
        worker::test_executor::block_on(test_global(s, &worker));
    });
    log::info!("settings tests completed");
    pass();
}

struct Tester {
    api: SettingsApi,
}

impl Tester {
    fn new() -> Self { Self { api: SettingsApi::default() } }

    fn run(&self, name: &str, f: impl FnOnce(&SettingsApi)) {
        let start = std::time::Instant::now();
        f(&self.api);
        let duration = start.elapsed();
        log::info!("{name} success - {:?}", duration);
    }
}

async fn test_global(settings: &SettingsApi, worker: &worker::WorkerHandle) {
    // scalar test
    settings.set_system_theme(SystemTheme::Light);
    assert_eq!(settings.get_system_theme(), SystemTheme::Light);

    let mut theme_updates = worker.subscribe_scalar::<settings_permissions::SettingsPermissions, _>(
        settings::messages::SubscribeSystemTheme,
    );
    assert_eq!(theme_updates.next().await, Some(SystemTheme::Light), "initial theme");

    settings.set_system_theme(SystemTheme::Dark);
    assert_eq!(theme_updates.next().await, Some(SystemTheme::Dark));

    settings.set_system_theme(SystemTheme::Light);
    assert_eq!(theme_updates.next().await, Some(SystemTheme::Light));

    // archive test
    settings.set_device_name("test".to_string());
    assert_eq!(settings.get_device_name(), "test".into());

    let mut device_name_updates = worker.subscribe_archive::<settings_permissions::SettingsPermissions, _>(
        settings::messages::SubscribeDeviceName,
    );
    assert_eq!(device_name_updates.next().await, Some("test".into()), "initial device name");

    settings.set_device_name("test2".to_string());
    assert_eq!(device_name_updates.next().await, Some("test2".into()));

    settings.set_device_name("test3".to_string());
    assert_eq!(device_name_updates.next().await, Some("test3".into()));
}
