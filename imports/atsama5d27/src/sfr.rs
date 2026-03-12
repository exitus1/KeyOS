//! Special function registers (SFR).

pub use utralib::HW_SFR_BASE;
use utralib::{utra::sfr::*, *};

pub struct Sfr {
    base_addr: u32,
}

impl Default for Sfr {
    fn default() -> Self {
        Sfr::new()
    }
}

impl Sfr {
    #[inline]
    pub fn new() -> Self {
        Self {
            base_addr: HW_SFR_BASE as u32,
        }
    }

    /// Creates SFR instance with a different base address. Used with virtual memory
    #[inline]
    pub fn with_alt_base_addr(base_addr: u32) -> Self {
        Self { base_addr }
    }

    #[inline]
    pub fn set_l2_cache_sram_enabled(&mut self, enabled: bool) {
        let mut sfr_csr = CSR::new(self.base_addr as *mut u32);
        sfr_csr.rmwf(SFR_L2CC_HRAMC_SRAM_SEL, enabled as u32);
    }

    #[inline]
    pub fn l2_cache_sram_enabled(&self) -> bool {
        let sfr_csr = CSR::new(self.base_addr as *mut u32);
        sfr_csr.rf(SFR_L2CC_HRAMC_SRAM_SEL) != 0
    }

    /// Returns the 64-bit unique serial number of the chip.
    #[inline]
    pub fn serial_number(&self) -> u64 {
        let sfr_csr = CSR::new(self.base_addr as *mut u32);
        let sn0 = sfr_csr.r(SFR_SN0);
        let sn1 = sfr_csr.r(SFR_SN1);

        ((sn1 as u64) << 32) | (sn0 as u64)
    }
}
