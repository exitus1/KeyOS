// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundationdevices.com>
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Low-power UART RX (`RXLP`).

use utralib::{
    utra::rxlp::{BRGR_CD, CMPR, CMPR_VAL1, CMPR_VAL2, CR_RSTRX, CR_RXDIS, CR_RXEN, MR_PAR, RHR},
    CSR,
    HW_RXLP_BASE,
};

#[derive(Copy, Clone, Debug)]
pub enum Parity {
    Even = 0,
    Odd = 1,
    Space = 2,
    Mark = 3,
    No = 4,
}

pub struct Rxlp {
    base_addr: u32,
}

impl Default for Rxlp {
    fn default() -> Self {
        Rxlp::new()
    }
}

impl Rxlp {
    #[inline]
    pub fn new() -> Self {
        Self {
            base_addr: HW_RXLP_BASE as u32,
        }
    }

    /// Creates `RXLP` instance with a different base address. Used with virtual memory
    #[inline]
    pub fn with_alt_base_addr(base_addr: u32) -> Self {
        Self { base_addr }
    }

    #[inline]
    pub fn init(&mut self, cd: u32, parity: Parity) {
        let mut csr = CSR::new(self.base_addr as *mut u32);

        // Reset everything
        csr.wfo(CR_RSTRX, 1);
        csr.wfo(CR_RXDIS, 1);

        // Set baud rate coefficient and parity
        // Actual baudrate is 32768 / (16 * cd) baud. Min value if cd is 1, max is 3
        csr.wfo(BRGR_CD, cd);
        csr.rmwf(MR_PAR, parity as u32);

        // Enable receiver
        self.set_enable(true);
    }

    #[inline]
    pub fn set_enable(&mut self, enable: bool) {
        let mut csr = CSR::new(self.base_addr as *mut u32);
        if enable {
            csr.wfo(CR_RXEN, 1);
        } else {
            csr.wfo(CR_RXDIS, 1);
        }
    }

    /// Makes `RXLP` generate a wake-up signal if the received character (`c`) is within
    /// the range of `val1 <= c <= val2`.
    #[inline]
    pub fn set_comparison(&mut self, val1: u8, val2: u8) {
        let mut csr = CSR::new(self.base_addr as *mut u32);

        let cmpr = csr.ms(CMPR_VAL1, val1 as u32) | csr.ms(CMPR_VAL2, val2 as u32);
        csr.wo(CMPR, cmpr);
    }

    /// Returns the received character.
    #[inline]
    pub fn read(&mut self) -> u8 {
        let csr = CSR::new(self.base_addr as *mut u32);
        let val = csr.r(RHR);
        val as u8
    }
}
