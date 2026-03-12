//! Slow Clock (SCKC) module.

use utralib::{utra::sckc::SCKC_CR_OSCSEL, *};

pub struct Sckc {
    base_addr: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SclkType {
    RcOscillator = 0,
    Crystal = 1,
}

impl Default for Sckc {
    fn default() -> Self {
        Sckc {
            base_addr: HW_SCKC_BASE as u32,
        }
    }
}

impl Sckc {
    #[inline]
    pub fn with_alt_base_addr(base_addr: u32) -> Self {
        Sckc { base_addr }
    }

    #[inline]
    pub fn selected_clock(&self) -> SclkType {
        let csr = CSR::new(self.base_addr as *mut u32);
        if csr.rf(SCKC_CR_OSCSEL) == 0 {
            SclkType::RcOscillator
        } else {
            SclkType::Crystal
        }
    }

    #[inline]
    pub fn select_clock(&mut self, clock: SclkType) {
        let mut csr = CSR::new(self.base_addr as *mut u32);
        csr.wfo(SCKC_CR_OSCSEL, clock as _)
    }
}
