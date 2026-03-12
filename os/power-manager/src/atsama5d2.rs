// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashSet;

use atsama5d27::{
    pmc::{PeripheralId, Pmc},
    rstc::Rstc,
};
use bq24157::Bq24157;
use bq27421::Bq27421;
use gpio::{GpioPin, PinSettings};
use i2c::Peripheral;
use power_manager::messages::*;
use power_manager::{ChargeStatus, PowerManagerError, Status};
use server::{
    ArchiveHandler, BlockingScalar, BlockingScalarHandler, ScalarEventHandler, ScalarEventSubscriber,
    ScalarEventSubscriptionHandler, ScalarHandler, ServerContext,
};
use tusb320::Tusb320;
use utralib::{HW_PMC_BASE, HW_RSTC_BASE};
use xous::{MemoryFlags, PID};

i2c::use_api!();
gpio::use_api!();

const DMA_CAPABLE_PERIPHERALS: [PeripheralId; 11] = [
    PeripheralId::Xdmac0,
    // Not included because it is owned by the kernel, and the kernel will not go to
    // deep sleep while using it.
    // PeripheralId::Xdmac1,
    PeripheralId::Lcdc,
    PeripheralId::Sdmmc0,
    PeripheralId::Sdmmc1,
    PeripheralId::Isi,
    // Not included because it is only active when the MCU or other DMA peripherals are active
    // PeripheralId::Aesb,
    PeripheralId::Icm,
    PeripheralId::Uhphs,
    PeripheralId::Udphs,
    PeripheralId::Gmac,
    PeripheralId::Can0Int0,
    PeripheralId::Can1Int0,
];

#[derive(server::Server)]
#[name = "os/power-manager"]
pub struct PowerManagerServer {
    pmc: Pmc,
    rstc: Rstc,
    charger: Bq24157<I2cPeripheral>,
    last_reported_charge_fault: Option<bq24157::ChargeFault>,
    num_faults: u32,
    fuel_gauge: Bq27421<I2cPeripheral>,
    port_controller: Tusb320<I2cPeripheral>,
    enabled_peripherals: HashSet<PeripheralId>,
    utmi_clock_enabled: bool,
    status_update_subscribers: Vec<ScalarEventSubscriber<Status>>,
    last_status: Option<Status>,
}

impl server::Server for PowerManagerServer {
    fn on_start(&mut self, context: &mut ServerContext<Self>) {
        let gpio_api: gpio::GpioApi<gpio_permissions::GpioPermissions> = gpio::GpioApi::default();
        gpio_api
            .enable_irq(GpioPin::BatChgStat, context)
            .expect("Could not enable charger status GPIO interrupt");
        gpio_api.enable_irq(GpioPin::FuelIrqB, context).expect("Could not enable fuel gauge GPIO interrupt");
        gpio_api
            .enable_irq(GpioPin::UsbCtrlIrqB, context)
            .expect("Could not enable USB port controller GPIO interrupt");
    }
}

impl ScalarEventHandler<gpio::IrqMessage> for PowerManagerServer {
    fn handle(&mut self, msg: gpio::IrqMessage, _sender: xous::PID, _context: &mut ServerContext<Self>) {
        match msg.pin {
            gpio::GpioPin::BatChgStat => {
                log::debug!("irq: battery charger status changed");
            }
            gpio::GpioPin::FuelIrqB => {
                log::debug!("irq: fuel gauge status changed");
            }
            gpio::GpioPin::UsbCtrlIrqB => {
                log::debug!("irq: USB port controller status changed");
                self.port_controller.clear_interrupt().ok();
            }
            _ => return,
        }

        self.update_status();
    }
}

impl PowerManagerServer {
    pub fn new() -> Result<Self, PowerManagerError> {
        // Map the PMC
        let pmc_mem = xous::map_memory(
            Some(xous::MemoryAddress::new(HW_PMC_BASE).unwrap()),
            None,
            0x1000,
            MemoryFlags::W | MemoryFlags::DEV,
        )?;

        log::debug!("Initializing PMC");
        let pmc_addr = pmc_mem.as_ptr() as u32;
        let mut pmc = Pmc::with_alt_base_addr(pmc_addr);

        let mut enabled_peripherals = HashSet::default();
        for pid in 2..60 {
            let Ok(pid) = PeripheralId::try_from(pid) else { continue };
            if pmc.is_peripheral_clock_enabled(pid) {
                log::trace!("Found enabled peripheral: {pid:?}");
                enabled_peripherals.insert(pid);
            }
        }

        // Map the RSTC peripheral
        let rstc_mem = xous::map_memory(
            Some(xous::MemoryAddress::new(HW_RSTC_BASE).unwrap()),
            None,
            0x1000,
            MemoryFlags::W | MemoryFlags::DEV,
        )?;

        log::debug!("Initializing RSTC");
        let rstc_addr = rstc_mem.as_ptr() as u32;
        let rstc = Rstc::with_alt_base_addr(rstc_addr);

        log::debug!("Claiming I2C peripherals");
        let i2c_api = I2cApi::default();
        let charger_periph = i2c_api.claim_peripheral(Peripheral::BatteryCharger)?;
        let charger = Bq24157::new(charger_periph);
        let fuel_gauge_periph = i2c_api.claim_peripheral(Peripheral::FuelGauge)?;
        let fuel_gauge = Bq27421::new(fuel_gauge_periph);
        let port_controller_periph = i2c_api.claim_peripheral(Peripheral::UsbPortController)?;
        let mut port_controller = Tusb320::new(port_controller_periph);
        port_controller.clear_interrupt().ok(); // Allow for the new interrupts to be seen

        log::debug!("Claiming interrupt pins");
        let gpio_api = GpioApi::default();
        gpio_api
            .claim_pin(GpioPin::BatChgStat, PinSettings::InterruptFalling, false)
            .expect("Could not claim batt charger stat IRQ pin");
        gpio_api
            .claim_pin(GpioPin::FuelIrqB, PinSettings::InterruptFalling, false)
            .expect("Could not claim fuel gauge IRQ pin");
        gpio_api
            .claim_pin(GpioPin::UsbCtrlIrqB, PinSettings::InterruptFalling, false)
            .expect("Could not claim USB port controller IRQ pin");

        log::debug!("Power manager initialized");

        Ok(Self {
            pmc,
            rstc,
            charger,
            fuel_gauge,
            port_controller,
            enabled_peripherals,
            utmi_clock_enabled: false,
            last_reported_charge_fault: None,
            num_faults: 0,
            status_update_subscribers: Vec::new(),
            last_status: None,
        })
    }

    fn update_utmi_clock(&mut self) {
        // The USB peripherals obviously use the UTMI clock, but it is also used as a generic clock input by
        // the SDMMC0 controller (as set up by at91bootstrap)
        if self.enabled_peripherals.contains(&PeripheralId::Uhphs)
            || self.enabled_peripherals.contains(&PeripheralId::Udphs)
            || self.enabled_peripherals.contains(&PeripheralId::Sdmmc0)
        {
            if !self.utmi_clock_enabled {
                log::trace!("Enabling UTMI clock");
                self.pmc.enable_utmi_clock();
                while !self.pmc.is_utmi_clock_ready() {
                    xous::yield_slice()
                }
                self.utmi_clock_enabled = true;
            }
        } else if self.utmi_clock_enabled {
            log::trace!("Disabling UTMI clock");
            self.pmc.disable_utmi_clock();
            self.utmi_clock_enabled = false;
        }
    }

    fn detect_potential_dma(&mut self) {
        let dma_possible = DMA_CAPABLE_PERIPHERALS.iter().any(|m| self.enabled_peripherals.contains(m));
        log::trace!("DMA possible: {dma_possible:?}");
        // We don't know if any DMA is actually in progress, but just to be sure if the peripheral is clocked,
        // assume that it is also doing DMA in the background.
        #[cfg(not(feature = "recovery-os"))]
        xous::set_power_management(if dma_possible {
            xous::DramIdleMode::KeepClocked
        } else {
            xous::DramIdleMode::LowPower
        })
        .ok();
    }
}

impl BlockingScalarHandler<Reboot> for PowerManagerServer {
    fn handle(&mut self, _msg: Reboot, _sender: xous::PID, _context: &mut server::ServerContext<Self>) {
        self.rstc.do_reset();
        #[allow(clippy::empty_loop)]
        loop {}
    }
}

impl BlockingScalarHandler<GetStatus> for PowerManagerServer {
    fn handle(
        &mut self,
        _msg: GetStatus,
        _sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> <GetStatus as BlockingScalar>::Response {
        log::trace!(
            "State of charge: {}  Charge current: {}  Voltage: {}  Capacity: {}",
            self.fuel_gauge.state_of_charge().unwrap(),
            self.fuel_gauge.charge_current().unwrap(),
            self.fuel_gauge.voltage().unwrap(),
            self.fuel_gauge.capacity().unwrap(),
        );

        self.update_status()
    }
}

impl BlockingScalarHandler<SetUsbBoost> for PowerManagerServer {
    fn handle(
        &mut self,
        msg: SetUsbBoost,
        _sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> SetUsbBoostResponse {
        let mut ctrl = match self.charger.control() {
            Ok(ctrl) => ctrl,
            Err(e) => {
                log::error!("Error getting control register: {e:?}");
                return SetUsbBoostResponse { success: false, previous_state: false };
            }
        };
        let previous_state = ctrl.opa_mode();
        ctrl.set_opa_mode(msg.enabled);
        match self.charger.set_control(ctrl) {
            Ok(()) => SetUsbBoostResponse { success: true, previous_state },
            Err(e) => {
                log::error!("Error setting control register: {e:?}");
                SetUsbBoostResponse { success: false, previous_state }
            }
        }
    }
}

impl BlockingScalarHandler<SetPeripheralEnabled> for PowerManagerServer {
    fn handle(
        &mut self,
        msg: SetPeripheralEnabled,
        _sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> <SetPeripheralEnabled as BlockingScalar>::Response {
        if msg.enabled {
            if !self.enabled_peripherals.contains(&msg.peripheral) {
                log::trace!("Enabling clock of {:?}", msg.peripheral);
                self.pmc.enable_peripheral_clock(msg.peripheral);
                self.enabled_peripherals.insert(msg.peripheral);
            }
        } else if self.enabled_peripherals.contains(&msg.peripheral) {
            log::trace!("Disabling clock of {:?}", msg.peripheral);
            self.pmc.disable_peripheral_clock(msg.peripheral);
            self.enabled_peripherals.remove(&msg.peripheral);
        }

        self.update_utmi_clock();
        self.detect_potential_dma();
    }
}

#[cfg(keyos)]
impl ScalarHandler<SetOtgPriority> for PowerManagerServer {
    fn handle(
        &mut self,
        SetOtgPriority(otg_priority): SetOtgPriority,
        sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) {
        log::debug!("Setting OTG priority to {otg_priority:?} by PID {sender}");
        if let Err(e) = self.port_controller.set_mode_select(otg_priority.into()) {
            log::error!("Error setting OTG priority: {e:?}");
        }
    }
}

impl ArchiveHandler<GetExtendedStatus> for PowerManagerServer {
    fn handle(
        &mut self,
        _msg: GetExtendedStatus,
        _sender: PID,
        _context: &mut ServerContext<Self>,
    ) -> <GetExtendedStatus as server::Archive>::Response {
        self.extended_status()
    }
}

impl ScalarHandler<ClearChargeFault> for PowerManagerServer {
    fn handle(
        &mut self,
        _msg: ClearChargeFault,
        _sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) {
        self.last_reported_charge_fault = None;
    }
}

impl ScalarEventSubscriptionHandler<StatusSubscribe> for PowerManagerServer {
    fn handle(
        &mut self,
        _msg: StatusSubscribe,
        subscriber: ScalarEventSubscriber<Status>,
        _context: &mut ServerContext<Self>,
    ) -> Result<(), server::Infallible> {
        let status = self.update_status();
        if subscriber.send(&status).is_err() {
            // If we couldn't send the update, then we won't add a subscriber
            return Ok(());
        }

        self.status_update_subscribers.push(subscriber);

        Ok(())
    }
}

impl PowerManagerServer {
    fn charge_status(&mut self) -> ChargeStatus {
        let Ok(raw_status) = self.charger.status() else {
            return ChargeStatus::Fault;
        };
        match raw_status.stat() {
            0 => {
                if raw_status.is_boost() {
                    ChargeStatus::Boosting
                } else {
                    ChargeStatus::Idle
                }
            }
            1 => ChargeStatus::Charging,
            2 => ChargeStatus::ChargeDone,
            _ => {
                if let Some(fault) = raw_status.charge_fault() {
                    // Normal and SleepMode aren't faults
                    if matches!(fault, bq24157::ChargeFault::Normal | bq24157::ChargeFault::SleepMode) {
                        // Normal state, no fault
                        return ChargeStatus::Idle;
                    }

                    // Ignores NoBattery fault, the actual fault should come later.
                    // Ignores BadAdaptor as this fault can often happen when the charger is disconnected
                    if !matches!(fault, bq24157::ChargeFault::NoBattery | bq24157::ChargeFault::BadAdaptor) {
                        log::warn!("Charger reported a fault: {fault:?}");

                        self.last_reported_charge_fault = Some(fault);
                        self.num_faults = self.num_faults.saturating_add(1);
                        self.reset_charger();
                    }
                }

                ChargeStatus::Fault
            }
        }
    }

    fn extended_status(&mut self) -> Option<ExtendedStatus> {
        let current = self.fuel_gauge.charge_current().ok()?;
        let voltage_mv = self.fuel_gauge.voltage().ok()?;
        let capacity_mah = self.fuel_gauge.capacity().ok()?;
        let remaining_capacity_mah = self.fuel_gauge.remaining_capacity().ok()?;

        Some(ExtendedStatus {
            current,
            voltage_mv,
            capacity_mah,
            remaining_capacity_mah,
            last_reported_fault: self.last_reported_charge_fault.map(Into::into),
            num_reported_faults: self.num_faults,
        })
    }

    fn reset_charger(&mut self) {
        if let Err(e) = self.charger.reset_charger() {
            log::error!("Error resetting charger: {e:?}");
            return;
        }

        if let Err(e) = self.charger.apply_register_dump(&keyos::batt::CHARGER_CONFIG_DUMP) {
            log::error!("Error resetting battery charger: {e:?}");
        }
    }

    fn update_status(&mut self) -> Status {
        let status = Status {
            charge_status: self.charge_status(),
            battery_percent: self.fuel_gauge.state_of_charge().unwrap_or_default(),
            attached_state: self.port_controller.attached_state().unwrap_or_default().into(),
        };

        if self.last_status != Some(status) {
            self.last_status.replace(status);
            self.status_update_subscribers.retain(|subscriber| subscriber.send(&status).is_ok())
        }

        status
    }
}
