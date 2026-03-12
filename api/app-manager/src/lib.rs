// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later
pub use error::*;
pub use messages::*;

pub mod error;
pub mod messages;

use anyhow::bail;
use server::{CheckedConn, CheckedPermissions, MessageAllowed};
use xous::{AppId, APP_ID_SIZE, PID};

#[macro_export]
macro_rules! use_api {
    () => {
        mod app_manager_permissions {
            use app_manager::messages::*;
            #[derive(Clone, Default, server::Permissions)]
            #[server_name = "os/app-manager"]
            pub struct AppManagerPermissions;
        }
        type AppManagerApi = app_manager::AppManagerApi<app_manager_permissions::AppManagerPermissions>;
    };
}

#[derive(Default)]
pub struct AppManagerApi<P: CheckedPermissions>(pub(crate) CheckedConn<P>);

impl<P: CheckedPermissions> AppManagerApi<P> {
    pub fn launch_app_blocking(&self, app_id: &AppId) -> Result<PID, AppManagerError>
    where
        P: MessageAllowed<LaunchAppBlocking>,
    {
        self.0
            .try_send_blocking_scalar(LaunchAppBlocking(*app_id))
            .map_err(|_| AppManagerError::InternalError)?
    }

    pub fn launch_app(&self, app_id: &AppId) -> Result<(), xous::Error>
    where
        P: MessageAllowed<LaunchApp>,
    {
        self.0.try_send_scalar(LaunchApp(*app_id))?;
        Ok(())
    }

    pub fn app_name_by_app_id(&self, id: &AppId, locale: &str) -> Option<String>
    where
        P: MessageAllowed<GetAppName>,
    {
        self.0.send_archive(GetAppName::new_by_app_id(id, locale))
    }

    pub fn app_name_by_pid(&self, pid: PID, locale: &str) -> Option<String>
    where
        P: MessageAllowed<GetAppName>,
    {
        self.0.send_archive(GetAppName::new_by_pid(pid, locale))
    }
}

pub fn decode_app_id_str(id: &str) -> anyhow::Result<AppId> {
    let id = id.strip_prefix("0x").unwrap_or(id);

    if id.len() != APP_ID_SIZE * 2 {
        bail!("Invalid hex AppId length: {}, expected {} hex characters", id.len(), APP_ID_SIZE * 2);
    }

    let mut id_bytes = [0u8; APP_ID_SIZE];
    hex::decode_to_slice(id, &mut id_bytes)?;

    Ok(AppId(id_bytes))
}
