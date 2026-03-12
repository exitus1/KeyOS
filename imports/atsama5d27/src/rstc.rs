//! Reset Controller (RSTC)

use utralib::{utra::rstc::*, HW_RSTC_BASE, *};

const RSTC_MR_KEY_PASSWD: u32 = 0xA5 << 24;

#[derive(Debug, Copy, Clone)]
pub enum ResetCause {
    /// Both VDDCORE and VDDBU rising
    General = 0,
    /// VDDCORE rising
    Wkup = 1,
    /// Watchdog fault occurred
    Wdt = 2,
    /// Processor reset required by the software
    Software = 3,
    /// NRST pin detected low
    User = 4,

    #[doc(hidden)]
    Reserved5 = 5,
    #[doc(hidden)]
    Reserved6 = 6,

    /// 32.768 kHz Crystal Oscillator Failure Detection Reset
    SlckXtal = 7,

    /// Unknown reset cause
    #[doc(hidden)]
    Unknown,
}

pub struct Rstc {
    base_addr: u32,
}

impl Default for Rstc {
    fn default() -> Self {
        Rstc::new()
    }
}

impl Rstc {
    #[inline]
    pub fn new() -> Self {
        Self {
            base_addr: HW_RSTC_BASE as u32,
        }
    }

    /// Creates RSTC instance with a different base address. Used with virtual memory
    #[inline]
    pub fn with_alt_base_addr(base_addr: u32) -> Self {
        Self { base_addr }
    }

    /// Resets the CPU and peripherals but not the backup area.
    #[inline]
    pub fn do_reset(&self) {
        let mut rstc_csr = CSR::new(self.base_addr as *mut u32);
        const RSTC_CR_PROCRST_MSK: u32 = 1;
        rstc_csr.wo(CR, RSTC_MR_KEY_PASSWD | RSTC_CR_PROCRST_MSK);
    }

    #[inline]
    pub fn reset_cause(&self) -> ResetCause {
        let rstc_csr = CSR::new(self.base_addr as *mut u32);
        let cause = rstc_csr.rf(SR_RSTTYP);
        match cause {
            0 => ResetCause::General,
            1 => ResetCause::Wkup,
            2 => ResetCause::Wdt,
            3 => ResetCause::Software,
            4 => ResetCause::User,
            5 => ResetCause::Reserved5,
            6 => ResetCause::Reserved6,
            7 => ResetCause::SlckXtal,

            _ => ResetCause::Unknown,
        }
    }
}
