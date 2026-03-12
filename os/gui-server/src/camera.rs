// SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::{AppWindow, DoubleBufferVMA, Gui},
    log::{info, warn},
    std::time::Duration,
    xous::{CID, PID},
};

camera::use_api!();

const CAMERA_CONNECTION_TIMEOUT_MS: u64 = 1000;

pub(crate) struct CameraWindow {
    #[allow(dead_code)]
    pub(crate) input_cid: CID,
    pub(crate) pid: PID,
    pub(crate) bufs: DoubleBufferVMA,
    pub(crate) notified_visible: bool,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct CameraState {
    pub(crate) y_pos: u16,
    state: CameraVisibilityState,
}

#[derive(Debug, Copy, Clone, Default)]
pub(crate) enum CameraVisibilityState {
    #[default]
    Hidden,
    Showing,
}

impl AppWindow {
    pub(crate) fn is_camera_visible(&self) -> bool {
        matches!(self.camera_state.state, CameraVisibilityState::Showing)
    }
}

impl Gui {
    pub(crate) fn show_camera_for_app(&mut self, pid: PID) {
        let Some(window) = self.windows.get_mut(&pid) else {
            warn!("Requested to show camera for PID={pid} but no window found");
            return;
        };

        info!("Requested to show camera by PID={pid}");
        window.camera_state.state = CameraVisibilityState::Showing;
        self.update_camera_window();
    }

    pub(crate) fn hide_camera_for_app(&mut self, pid: PID) {
        let Some(window) = self.windows.get_mut(&pid) else {
            warn!("Requested to hide camera for PID={pid} but no window found");
            return;
        };

        info!("Requested to hide the camera by PID={pid}");
        window.camera_state.state = CameraVisibilityState::Hidden;
        self.update_camera_window();
    }

    pub(crate) fn update_camera_window(&mut self) {
        let Some(pid) = self.active_app_pid() else { return };
        let Some(camera_api) = self.camera_server_connection() else { return };
        let Some(window) = self.windows.get_mut(&pid) else { return };
        let Some(camera_window) = &mut self.camera_window else { return };

        let visible = window.is_camera_visible();
        if visible != camera_window.notified_visible {
            if let Err(e) = camera_api.notify_visible(visible) {
                log::error!("Couln't notify camera of visible={visible:?}: {e:?}");
            } else {
                camera_window.notified_visible = visible;
            }
        }
    }

    pub(crate) fn camera_window_notify_hidden(&mut self) {
        let Some(camera_api) = self.camera_server_connection() else { return };
        let Some(camera_window) = &mut self.camera_window else { return };
        if camera_window.notified_visible {
            if let Err(e) = camera_api.notify_visible(false) {
                log::error!("Couln't notify camera that it's hidden: {e:?}");
            } else {
                camera_window.notified_visible = false;
            }
        }
    }

    pub(crate) fn swap_camera_bufs(&mut self, pid: PID) -> bool {
        let camera_visible = self.with_active_app_mut(|app| app.is_camera_visible()).unwrap_or(false);

        if let Some(camera_window) = &mut self.camera_window {
            if camera_visible && camera_window.pid == pid {
                camera_window.bufs.swap();
                return true;
            }
        }

        false
    }

    fn camera_server_connection(&mut self) -> Option<CameraApi> {
        CameraApi::try_new_with_timeout(Duration::from_millis(CAMERA_CONNECTION_TIMEOUT_MS))
    }
}
