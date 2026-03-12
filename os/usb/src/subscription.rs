// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use server::{CheckedPermissions, WithAllPermissions};
use settings::global::{DeviceName, UsbEnabled};

settings::use_api!();

#[derive(Default)]
pub struct SubscriptionServer {
    device: usb::device::api::UsbDeviceEmulation<WithAllPermissions<DevicePermissions>>,
    host: usb::host::api::UsbHost<WithAllPermissions<HostPermissions>>,
}

#[derive(Default, Clone)]
struct DevicePermissions;

impl CheckedPermissions for DevicePermissions {
    const NAME: &str = "os/usbdev";
}

#[derive(Default, Clone)]
struct HostPermissions;

impl CheckedPermissions for HostPermissions {
    const NAME: &str = "os/usb";
}

impl server::ServerMessages for SubscriptionServer {
    const NAME: &'static str = "";

    fn messages() -> &'static [server::MessageDef<Self>] { &[] }
}

impl server::Server for SubscriptionServer {
    fn on_start(&mut self, context: &mut server::ServerContext<Self>) {
        let settings = SettingsApi::default();
        settings.server_subscribe_device_name(context);
        settings.server_subscribe_usb_enabled(context);
    }
}

impl server::ArchiveEventHandler<DeviceName> for SubscriptionServer {
    fn handle(
        &mut self,
        msg: server::Owned<DeviceName>,
        _sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) {
        let Ok(name) = msg.deserialize() else { return };
        log::debug!("received device name event {:?}", name.0);
        *crate::DEVICE_NAME.lock().unwrap_or_else(|e| e.into_inner()) = name.0;
    }
}

impl server::ScalarEventHandler<UsbEnabled> for SubscriptionServer {
    fn handle(
        &mut self,
        UsbEnabled(enabled): UsbEnabled,
        _sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) {
        log::debug!("received usb enabled event {enabled}");
        self.device
            .set_enabled(enabled)
            .inspect_err(|e| log::warn!("failed to set usb device enabled {enabled:?} {e:?}"))
            .ok();
        self.host
            .set_enabled(enabled)
            .inspect_err(|e| log::warn!("failed to set usb host enabled {enabled:?} {e:?}"))
            .ok();
    }
}
