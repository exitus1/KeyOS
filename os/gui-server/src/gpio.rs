// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use gpio::{GpioPin, PinSettings};
use log::debug;
use server::{ScalarEventHandler, ServerContext};

use crate::Gui;

gpio::use_api!();
pub use gpio_permissions::GpioPermissions;

impl Gui {
    pub(crate) fn subscribe_to_gpio(&mut self, context: &mut ServerContext<Gui>) {
        let gpio_api = GpioApi::default();
        gpio_api
            .claim_pin(GpioPin::CtpIrqB, PinSettings::InterruptFalling, false)
            .expect("Could not claim touch IRQ pin");
        gpio_api
            .claim_pin(GpioPin::PowerButton, PinSettings::InterruptBoth, true)
            .expect("Could not claim power button pin");

        debug!("Enabling power button pin IRQ");
        gpio_api
            .enable_irq(GpioPin::PowerButton, context)
            .expect("Could not enable power button GPIO interrupt");

        debug!("Enabling touch controller IRQ");
        gpio_api.enable_irq(GpioPin::CtpIrqB, context).expect("Could not enable touch GPIO interrupt");
    }
}

impl ScalarEventHandler<gpio::IrqMessage> for Gui {
    fn handle(&mut self, msg: gpio::IrqMessage, _sender: xous::PID, _context: &mut ServerContext<Self>) {
        match msg.pin {
            GpioPin::CtpIrqB => self.handle_touch_irq(),
            GpioPin::PowerButton => self.handle_power_button(!msg.is_high),

            _ => (),
        }
    }
}
