//! Shutdown Controller (SHDWC)

use {
    bitflags::bitflags,
    utralib::{utra::shdwc::*, HW_SHDWC_BASE, *},
};

const SHDWC_MR_KEY_PASSWD: u32 = 0xA5 << 24;
const WKUP_ENABLE: u32 = 1;
const WKUPT_HIGH: u32 = 1;
const WKUPT_LOW: u32 = 0;

#[derive(Debug, Copy, Clone)]
pub enum DebouncePeriod {
    Immediate = 0,
    Sclk3 = 1,
    Sclk32 = 2,
    Sclk512 = 3,
    Sclk4096 = 4,
    Sclk32768 = 5,
}

bitflags! {
    #[derive(Debug, Copy, Clone)]
    pub struct Status: u32 {
        /// `WKUPS`: PIOBU, WKUP Wakeup Status
        ///
        /// - 0 (NO): No wakeup due to the assertion of the PIOBU, WKUP pins has occurred since the last read of SHDW_SR.
        /// - 1 (PRESENT): At least one wakeup due to the assertion of the PIOBU, WKUP pins has occurred since the last read of SHDW_SR.
        /// Note:
        /// WKUPIS1 reports the status of the Security Module event.
        const WKUPS = 1 << 0;

        /// `RTCWK` RTC Controller Wakeup
        const RTCWK = 1 << 5;

        /// `ACCWK` Analog Comparator Controller Wakeup
        ///
        /// - 0: No wakeup alarm from the ACC occurred since the last read of SHDW_SR.
        /// - 1: At least one wakeup alarm from the ACC occurred since the last read of SHDW_SR.
        const ACCWK = 1 << 6;

        /// `RXLPWK`: Debug Unit Wakeup
        ///
        /// - 0: No wakeup alarm from the Backup RX UART Comparison unit (RXLP) occurred since the last read of SHDW_SR.
        /// - 1: At least one wakeup alarm from the Backup RX UART Comparison unit (RXLP) occurred since the last read of SHDW_SR.
        const RXLPWK = 1 << 7;

        /// `WKUPISx`: Wakeup 0 to 9 Input Status
        ///
        /// - 0 (DISABLE): The corresponding wakeup input is disabled, or was inactive at the time the debouncer triggered a wakeup event.
        /// - 1 (ENABLE): The corresponding wakeup input was active at the time the debouncer triggered a wakeup event.
        const WKUPIS0 = 1 << 16;
        const WKUPIS1 = 1 << 17;
        const WKUPIS2 = 1 << 18;
        const WKUPIS3 = 1 << 19;
        const WKUPIS4 = 1 << 20;
        const WKUPIS5 = 1 << 21;
        const WKUPIS6 = 1 << 22;
        const WKUPIS7 = 1 << 23;
        const WKUPIS8 = 1 << 24;
        const WKUPIS9 = 1 << 25;
    }
}

bitflags! {
    #[derive(Debug, Copy, Clone)]
    pub struct WakeupOptions: u32 {
        const WKUP0 = 1 << 0;
        const RXLP = 1 << 1;
    }
}

pub struct Shdwc {
    base_addr: u32,
}

impl Default for Shdwc {
    fn default() -> Self {
        Shdwc::new()
    }
}

impl Shdwc {
    #[inline]
    pub fn new() -> Self {
        Self {
            base_addr: HW_SHDWC_BASE as u32,
        }
    }

    /// Creates `SHDWC` instance with a different base address. Used with virtual memory
    #[inline]
    pub fn with_alt_base_addr(base_addr: u32) -> Self {
        Self { base_addr }
    }

    /// Configures wakeup signals, then shuts the CPU down and asserts the `SHDN` PMIC
    /// signal to power off the rest of the system.
    #[inline]
    pub fn do_shutdown(&self, options: WakeupOptions) -> ! {
        let mut shdwc_csr = CSR::new(self.base_addr as *mut u32);

        // Select wake-up sources

        // Mandatory:
        // - Secumod alerts (WKUP1)
        // Optional:
        // - Power button (WKUP0)
        // - RXLP

        let mut wuir =
            shdwc_csr.ms(WUIR_WKUPEN1, WKUP_ENABLE) | shdwc_csr.ms(WUIR_WKUPT1, WKUPT_HIGH);
        if options.contains(WakeupOptions::WKUP0) {
            wuir |= shdwc_csr.ms(WUIR_WKUPEN0, WKUP_ENABLE) | shdwc_csr.ms(WUIR_WKUPT0, WKUPT_LOW);
        }
        shdwc_csr.wo(WUIR, wuir);

        // Debounce the power button, optionally wake up on RXLP, and turn off wakeup to RTC, ACC.
        let mr = shdwc_csr.ms(MR_RXLPWKEN, options.contains(WakeupOptions::RXLP) as u32)
            | shdwc_csr.ms(MR_WKUPDBC, DebouncePeriod::Sclk512 as u32)
            | shdwc_csr.ms(MR_ACCWKEN, 0)
            | shdwc_csr.ms(MR_RTCWKEN, 0);
        shdwc_csr.wo(MR, mr);

        // Actual shutdown.
        let reg = SHDWC_MR_KEY_PASSWD | shdwc_csr.ms(CR_SHDW, 1);
        shdwc_csr.wo(CR, reg);

        #[allow(clippy::empty_loop)]
        loop {}
    }

    /// Get the status of the SHDWC.
    /// Has the effect to clear status.
    #[inline]
    pub fn status(&self) -> Status {
        let shdwc_csr = CSR::new(self.base_addr as *mut u32);
        Status::from_bits_retain(shdwc_csr.r(SR))
    }
}
