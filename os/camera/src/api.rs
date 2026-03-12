// SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::time::Duration;

#[cfg(not(keyos))]
use gui_server_api::consts::CAMERA_FB_SIZE_BYTES;
use server::{CheckedConn, CheckedPermissions, MessageAllowed};
use {crate::messages::*, xous::MemoryRange};

pub const SERVER_NAME: &str = "os/camera";

#[macro_export]
macro_rules! use_api {
    () => {
        mod camera_permissions {
            use camera::messages::*;
            #[derive(Clone, Default, server::Permissions)]
            #[server_name = "os/camera"]
            pub struct CameraPermissions;
        }
        type CameraApi = camera::api::CameraApi<camera_permissions::CameraPermissions>;
    };
}

#[derive(Default)]
pub struct CameraApi<P: CheckedPermissions> {
    conn: CheckedConn<P>,
    frame_mem: Option<MemoryRange>,
}

impl<P: CheckedPermissions> CameraApi<P> {
    pub fn try_new_with_timeout(timeout: Duration) -> Option<Self> {
        Some(Self { conn: CheckedConn::try_connect_with_timeout(timeout)?, frame_mem: None })
    }

    pub fn is_frame_ready(&self) -> bool
    where
        P: MessageAllowed<IsReady>,
    {
        self.conn.try_send_blocking_scalar(IsReady).unwrap_or(false)
    }

    /// Requests the latest camera frame.
    ///
    /// *Note*: the frame is allocated client-side internally, and its `MemoryRange` is
    /// returned.
    pub fn get_frame_mirror(&mut self) -> Result<MemoryRange, xous::Error>
    where
        P: MessageAllowed<GetFrameMemoryMirror>,
    {
        if let Some(frame_mem) = self.frame_mem {
            Ok(frame_mem)
        } else {
            let frame_mem = self
                .conn
                .try_send_blocking_scalar(GetFrameMemoryMirror)?
                .ok_or(xous::Error::InternalError)?;
            self.frame_mem = Some(frame_mem);
            Ok(frame_mem)
        }
    }

    /// Enable the use of the camera. Intended to be used by the control center
    pub fn set_enabled(&self, enabled: bool) -> Result<(), xous::Error>
    where
        P: MessageAllowed<SetEnabled>,
    {
        self.conn.try_send_scalar(SetEnabled(enabled))?;
        Ok(())
    }

    /// Notify the app that the camera image is visible on the screen.
    /// Inteded to be used by the GUI server
    pub fn notify_visible(&self, visible: bool) -> Result<(), xous::Error>
    where
        P: MessageAllowed<NotifyVisible>,
    {
        self.conn.try_send_scalar(NotifyVisible(visible)).map_err(From::from)?;
        Ok(())
    }

    pub fn is_enabled(&self) -> Result<bool, xous::Error>
    where
        P: MessageAllowed<IsEnabled>,
    {
        self.conn.try_send_blocking_scalar(IsEnabled).map_err(From::from)
    }

    pub fn is_in_use(&self) -> Result<bool, xous::Error>
    where
        P: MessageAllowed<IsInUse>,
    {
        self.conn.try_send_blocking_scalar(IsInUse).map_err(From::from)
    }

    #[cfg(not(keyos))]
    pub fn get_frame_buffer_addr(&mut self) -> Result<usize, xous::Error>
    where
        P: MessageAllowed<GetFrameBufId>,
    {
        use gui_server_api::consts::{CAMERA_BYTES_PER_PX, CAMERA_MARGIN, CAMERA_WIDTH};

        let id = self.conn.send_archive(GetFrameBufId);
        let id_str = match gui_server_api::utils::str_from_u8_nul_utf8(&id) {
            Ok(s) => s,
            Err(e) => {
                log::warn!("could not parse frame buffer id: {}", e);
                return Err(xous::Error::InternalError);
            }
        };
        match gui_server_api::utils::fb_id_to_addr(id_str, CAMERA_FB_SIZE_BYTES) {
            Ok(a) => Ok(a + CAMERA_MARGIN * CAMERA_WIDTH * CAMERA_BYTES_PER_PX),
            Err(_) => Err(xous::Error::InternalError),
        }
    }
}
