// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundationdevices.com>
// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]

mod reg;

use core::ops::Range;

pub use reg::Gain;
use reg::{Control, InterruptSettings, Register};

const LTR303_ADDRESS: u8 = 0x29;
const EXPECTED_MANUFACTURER: u8 = 0x05;
const EXPECTED_PART_NO: u8 = 0xa0;

#[derive(Debug)]
pub enum Ltr303Error<I2C> {
    I2cError(I2C),
    WrongChipId,
}

#[derive(Debug, Clone, Copy)]
pub struct Measurement {
    /// Light intensity in the visible spectrum, in linear logical units (1 is 1 lux in Gain1X mode)
    pub intensity_visible: u16,
    /// Light intensity in the IR range, in linear logical units
    pub intensity_ir: u16,
}

pub struct Ltr303<I2C> {
    i2c: I2C,
}

impl<I2C: embedded_hal::i2c::I2c> Ltr303<I2C> {
    pub fn new(i2c: I2C) -> Self { Self { i2c } }

    pub fn verify_chip_id(&mut self) -> Result<(), Ltr303Error<I2C::Error>> {
        if self.read_reg(Register::ManufacturerId)? != EXPECTED_MANUFACTURER
            || self.read_reg(Register::PartId)? != EXPECTED_PART_NO
        {
            Err(Ltr303Error::WrongChipId)
        } else {
            Ok(())
        }
    }

    pub fn enable(&mut self, gain: Gain) -> Result<(), Ltr303Error<I2C::Error>> {
        let mut control_command = Control(0);
        control_command.set_active(true);
        control_command.set_gain(gain as u8);
        self.write_reg(Register::Control, control_command.0)
    }

    pub fn disable(&mut self) -> Result<(), Ltr303Error<I2C::Error>> { self.write_reg(Register::Control, 0) }

    pub fn reset(&mut self) -> Result<(), Ltr303Error<I2C::Error>> {
        let mut control_command = Control(0);
        control_command.set_reset(true);
        self.write_reg(Register::Control, control_command.0)
    }

    /// Enables interrupts, active level: High.
    /// Interrupts are sent every time there's a new measurement (subject to threshold filtering, see
    /// [`Ltr303::set_interrupt_threshold`] ), and can be acknowledged by reading the channel values with
    /// [`Ltr303::read()`].
    pub fn enable_interrupts(&mut self) -> Result<(), Ltr303Error<I2C::Error>> {
        let mut interrupt_settings = InterruptSettings(0);
        interrupt_settings.set_enable(true);
        interrupt_settings.set_polarity(true);
        self.write_reg(Register::InterruptSettings, interrupt_settings.0)
    }

    /// Set the threshold interval for interrupts. Measurement interrupts will only be sent if the visible
    /// light measurement goes out of this range.
    pub fn set_interrupt_threshold(&mut self, range: Range<u16>) -> Result<(), Ltr303Error<I2C::Error>> {
        self.write_reg(Register::ThresholdUpperLow, range.end as u8)?;
        self.write_reg(Register::ThresholdUpperHigh, (range.end >> 8) as u8)?;
        self.write_reg(Register::ThresholdLowerLow, range.start as u8)?;
        self.write_reg(Register::ThresholdLowerHigh, (range.start >> 8) as u8)?;
        Ok(())
    }

    pub fn read(&mut self) -> Result<Measurement, Ltr303Error<I2C::Error>> {
        // Registers need to be read in this order, and all of them have to be read,
        // because the hardware latches on the first read and unlatches on the last.
        let intensity_ir =
            self.read_reg(Register::DataCh1Low)? as u16 | (self.read_reg(Register::DataCh1High)? as u16) << 8;
        let intensity_visible =
            self.read_reg(Register::DataCh0Low)? as u16 | (self.read_reg(Register::DataCh0High)? as u16) << 8;
        Ok(Measurement { intensity_visible, intensity_ir })
    }

    fn read_reg(&mut self, reg: Register) -> Result<u8, Ltr303Error<I2C::Error>> {
        let reg_addr = reg as u8;
        let mut buf = [0u8; 1];
        self.i2c.write_read(LTR303_ADDRESS, &[reg_addr], &mut buf).map_err(Ltr303Error::I2cError)?;

        Ok(buf[0])
    }

    fn write_reg(&mut self, registers: Register, data: u8) -> Result<(), Ltr303Error<I2C::Error>> {
        let reg_addr = registers as u8;
        let tx_buf = [reg_addr, data];
        self.i2c.write_read(LTR303_ADDRESS, &tx_buf, &mut []).map_err(Ltr303Error::I2cError)?;

        Ok(())
    }
}
