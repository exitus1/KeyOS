// SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

#[cfg(keyos)]
mod implementation {
    use accelerometer::AccelerometerMeasurement;
    use ambient_light::AmbientLightMeasurement;
    use server::{ScalarEventHandler, Server, ServerMessages};

    accelerometer::use_api!();
    ambient_light::use_api!();

    pub struct SensorTestServer;

    impl ServerMessages for SensorTestServer {
        const NAME: &'static str = "";

        fn messages() -> &'static [server::MessageDef<Self>] { &[] }
    }
    impl Server for SensorTestServer {
        fn on_start(&mut self, context: &mut server::ServerContext<Self>) {
            log::info!("Subscribing to sensors");
            AmbientLightApi::default().subscribe_ambient_light(context);
            AccelerometerApi::default().subscribe_accelerometer(context);
            log::info!("Subscription done.");
        }
    }

    impl ScalarEventHandler<AccelerometerMeasurement> for SensorTestServer {
        fn handle(
            &mut self,
            msg: AccelerometerMeasurement,
            _sender: xous::PID,
            _context: &mut server::ServerContext<Self>,
        ) {
            log::info!("{msg:?}")
        }
    }

    impl ScalarEventHandler<AmbientLightMeasurement> for SensorTestServer {
        fn handle(
            &mut self,
            msg: AmbientLightMeasurement,
            _sender: xous::PID,
            _context: &mut server::ServerContext<Self>,
        ) {
            log::info!("{msg:?}")
        }
    }
}

pub fn main() {
    log_server::init_wait(env!("CARGO_CRATE_NAME")).unwrap();
    log::set_max_level(log::LevelFilter::Info);

    #[cfg(keyos)]
    server::listen(implementation::SensorTestServer)
}
