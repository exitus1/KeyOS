// SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use {crate::camera::CameraServer, server::Server};

pub mod api;
mod camera;
pub mod error;
pub mod messages;

gui_server_api::use_api!();
settings::use_api!();

impl Server for CameraServer {
    fn on_start(&mut self, context: &mut server::ServerContext<Self>) {
        self.start(context).expect("run() failed");
        SettingsApi::default().server_subscribe_camera_enabled(context);
    }
}

pub fn listen() { server::listen(CameraServer::new().unwrap()) }
