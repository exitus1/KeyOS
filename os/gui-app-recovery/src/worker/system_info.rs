// SPDX-FileCopyrightText: 2024-2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use slint_keyos_platform::{async_archive, slint::ComponentHandle, spawn_local};

use crate::{recovery_worker_permissions::RecoveryWorkerPermissions, AppWindow, InfoGlobal};

pub fn subscribe_bootloader(ui: AppWindow) {
    spawn_local(async move {
        let bootloader =
            async_archive::<RecoveryWorkerPermissions, _>(recovery_worker::GetBootloaderInfo).await;
        log::info!("received Bootloader info: {:?}", bootloader);
        ui.global::<InfoGlobal>().set_bootloader_hash(bootloader.hash_str.into());
    })
    .detach();
}

pub fn subscribe_keyos(ui: AppWindow) {
    spawn_local(async move {
        let keyos = async_archive::<RecoveryWorkerPermissions, _>(recovery_worker::GetKeyOsInfo).await;
        log::info!("received Keyos info: {:?}", keyos);
        ui.global::<InfoGlobal>().set_firmware_hash(keyos.hash_str.into());
        ui.global::<InfoGlobal>().set_firmware_version(keyos.version.into());
        ui.global::<InfoGlobal>().set_firmware_build_date(keyos.date_str.into());
    })
    .detach();
}

pub fn subscribe_recovery(ui: AppWindow) {
    spawn_local(async move {
        let recovery = async_archive::<RecoveryWorkerPermissions, _>(recovery_worker::GetRecoveryInfo).await;
        log::info!("received Recovery info: {:?}", recovery);
        ui.global::<InfoGlobal>().set_recovery_hash(recovery.hash_str.into());
        ui.global::<InfoGlobal>().set_recovery_version(recovery.version.into());
        ui.global::<InfoGlobal>().set_recovery_build_date(recovery.date_str.into());
    })
    .detach();
}
