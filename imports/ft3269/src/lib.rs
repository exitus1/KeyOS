#![no_std]

pub use crate::reg::{Dimensions, PowerMode, Status, Touch, TouchKind};
use crate::reg::{Register, NUM_REGS_PER_TOUCH};

mod reg;

const FT3269_I2C_ADDRESS: u8 = 0x38;
const FT3269_CHIP_ID: u8 = 0x79;

#[derive(Debug)]
pub enum Ft3269Error<I2C> {
    I2cError(I2C),
}

pub struct Ft3269<I2C> {
    i2c: I2C,
}

impl<I2C: embedded_hal::i2c::I2c> Ft3269<I2C> {
    pub fn new(i2c: I2C) -> Self { Self { i2c } }

    pub fn verify(&mut self) -> Result<bool, Ft3269Error<I2C::Error>> {
        Ok(self.chip_id()? == FT3269_CHIP_ID)
    }

    pub fn chip_id(&mut self) -> Result<u8, Ft3269Error<I2C::Error>> { self.read_reg_u8(Register::RegId) }

    pub fn status(&mut self) -> Result<Status, Ft3269Error<I2C::Error>> {
        Ok(self.read_reg_u8(Register::Status)?.into())
    }

    /// Fills the provided buffer of `Touch`-es and returns the number of simultaneous touches registered by
    /// the controller.
    pub fn touches(&mut self, touches: &mut [Touch; 5]) -> Result<(), Ft3269Error<I2C::Error>> {
        let mut buf = [0u8; 5 * NUM_REGS_PER_TOUCH as usize];
        self.i2c
            .write_read(FT3269_I2C_ADDRESS, &[Register::Touch1XH as u8], &mut buf)
            .map_err(Ft3269Error::I2cError)?;
        for (touch, data) in touches.iter_mut().zip(buf.chunks(NUM_REGS_PER_TOUCH as usize)) {
            *touch = Touch::from_data(&data)
        }

        Ok(())
    }

    pub fn dimensions(&mut self) -> Result<Dimensions, Ft3269Error<I2C::Error>> {
        // TODO: optimize with burst read (needs fixing TWI RX FIFO issues)
        let xh = self.read_reg_u8(Register::MaxXH)?;
        let xl = self.read_reg_u8(Register::MaxXL)?;
        let yh = self.read_reg_u8(Register::MaxYH)?;
        let yl = self.read_reg_u8(Register::MaxYL)?;

        Ok(Dimensions::from_data(xh, xl, yh, yl))
    }

    pub fn set_dimensions(&mut self, dimensions: &Dimensions) -> Result<(), Ft3269Error<I2C::Error>> {
        // TODO: optimize with burst write
        let (xh, xl, yh, yl) = dimensions.to_data();
        self.write_reg_u8(Register::MaxXH, xh)?;
        self.write_reg_u8(Register::MaxXL, xl)?;
        self.write_reg_u8(Register::MaxYH, yh)?;
        self.write_reg_u8(Register::MaxYL, yl)?;

        Ok(())
    }

    pub fn set_power_mode(&mut self, mode: PowerMode) -> Result<(), Ft3269Error<I2C::Error>> {
        self.write_reg_u8(Register::PowerMode, mode as u8)
    }

    fn read_reg_u8(&mut self, reg: Register) -> Result<u8, Ft3269Error<I2C::Error>> {
        self.read_reg_u8_by_addr(reg.into())
    }

    pub fn read_reg_u8_by_addr(&mut self, addr: u8) -> Result<u8, Ft3269Error<I2C::Error>> {
        let bytes = [addr];
        let mut buf = [0];

        self.i2c.write_read(FT3269_I2C_ADDRESS, &bytes, &mut buf).map_err(Ft3269Error::I2cError)?;

        Ok(buf[0])
    }

    pub fn dump_regs(&mut self, console: &mut impl core::fmt::Write) {
        write!(console, "     ").ok();
        for i in 0..16 {
            write!(console, "{:02X} ", i).ok();
        }
        writeln!(console).ok();
        writeln!(console, "----------------------------------------------------").ok();
        for reg in 0..=0xff_u8 {
            if reg % 16 == 0 {
                write!(console, "{:02X} | ", (reg / 16) << 4).ok();
            }

            match self.read_reg_u8_by_addr(reg) {
                Ok(val) => {
                    if (reg.saturating_add(1)) % 16 == 0 {
                        writeln!(console, "{:02x}", val).ok();
                    } else {
                        write!(console, "{:02x} ", val).ok();
                    }
                }

                Err(_) => {
                    if (reg.saturating_add(1)) % 16 == 0 {
                        writeln!(console, "??").ok();
                    } else {
                        write!(console, "?? ").ok();
                    }
                }
            }
        }
    }

    fn write_reg_u8(&mut self, reg: Register, val: u8) -> Result<(), Ft3269Error<I2C::Error>> {
        self.write_reg_u8_by_addr(reg.into(), val)
    }

    fn write_reg_u8_by_addr(&mut self, addr: u8, val: u8) -> Result<(), Ft3269Error<I2C::Error>> {
        self.i2c.write(FT3269_I2C_ADDRESS, &[addr, val]).map_err(Ft3269Error::I2cError)?;

        Ok(())
    }
}
