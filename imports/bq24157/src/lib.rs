// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundationdevices.com>
// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]

mod reg;

pub use reg::{
    BatteryVoltage,
    BoostFault,
    ChargeFault,
    Control,
    Registers,
    Revision,
    SafetyLimit,
    SpecialChargerVoltage,
    Status,
    SAFETY_V_CURR_SENSE_DEFAULT,
};

use crate::reg::{ChargerCurrent, PN_BQ24157};

const BQ24157_ADDRESS: u8 = 0x6A;

#[derive(Debug)]
pub enum Bq24517Error<I2C> {
    I2cError(I2C),
}

pub struct Bq24157<I2C> {
    i2c: I2C,
}

impl<I2C: embedded_hal::i2c::I2c> Bq24157<I2C> {
    pub fn new(i2c: I2C) -> Self {
        Self { i2c }
    }

    pub fn verify_chip_id(&mut self) -> Result<bool, Bq24517Error<I2C::Error>> {
        let rev_reg = self.read_reg(Registers::Revision)?;
        Ok(Revision(rev_reg).pn() == PN_BQ24157)
    }

    pub fn status(&mut self) -> Result<Status, Bq24517Error<I2C::Error>> {
        Ok(Status(self.read_reg(Registers::Status)?))
    }

    pub fn safety_limits(&mut self) -> Result<SafetyLimit, Bq24517Error<I2C::Error>> {
        Ok(SafetyLimit(self.read_reg(Registers::SafetyLimit)?))
    }

    pub fn set_safety_limits(
        &mut self,
        safety_limit: SafetyLimit,
    ) -> Result<(), Bq24517Error<I2C::Error>> {
        self.write_reg(Registers::SafetyLimit, safety_limit.0)?;
        Ok(())
    }

    pub fn control(&mut self) -> Result<Control, Bq24517Error<I2C::Error>> {
        Ok(Control(self.read_reg(Registers::Control)?))
    }

    pub fn set_control(&mut self, control: Control) -> Result<(), Bq24517Error<I2C::Error>> {
        self.write_reg(Registers::Control, control.0)
    }

    pub fn batt_voltage(&mut self) -> Result<BatteryVoltage, Bq24517Error<I2C::Error>> {
        Ok(BatteryVoltage(self.read_reg(Registers::BatteryVoltage)?))
    }

    pub fn set_batt_voltage(
        &mut self,
        voltage: BatteryVoltage,
    ) -> Result<(), Bq24517Error<I2C::Error>> {
        self.write_reg(Registers::BatteryVoltage, voltage.0)
    }

    pub fn special_charger_voltage(
        &mut self,
    ) -> Result<SpecialChargerVoltage, Bq24517Error<I2C::Error>> {
        Ok(SpecialChargerVoltage(
            self.read_reg(Registers::SpecialChargerVoltage)?,
        ))
    }

    pub fn set_special_charger_voltage(
        &mut self,
        voltage: SpecialChargerVoltage,
    ) -> Result<(), Bq24517Error<I2C::Error>> {
        self.write_reg(Registers::SpecialChargerVoltage, voltage.0)
    }

    pub fn charger_current(&mut self) -> Result<ChargerCurrent, Bq24517Error<I2C::Error>> {
        Ok(ChargerCurrent(self.read_reg(Registers::ChargerCurrent)?))
    }

    pub fn set_charge_current(
        &mut self,
        current: ChargerCurrent,
    ) -> Result<(), Bq24517Error<I2C::Error>> {
        self.write_reg(Registers::ChargerCurrent, current.0)
    }

    pub fn reset_charger(&mut self) -> Result<(), Bq24517Error<I2C::Error>> {
        let mut charger_current = self.charger_current()?;
        charger_current.set_reset(true);
        self.write_reg(Registers::ChargerCurrent, charger_current.0)
    }

    pub fn apply_register_dump(
        &mut self,
        dump: &[(u8, u8)],
    ) -> Result<(), Bq24517Error<I2C::Error>> {
        for &(reg, value) in dump {
            self.write_reg_raw(reg, value)?;
        }
        Ok(())
    }

    fn read_reg(&mut self, reg: Registers) -> Result<u8, Bq24517Error<I2C::Error>> {
        let reg_addr = reg as u8;
        let mut buf = [0u8; 1];
        self.i2c
            .write_read(BQ24157_ADDRESS, &[reg_addr], &mut buf)
            .map_err(Bq24517Error::I2cError)?;

        Ok(buf[0])
    }

    fn write_reg(
        &mut self,
        registers: Registers,
        data: u8,
    ) -> Result<(), Bq24517Error<I2C::Error>> {
        self.write_reg_raw(registers as u8, data)?;
        Ok(())
    }

    fn write_reg_raw(&mut self, reg: u8, data: u8) -> Result<(), Bq24517Error<I2C::Error>> {
        let tx_buf = [reg, data];
        self.i2c
            .write_read(BQ24157_ADDRESS, &tx_buf, &mut [])
            .map_err(Bq24517Error::I2cError)?;

        Ok(())
    }
}
