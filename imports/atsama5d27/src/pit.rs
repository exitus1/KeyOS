pub use utralib::utra::pit::HW_PIT_BASE;
use utralib::{
    utra::pit::{MR_PITEN, MR_PITIEN, MR_PIV, PIIR, PIVR, SR},
    *,
};

pub struct Pit {
    base_addr: u32,
    clock_speed: Option<u32>,
}

pub const PIV_MAX: u32 = 0xfffff; // PIV is 20 bit wide

impl Default for Pit {
    fn default() -> Pit {
        Self::new()
    }
}

impl Pit {
    #[inline]
    pub fn new() -> Self {
        Self {
            base_addr: HW_PIT_BASE as u32,
            clock_speed: None,
        }
    }

    /// Creates PIT instance with a different base address. Used with virtual memory
    #[inline]
    pub fn with_alt_base_addr(base_addr: u32) -> Self {
        Self {
            base_addr,
            clock_speed: None,
        }
    }

    #[inline]
    pub fn set_clock_speed(&mut self, clock_speed: u32) {
        self.clock_speed = Some(clock_speed);
    }

    #[inline]
    pub fn set_interval(&mut self, interval: u32) {
        let mut pit_csr = CSR::new(self.base_addr as *mut u32);
        pit_csr.rmwf(MR_PIV, interval & PIV_MAX);
    }

    #[inline]
    pub fn set_interrupt(&mut self, enabled: bool) {
        let mut pit_csr = CSR::new(self.base_addr as *mut u32);
        pit_csr.rmwf(MR_PITIEN, enabled.into());
    }

    #[inline]
    pub fn set_enabled(&mut self, enabled: bool) {
        let mut pit_csr = CSR::new(self.base_addr as *mut u32);
        pit_csr.rmwf(MR_PITEN, enabled.into());
    }

    /// Reads the current timer values and resets the timer.
    #[inline]
    pub fn reset(&mut self) -> u32 {
        let pit_csr = CSR::new(self.base_addr as *mut u32);
        pit_csr.r(PIVR)
    }

    /// Reads the current timer values but does not reset it.
    #[inline]
    pub fn read(&self) -> u32 {
        let pit_csr = CSR::new(self.base_addr as *mut u32);
        pit_csr.r(PIIR)
    }

    #[inline]
    pub fn status(&self) -> bool {
        let pit_csr = CSR::new(self.base_addr as *mut u32);
        pit_csr.r(SR) != 0
    }

    #[inline]
    pub fn busy_wait_ms(&mut self, curr_clock_speed: u32, ms: u32) {
        self.reset();
        let base = self.read();
        let delay = ((curr_clock_speed / 1000) * ms) / 16;
        let mut current;

        loop {
            current = self.read();
            current = current.saturating_sub(base);

            if current >= delay {
                break;
            }
        }
    }
}

#[cfg(feature = "eh-0")]
impl eh_0::blocking::delay::DelayMs<u8> for Pit {
    fn delay_ms(&mut self, ms: u8) {
        self.busy_wait_ms(
            self.clock_speed.expect("clock speed must be set"),
            ms as u32,
        );
    }
}

#[cfg(feature = "eh-0")]
impl eh_0::blocking::delay::DelayMs<u32> for Pit {
    fn delay_ms(&mut self, ms: u32) {
        self.busy_wait_ms(self.clock_speed.expect("clock speed must be set"), ms);
    }
}

#[cfg(feature = "eh-1")]
impl eh_1::delay::DelayNs for Pit {
    fn delay_ns(&mut self, ns: u32) {
        self.delay_us(ns / 1000);
    }

    fn delay_us(&mut self, us: u32) {
        self.delay_ms(us / 1000);
    }

    fn delay_ms(&mut self, ms: u32) {
        self.busy_wait_ms(self.clock_speed.expect("clock speed must be set"), ms);
    }
}
