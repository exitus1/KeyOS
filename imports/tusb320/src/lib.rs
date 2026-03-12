// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundationdevices.com>
// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]

mod reg;

use reg::DebounceModeSelectReset;
pub use reg::{Registers, ModeSelect, AttachedState};

const TUSB320_ADDRESS: u8 = 0x61;
const TUSB320_CHIP_ID: [u8; 8] = [0x30, 0x32, 0x33, 0x42, 0x53, 0x55, 0x54, 0x00];

#[derive(Debug)]
pub enum TUsb320Error<I2C> {
    I2cError(I2C),
}

pub struct Tusb320<I2C> {
    i2c: I2C,
}

impl<I2C: embedded_hal::i2c::I2c> Tusb320<I2C> {
    pub fn new(i2c: I2C) -> Self { Self { i2c } }

    pub fn verify_chip_id(&mut self) -> Result<bool, TUsb320Error<I2C::Error>> {
        let mut chip_id_bytes = [0u8; 8];
        for i in 0..8 {
            chip_id_bytes[i] = self.read_reg_raw(i as u8)?;
        }
        Ok(chip_id_bytes == TUSB320_CHIP_ID)
    }

    pub fn set_mode_select(&mut self, mode: ModeSelect) -> Result<(), TUsb320Error<I2C::Error>> {
        let reg = self.read_reg(Registers::DebounceModeSelectReset)?;
        let mut debounce = DebounceModeSelectReset(reg);
        debounce.set_mode_select(mode as u8);
        self.write_reg(Registers::DebounceModeSelectReset, debounce.0)?;

        Ok(())
    }
    
    pub fn mode_select(&mut self) -> Result<ModeSelect, TUsb320Error<I2C::Error>> {
        let reg = self.read_reg(Registers::DebounceModeSelectReset)?;
        let debounce = DebounceModeSelectReset(reg);
        Ok(ModeSelect::from(debounce.mode_select()))
    }

    pub fn soft_reset(&mut self) -> Result<(), TUsb320Error<I2C::Error>> {
        let reg = self.read_reg(Registers::DebounceModeSelectReset)?;
        let mut debounce = DebounceModeSelectReset(reg);
        debounce.set_i2c_soft_reset(true);
        self.write_reg(Registers::DebounceModeSelectReset, debounce.0)?;

        Ok(())
    }
    
    pub fn attached_state(&mut self) -> Result<AttachedState, TUsb320Error<I2C::Error>> {
        let reg = self.read_reg(Registers::StateDirInterruptStatus)?;
        let state_dir_interrupt = reg::StateDirInterruptStatus(reg);
        Ok(AttachedState::from(state_dir_interrupt.attached_state()))
    }

    pub fn clear_interrupt(&mut self) -> Result<(), TUsb320Error<I2C::Error>> {
        let mut state_dir_interrupt = reg::StateDirInterruptStatus(0);
        state_dir_interrupt.set_interrupt_status(true);
        self.write_reg(Registers::StateDirInterruptStatus, state_dir_interrupt.0)?;
        Ok(())
    }

    fn read_reg_raw(&mut self, reg: u8) -> Result<u8, TUsb320Error<I2C::Error>> {
        let mut buf = [0u8; 1];
        self.i2c.write_read(TUSB320_ADDRESS, &[reg], &mut buf).map_err(TUsb320Error::I2cError)?;

        Ok(buf[0])
    }
    
    fn read_reg(&mut self, reg: Registers) -> Result<u8, TUsb320Error<I2C::Error>> {
        Ok(self.read_reg_raw(reg as u8)?)
    }

    fn write_reg(&mut self, reg: Registers, data: u8) -> Result<(), TUsb320Error<I2C::Error>> {
        let tx_buf = [reg as u8, data];
        self.i2c.write_read(TUSB320_ADDRESS, &tx_buf, &mut []).map_err(TUsb320Error::I2cError)?;

        Ok(())
    }
}