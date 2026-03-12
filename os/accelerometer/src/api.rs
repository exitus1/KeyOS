// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use server::{CheckedConn, CheckedPermissions, MessageAllowed};

use crate::{messages::*, AccelerometerMeasurement};

#[macro_export]
macro_rules! use_api {
    () => {
        mod accelerometer_permissions {
            use accelerometer::messages::*;
            #[derive(Clone, Default, server::Permissions)]
            #[server_name = "os/accelerometer"]
            pub struct AccelerometerPermissions;
        }
        type AccelerometerApi =
            accelerometer::api::AccelerometerApi<accelerometer_permissions::AccelerometerPermissions>;
    };
}

#[derive(Default)]
pub struct AccelerometerApi<P: CheckedPermissions> {
    conn: CheckedConn<P>,
}

impl<P: CheckedPermissions> AccelerometerApi<P> {
    /// Subscribe to Accelerometer updates. Updates are sent periodically (see ACCELEROMETER_POLL_INTERVAL)
    pub fn subscribe_accelerometer<S>(&self, context: &mut server::ServerContext<S>)
    where
        S: server::Server + server::ScalarEventHandler<AccelerometerMeasurement>,
        P: MessageAllowed<AccelerometerSubscribe>,
    {
        self.conn.subscribe_scalar_infallible(AccelerometerSubscribe, context);
    }
}
