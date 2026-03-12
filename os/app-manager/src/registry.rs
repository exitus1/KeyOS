// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;

use app_manager::decode_app_id_str;
use app_manifest::Manifest;
use log::error;
use xous::{AppId, PID};

use crate::launch::list_apps;

#[derive(Debug, Clone)]
pub(crate) struct AppInfo {
    id: AppId,
    elf_path: Option<String>,
    manifest: Manifest,
}

#[derive(Debug, Clone)]
pub(crate) struct RunningAppInfo {
    pub(crate) info: AppInfo,
    pub(crate) launched_by: PID,
}

#[derive(Debug, Default)]
pub(crate) struct AppRegistry {
    installed_apps: HashMap<AppId, AppInfo>,
    running_apps: HashMap<PID, RunningAppInfo>,
}

impl AppRegistry {
    pub(crate) fn scan_installed_apps(&mut self) -> anyhow::Result<()> {
        match list_apps("/keyos/apps") {
            Ok(apps_list) => {
                for (path, manifest) in apps_list {
                    let Ok(app_id) = decode_app_id_str(&manifest.app_id) else {
                        error!("Invalid app ID format in manifest: {}", manifest.app_id);
                        continue;
                    };

                    if self.installed_apps.contains_key(&app_id) {
                        continue;
                    }

                    #[cfg(not(keyos))]
                    let elf_path = path.map(|p| p.to_string_lossy().to_string());
                    #[cfg(keyos)]
                    let elf_path = path.map(|s| s.to_string());

                    self.installed_apps.insert(app_id, AppInfo { id: app_id, elf_path, manifest });
                }
            }

            Err(e) => {
                log::error!("Error listing apps: {:?}", e);
            }
        }

        Ok(())
    }

    pub(crate) fn app_name_by_id(&self, id: &AppId, locale: &str) -> Option<String> {
        self.installed_apps
            .get(id)
            .and_then(|app_info| app_info.manifest.app_name.get(&locale.to_string().into()).cloned())
    }

    pub(crate) fn app_name_by_pid(&self, pid: PID, locale: &str) -> Option<String> {
        self.running_apps
            .get(&pid)
            .and_then(|app_info| app_info.info.manifest.app_name.get(&locale.to_string().into()).cloned())
    }

    pub(crate) fn elf_path(&self, app_id: AppId) -> Option<String> {
        self.installed_apps.get(&app_id).and_then(|app_info| app_info.elf_path.clone())
    }

    pub(crate) fn register_running_app(&mut self, pid: PID, app_id: AppId, launched_by: PID) {
        self.installed_apps.get(&app_id).inspect(|app_info| {
            self.running_apps.insert(pid, RunningAppInfo { info: (*app_info).clone(), launched_by });
        });
    }

    pub(crate) fn app_id_by_pid(&self, pid: PID) -> Option<&AppId> {
        self.running_apps.get(&pid).map(|app_info| &app_info.info.id)
    }

    pub(crate) fn launched_by(&self, app_id: &AppId) -> Option<PID> {
        self.running_apps
            .values()
            .find(|app_info| app_info.info.id == *app_id)
            .map(|app_info| app_info.launched_by)
    }

    pub(crate) fn terminate_app(&mut self, pid: PID) { self.running_apps.remove(&pid); }
}
