// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

//! `WDT` (watchdog timer) driver.

pub use utralib::utra::wdt::HW_WDT_BASE;
use utralib::{
    utra::wdt::{
        CR,
        CR_KEY,
        CR_LOCKMR,
        CR_WDRSTT,
        MR,
        MR_WDD,
        MR_WDDBGHLT,
        MR_WDDIS,
        MR_WDFIEN,
        MR_WDIDLEHLT,
        MR_WDRSTEN,
        MR_WDV,
        SR,
    },
    CSR,
};

const WDT_KEY: u32 = 0xA5;

/// Watchdog timer timeout in ticks (12 bit)
/// `0xfff` is roughly 16 seconds of timeout.
pub const WDT_COUNTER_TICKS: u32 = 0xfff;

#[derive(Debug, Clone)]
pub struct Config {
    pub enable_interrupt: bool,
    pub enable_reset: bool,
    pub debug_halt: bool,
    pub idle_halt: bool,
    pub delta: u32,
    pub counter: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enable_interrupt: true,
            enable_reset: true,
            debug_halt: true,
            idle_halt: true,
            delta: WDT_COUNTER_TICKS,
            counter: WDT_COUNTER_TICKS,
        }
    }
}

impl Config {
    pub fn with_debug_halt(self, debug_halt: bool) -> Self {
        Self { debug_halt, ..self }
    }

    pub fn with_interrupt(self, enable_interrupt: bool) -> Self {
        Self {
            enable_interrupt,
            ..self
        }
    }
}

pub struct Wdt {
    base_addr: u32,
}

impl Default for Wdt {
    fn default() -> Wdt {
        Self::new()
    }
}

impl Wdt {
    #[inline]
    pub fn new() -> Self {
        Self {
            base_addr: HW_WDT_BASE as u32,
        }
    }

    /// Creates a WDT instance with a different base address. Used with virtual memory
    #[inline]
    pub fn with_alt_base_addr(base_addr: u32) -> Self {
        Self { base_addr }
    }

    /// Enables the watchdog timer with specified configuration
    pub fn enable(&mut self, config: &Config) {
        let mut csr = CSR::new(self.base_addr as *mut u32);

        let mut mode = 0;

        mode |= csr.ms(MR_WDFIEN, config.enable_interrupt as u32);
        mode |= csr.ms(MR_WDRSTEN, config.enable_reset as u32);
        mode |= csr.ms(MR_WDDBGHLT, config.debug_halt as u32);
        mode |= csr.ms(MR_WDIDLEHLT, config.idle_halt as u32);

        mode |= csr.ms(MR_WDD, config.delta & 0xFFF);
        mode |= csr.ms(MR_WDV, config.counter & 0xFFF);

        // The `WDT_MR` register values must not be modified within *three slow clock* periods
        // following a restart of the watchdog performed by write access in WDT_MR. Any
        // modification will cause the watchdog to trigger an end of a period earlier than
        // expected
        wait_sclk_cycles(10);

        csr.wo(MR, mode);
    }

    /// Disables the watchdog timer.
    #[inline]
    pub fn disable(&self) {
        // The `WDT_MR` register values must not be modified within *three slow clock* periods
        // following a restart of the watchdog performed by write access in WDT_MR. Any
        // modification will cause the watchdog to trigger an end of a period earlier than
        // expected
        wait_sclk_cycles(10);

        let mut csr = CSR::new(self.base_addr as *mut u32);
        csr.wfo(MR_WDDIS, 1);
    }

    /// Restarts (feeds) the watchdog timer.
    #[inline]
    pub fn restart(&self) {
        let mut wdt_csr = CSR::new(self.base_addr as *mut u32);

        // The `WDT_CR` register values must not be modified within *three slow clock* periods
        // following a restart of the watchdog performed by write access in WDT_CR. Any
        // modification will cause the watchdog to trigger an end of a period earlier than
        // expected
        wait_sclk_cycles(10);

        let reg = wdt_csr.ms(CR_WDRSTT, 1) | wdt_csr.ms(CR_KEY, WDT_KEY);
        wdt_csr.wo(CR, reg);
    }

    /// Locks the watchdog's mode register (`MR`) from modification until a system reset.
    #[inline]
    pub fn lock_mr(&mut self) {
        let mut wdt_csr = CSR::new(self.base_addr as *mut u32);

        // The `WDT_CR` register values must not be modified within *three slow clock* periods
        // following a restart of the watchdog performed by write access in WDT_CR. Any
        // modification will cause the watchdog to trigger an end of a period earlier than
        // expected
        wait_sclk_cycles(10);

        let reg = wdt_csr.ms(CR_LOCKMR, 1) | wdt_csr.ms(CR_KEY, WDT_KEY);
        wdt_csr.wo(CR, reg);
    }

    /// Returns the current status of the watchdog timer.
    #[inline]
    pub fn status(&self) -> u32 {
        let csr = CSR::new(self.base_addr as *mut u32);
        csr.r(SR)
    }
}

#[inline]
fn wait_sclk_cycles(num_cycles: u32) {
    const CPU_HZ: u32 = 500_000_000;
    const SCLK_FREQ_HZ: u32 = 32_768;
    const CYCLES_PER_LOOP: u32 = 6;

    for _ in 0..(num_cycles * CPU_HZ) / (SCLK_FREQ_HZ * CYCLES_PER_LOOP) {
        armv7::asm::nop();
    }
}
