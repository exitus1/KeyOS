//! Timer-Counter (TC) implementation.

use {
    utra::tc0::{
        CCR0,
        CCR0_CLKDIS,
        CCR0_CLKEN,
        CCR0_SWTRG,
        CMR0,
        CV0,
        IDR0_CPCS,
        IER0_CPCS,
        RC0,
        SR0_CPCS,
    },
    utralib::*,
};

const TC_CMR_WAVEFORM_WAVSEL_UP_RC: u32 = 0x02; // UP mode with automatic trigger on RC Compare Position
const TC_CMR_WAVEFORM_WAVSEL_POS: u32 = 13;
const TC_CMR_WAVEFORM_MODE_MSK: u32 = 1 << 15;

pub struct Tc {
    /// Base address for the timer channel. Note that non-channel registers such as
    /// Clock-control will only be correct with Ch0
    base_addr: u32,
}

pub enum TimerChannel {
    Ch0,
    Ch1,
    Ch2,
}

pub enum TimerInput {
    Gclk = 0,
    SystemBusDiv8 = 1,
    SystemBusDiv32 = 2,
    SystemBusDiv128 = 3,
    SlowClock = 4,
    Xc0 = 5,
    Xc1 = 6,
    Xc2 = 7,
}

impl Default for Tc {
    fn default() -> Tc {
        Self::new(TimerChannel::Ch0)
    }
}

impl Tc {
    #[inline]
    pub fn new(channel: TimerChannel) -> Self {
        Self {
            base_addr: HW_TC0_BASE as u32 + channel.offset(),
        }
    }

    /// Creates TC instance with a different base address. Used with virtual memory
    #[inline]
    pub fn with_alt_base_addr(base_addr: u32, channel: TimerChannel) -> Self {
        Self {
            base_addr: base_addr + channel.offset(),
        }
    }

    #[inline]
    pub fn setup(&mut self, input: TimerInput) {
        let mut tc_csr = CSR::new(self.base_addr as *mut u32);

        let cmr0 = input as u32
            | (TC_CMR_WAVEFORM_WAVSEL_UP_RC << TC_CMR_WAVEFORM_WAVSEL_POS)
            | TC_CMR_WAVEFORM_MODE_MSK;
        tc_csr.wo(CMR0, cmr0);
    }

    #[inline]
    pub fn restart(&self) {
        let mut tc_csr = CSR::new(self.base_addr as *mut u32);

        let ccr0 = tc_csr.ms(CCR0_CLKEN, 1) | tc_csr.ms(CCR0_SWTRG, 1);
        tc_csr.wo(CCR0, ccr0);
    }

    #[inline]
    pub fn stop(&self) {
        let mut tc_csr = CSR::new(self.base_addr as *mut u32);
        tc_csr.wfo(CCR0_CLKDIS, 1);
    }

    #[inline]
    pub fn period(&self) -> u32 {
        let tc_csr = CSR::new(self.base_addr as *mut u32);
        tc_csr.r(RC0)
    }

    #[inline]
    pub fn set_period(&self, period: u32) {
        let mut tc_csr = CSR::new(self.base_addr as *mut u32);
        tc_csr.wo(RC0, period);
    }

    #[inline]
    pub fn set_interrupt(&self, enable: bool) {
        let mut tc_csr = CSR::new(self.base_addr as *mut u32);
        if enable {
            tc_csr.wfo(IER0_CPCS, 1);
        } else {
            tc_csr.wfo(IDR0_CPCS, 1);
        }
    }

    #[inline]
    pub fn period_passed(&self) -> bool {
        let tc_csr = CSR::new(self.base_addr as *mut u32);
        tc_csr.rf(SR0_CPCS) != 0
    }

    #[inline]
    pub fn counter(&self) -> u32 {
        let tc_csr = CSR::new(self.base_addr as *mut u32);
        tc_csr.r(CV0)
    }
}

impl TimerChannel {
    #[inline]
    fn offset(&self) -> u32 {
        match self {
            TimerChannel::Ch0 => 0,
            TimerChannel::Ch1 => 0x40,
            TimerChannel::Ch2 => 0x80,
        }
    }
}
