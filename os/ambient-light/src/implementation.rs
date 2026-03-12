// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::{error::Error, AmbientLightMeasurement, AmbientLightSubscribe},
    gpio::{GpioPin, PinSettings},
    i2c::Peripheral,
    ltr303::Ltr303,
    num_traits::ToPrimitive,
    server::{ScalarEventHandler, ScalarEventSubscriptionHandler, ServerContext},
};

i2c::use_api!();
gpio::use_api!();

const LEVEL_CHANGE_THRESHOLD_RATIO: f32 = 0.05;

#[derive(server::Server)]
#[name = "os/ambient-light"]
pub struct AmbientLightServer {
    als: Ltr303<I2cPeripheral>,
    als_subscribers: Vec<server::ScalarEventSubscriber<AmbientLightMeasurement>>,
}

impl server::Server for AmbientLightServer {
    fn on_start(&mut self, context: &mut server::ServerContext<Self>) {
        log::debug!("Claiming ALS interrupt pin signal");
        let gpio_api = GpioApi::default();
        gpio_api
            .claim_pin(GpioPin::AlsIrqB, PinSettings::InterruptRising, false)
            .expect("Couldn't claim GPIO pin");
        gpio_api.enable_irq(GpioPin::AlsIrqB, context).expect("Couldn't init GPIO IRQ");
    }
}

impl AmbientLightServer {
    pub(crate) fn new() -> Result<Self, Error> {
        log::debug!("Claiming ALS I2C peripheral");
        let i2c_api = I2cApi::default();
        let als = i2c_api.claim_peripheral(Peripheral::AmbientLightSensor)?;

        let mut als = Ltr303::new(als);
        als.verify_chip_id()?;
        als.reset()?;

        Ok(AmbientLightServer { als, als_subscribers: Default::default() })
    }

    fn enable_als(&mut self) -> Result<(), Error> {
        log::info!("Enabling Ambient Light sensor");
        self.als.set_interrupt_threshold(0..0)?;
        self.als.enable_interrupts()?;
        self.als.enable(ltr303::Gain::Gain8x)?;
        Ok(())
    }

    fn read_als(&mut self) -> Result<(), Error> {
        let measurement = self.als.read();
        log::trace!("Got ALS interrupt, measurement: {measurement:?}");
        let measurement = measurement.map(|m| m.intensity_visible).unwrap_or(0);
        let thr_lo = ((measurement as f32) * (1.0 - LEVEL_CHANGE_THRESHOLD_RATIO)).to_u16().unwrap_or(0);
        let thr_hi =
            ((measurement as f32) * (1.0 + LEVEL_CHANGE_THRESHOLD_RATIO)).to_u16().unwrap_or(u16::MAX);
        self.als.set_interrupt_threshold(thr_lo..thr_hi).ok();
        self.als_subscribers
            .retain(|subscriber| subscriber.send(AmbientLightMeasurement { measurement }).is_ok());
        Ok(())
    }
}

impl ScalarEventHandler<gpio::IrqMessage> for AmbientLightServer {
    fn handle(&mut self, _msg: gpio::IrqMessage, _sender: xous::PID, _context: &mut ServerContext<Self>) {
        if let Err(e) = self.read_als() {
            log::error!("Error reading ALS: {e:?}");
        }
    }
}

impl ScalarEventSubscriptionHandler<AmbientLightSubscribe> for AmbientLightServer {
    fn handle(
        &mut self,
        _msg: AmbientLightSubscribe,
        subscriber: server::ScalarEventSubscriber<AmbientLightMeasurement>,
        _context: &mut server::ServerContext<Self>,
    ) -> Result<(), server::Infallible> {
        if self.als_subscribers.is_empty() {
            if let Err(e) = self.enable_als() {
                log::error!("Error enabling ALS: {e:?}");
            }
        }
        self.als_subscribers.push(subscriber);
        Ok(())
    }
}
