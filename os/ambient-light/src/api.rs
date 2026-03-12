// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use server::{CheckedConn, CheckedPermissions, MessageAllowed};

use crate::{messages::*, AmbientLightMeasurement};

#[macro_export]
macro_rules! use_api {
    () => {
        mod ambient_light_permissions {
            use ambient_light::messages::*;
            #[derive(Clone, Default, server::Permissions)]
            #[server_name = "os/ambient-light"]
            pub struct AmbientLightPermissions;
        }
        type AmbientLightApi =
            ambient_light::api::AmbientLightApi<ambient_light_permissions::AmbientLightPermissions>;
    };
}

#[derive(Default)]
pub struct AmbientLightApi<P: CheckedPermissions> {
    conn: CheckedConn<P>,
}

impl<P: CheckedPermissions> AmbientLightApi<P> {
    /// Subscribe to AmbientLight updates. Updates are sent periodically (see ambient_light_POLL_INTERVAL)
    pub fn subscribe_ambient_light<S>(&self, context: &mut server::ServerContext<S>)
    where
        S: server::Server + server::ScalarEventHandler<AmbientLightMeasurement>,
        P: MessageAllowed<AmbientLightSubscribe>,
    {
        self.conn.subscribe_scalar_infallible(AmbientLightSubscribe, context);
    }
}
