// SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use i2c::messages::*;
use i2c::{I2cError, Peripheral};
use server::ArchiveHandler;
use {
    atsama5d27::twi::Twi,
    server::{BlockingScalarHandler, Server, ServerContext},
    std::collections::HashMap,
    utralib::HW_TWIHS0_BASE,
    xous::{keyos::MASTER_CLOCK_SPEED, PID},
};

/// TWI (I2C) bus speed.
const TWI_BUS_SPEED_HZ: usize = 100_000;

static mut I2C_SERVER_STATE: Option<I2cServerState> = None;

#[derive(Debug, server::Server)]
#[name = "os/i2c"]
pub struct I2cServer {}

pub(crate) struct I2cServerState {
    claimed_peripherals: HashMap<Peripheral, PID>,
    twi: Twi,
}

impl I2cServer {
    pub fn init() -> Self {
        log::debug!("Initializing TWI0");

        let mem = xous::map_memory(
            xous::MemoryAddress::new(HW_TWIHS0_BASE),
            None,
            0x1000,
            xous::MemoryFlags::W | xous::MemoryFlags::DEV,
        )
        .expect("map TWI0");
        let addr = mem.as_ptr() as u32;
        log::debug!("Mapped TWI0 to 0x{:08x}", addr);

        let twi = Twi::with_base_addr(addr);
        twi.init_master(MASTER_CLOCK_SPEED as usize, TWI_BUS_SPEED_HZ);

        log::debug!("Initialized TWI0 master at {} Hz bus speed", TWI_BUS_SPEED_HZ);

        unsafe {
            I2C_SERVER_STATE = Some(I2cServerState { twi, claimed_peripherals: Default::default() });
        }
        Self {}
    }
}

impl Server for I2cServer {}

impl BlockingScalarHandler<ClaimPeripheral> for I2cServer {
    fn handle(
        &mut self,
        ClaimPeripheral(peripheral): ClaimPeripheral,
        sender: PID,
        _context: &mut ServerContext<Self>,
    ) -> Result<(), I2cError> {
        log::debug!("PID={sender:} tries to claim {peripheral:?} I2C peripheral");

        let state = unsafe { (&mut *core::ptr::addr_of_mut!(I2C_SERVER_STATE)).as_mut() }
            .ok_or(I2cError::InternalError)?;
        if state.claimed_peripherals.contains_key(&peripheral) {
            log::error!("{peripheral:?} is already claimed");
            return Err(I2cError::AlreadyClaimed);
        }

        state.claimed_peripherals.insert(peripheral, sender);
        log::debug!("{peripheral:?} is now claimed by PID={sender:}");

        Ok(())
    }
}

impl ArchiveHandler<SingleTransfer> for I2cServer {
    fn handle(
        &mut self,
        transfer: SingleTransfer,
        sender: PID,
        _context: &mut ServerContext<Self>,
    ) -> Result<Vec<u8>, I2cError> {
        let state = unsafe { (&mut *core::ptr::addr_of_mut!(I2C_SERVER_STATE)).as_mut() }
            .ok_or(I2cError::InternalError)?;
        let peripheral = transfer.peripheral;
        let claimed_by = state.claimed_peripherals.get(&peripheral).ok_or(I2cError::PeripheralNotClaimed)?;
        if *claimed_by != sender {
            log::error!("PID={sender:} tried to access {peripheral:?} that's claimed by PID={claimed_by:}");
            return Err(I2cError::AccessDenied);
        }

        log::trace!("I2C transfer: {transfer:02x?}");

        let mut result = vec![0_u8; transfer.read_len as usize];
        state.twi.write_read_bytes(peripheral.i2c_addr(), &transfer.write_data, &mut result).map_err(
            |err| {
                log::error!("I2C error: {err:?}");
                I2cError::InternalError
            },
        )?;
        log::trace!("Result: {result:02x?}");
        Ok(result)
    }
}
