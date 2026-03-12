// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use embedded_hal::i2c::I2c;
use i2c::Peripheral;
use server::{BlockingScalarHandler, CheckedConn, ScalarEventSubscriptionHandler, ServerContext};

use crate::{error::Error, messages::*, AccelerometerMeasurement};

i2c::use_api!();

const ACCELEROMETER_ADDRESS: u8 = Peripheral::Accelerometer.i2c_addr();
const ACCELEROMETER_CHIP_ID: u8 = 0x02;
const ACCELEROMETER_POWER_UP: [u8; 2] = [
    0x0D, // Control register
    0x00, // FSR: 0b00 (range: +-2g), Cklsel: 0, Power down bit: 0
];
const ACCELEROMETER_POWER_DOWN: [u8; 2] = [
    0x0D, // Control register
    0x01, // Power down bit: 1
];
const ACCELEROMETER_POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(100);

#[derive(server::Server)]
#[name = "os/accelerometer"]
pub struct AccelerometerServer {
    accelerometer: I2cPeripheral,
    accelerometer_subscribers: Vec<server::ScalarEventSubscriber<AccelerometerMeasurement>>,
}

impl server::Server for AccelerometerServer {}

#[derive(Debug, Default, Clone, server::Permissions)]
#[server_name = "os/accelerometer"]
#[all_permissions]
struct InternalPermissions;

impl AccelerometerServer {
    pub(crate) fn new() -> Result<Self, Error> {
        log::debug!("Claiming Accelerometer I2C peripheral");
        let i2c_api = I2cApi::default();
        let mut accelerometer = i2c_api.claim_peripheral(Peripheral::Accelerometer)?;

        let mut chip_id = [0u8; 1];
        accelerometer.write_read(ACCELEROMETER_ADDRESS, &[0x0e], &mut chip_id)?;
        if chip_id[0] != ACCELEROMETER_CHIP_ID {
            log::error!(
                "Wrong chip ID read from accelerometer: 0x{:02x} instead of 0x{ACCELEROMETER_CHIP_ID:02x})",
                chip_id[0]
            );
            return Err(Error::UnknownError);
        }
        accelerometer.write(ACCELEROMETER_ADDRESS, &ACCELEROMETER_POWER_DOWN)?;

        Ok(AccelerometerServer { accelerometer, accelerometer_subscribers: Default::default() })
    }

    fn enable_accelerometer(&mut self) -> Result<(), Error> {
        log::info!("Enabling Accelerometer");
        self.accelerometer.write(ACCELEROMETER_ADDRESS, &ACCELEROMETER_POWER_UP)?;
        std::thread::spawn(Self::accelerometer_polling_thread);
        Ok(())
    }

    fn accelerometer_polling_thread() {
        log::debug!("Starting Accelerometer polling thread");
        let conn = CheckedConn::<InternalPermissions>::default();
        loop {
            if let Err(e) = conn.try_send_blocking_scalar(AccelerometerPoll) {
                log::error!("Error sending accelerometer poll message: {e:?}");
                return;
            }
            std::thread::sleep(ACCELEROMETER_POLL_INTERVAL);
        }
    }

    fn read_accelerometer(&mut self) -> Result<(), Error> {
        let mut coordinates = [0i16; 3];

        // Read registers 0x03-0x08: x, y, z measurements, 2 bytes each.
        for (i, coord) in coordinates.iter_mut().enumerate() {
            let mut regs = [0u8; 2];
            self.accelerometer.write_read(ACCELEROMETER_ADDRESS, &[0x03 + i as u8 * 2], &mut regs)?;

            let coord16 = (((regs[0] as u16) << 8) | (regs[1] as u16)) as i16;

            // The data is only 12 bits, and starts from the highest bit, i.e. the lowest 4 bits are always 0.
            // Rust bit shift to right is an arithmetic shift for signed types,
            // i.e. it will be sign-extended
            *coord = coord16 >> 4
        }
        log::trace!("Got ALS interrupt, coordinates: {coordinates:02x?}");

        self.accelerometer_subscribers.retain(|subscriber| {
            subscriber
                .send(AccelerometerMeasurement { x: coordinates[0], y: coordinates[1], z: coordinates[2] })
                .is_ok()
        });

        Ok(())
    }
}

impl ScalarEventSubscriptionHandler<AccelerometerSubscribe> for AccelerometerServer {
    fn handle(
        &mut self,
        _msg: AccelerometerSubscribe,
        subscriber: server::ScalarEventSubscriber<AccelerometerMeasurement>,
        _context: &mut ServerContext<Self>,
    ) -> Result<(), server::Infallible> {
        if self.accelerometer_subscribers.is_empty() {
            if let Err(e) = self.enable_accelerometer() {
                log::error!("Error enabling Accelerometer: {e:?}");
            }
        }
        self.accelerometer_subscribers.push(subscriber);
        Ok(())
    }
}

impl BlockingScalarHandler<AccelerometerPoll> for AccelerometerServer {
    fn handle(&mut self, _msg: AccelerometerPoll, sender: xous::PID, _context: &mut ServerContext<Self>) {
        if sender != xous::current_pid().unwrap() {
            return;
        }
        if let Err(e) = self.read_accelerometer() {
            log::error!("Error reading Accelerometer: {e:?}");
        }
    }
}
