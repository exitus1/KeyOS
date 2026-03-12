use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use atsama5d27::tc::{Tc, TimerInput};
use atsama5d27::wdt::Wdt;
use atsama5d27::{rtc::Rtc, tc::TimerChannel};
use utralib::*;
use xous::arch::irq::IrqNumber;
use xous_api_ticktimer::api::Opcode;

const NS_IN_SEC: u64 = 1000_000_000;

const SLOW_CLOCK_SPEED: u64 = 32768;
// Reset the slow clock counter every 24 hours so it doesn't overflow 32 bit.
const FREE_RUNNING_IRQ_PERIOD: u64 = 24 * 60 * 60 * SLOW_CLOCK_SPEED;

const MINIMUM_TIME_UTC: u32 = 1704112440; // 2024.01.01. 12:34:00

pub struct XousTickTimer {
    /// A timer used for variable-length sleep interrupts
    sleep_timer: Tc,

    /// A free running timer to keep track of the system uptime in milliseconds
    uptime_timer: Tc,

    rtc: Rtc,

    wdt: Wdt,

    rtc_time: AtomicU32,
    pending_rtc_set_time: AtomicU32,
    rtc_time_sampled_at: AtomicU64,

    /// Offset to the system uptime counter, as measured by the slow clock
    uptime_offset: AtomicU64,

    connection: xous::CID,
}

static mut XTT: Option<XousTickTimer> = None;

fn handle_irq(_irq_no: usize, arg: *mut usize) {
    let xtt = unsafe { &*(arg as *const XousTickTimer) };

    // Possibly acknowledge slowclock interrupt, and also update the offset
    if xtt.uptime_timer.period_passed() {
        xtt.uptime_offset.fetch_add(FREE_RUNNING_IRQ_PERIOD, Ordering::SeqCst);
    }

    // This also acknowledge the timer interrupt if that's why we're here
    if xtt.sleep_timer.period_passed() {
        xtt.sleep_timer.stop();

        // Clear flag if it accidentally happened again between the `if` and `stop()`
        xtt.sleep_timer.period_passed();

        // Let the server know we have expired
        xous::try_send_message(
            xtt.connection,
            xous::Message::Scalar(xous::ScalarMessage {
                id: Opcode::TimerInterrupt as usize,
                arg1: 0,
                arg2: 0,
                arg3: 0,
                arg4: 0,
            }),
        )
        .ok();
    }
}

fn handle_irq_rtc(_irq_no: usize, arg: *mut usize) {
    let xtt = unsafe { &mut *(arg as *mut XousTickTimer) };

    let pending_rtc_set_time = xtt.pending_rtc_set_time.load(Ordering::SeqCst);
    let pending_rtc_set_time = if pending_rtc_set_time > 0 { Some(pending_rtc_set_time) } else { None };
    let set_time_happened = xtt.rtc.handle_interrupt(pending_rtc_set_time);
    if set_time_happened {
        xtt.pending_rtc_set_time.store(0, Ordering::SeqCst);
    }

    if let Some(rtc_time) = xtt.rtc.time() {
        xtt.rtc_time.store(rtc_time, Ordering::SeqCst);
        xtt.rtc_time_sampled_at.store(xtt.uptime_slow_cycles(), Ordering::SeqCst);
    }
}

impl XousTickTimer {
    pub fn new(connection: xous::CID) -> &'static XousTickTimer {
        let csr = xous::syscall::map_memory(
            xous::MemoryAddress::new(HW_TC1_BASE),
            None,
            0x1000,
            xous::MemoryFlags::W | xous::MemoryFlags::DEV,
        )
        .expect("couldn't map TC1 CSR range");

        let mut sleep_timer = Tc::with_alt_base_addr(csr.as_ptr() as u32, TimerChannel::Ch0);
        sleep_timer.setup(TimerInput::SlowClock);

        let mut uptime_timer = Tc::with_alt_base_addr(csr.as_ptr() as u32, TimerChannel::Ch1);
        uptime_timer.setup(TimerInput::SlowClock);

        let csr_rtc = xous::syscall::map_memory(
            xous::MemoryAddress::new(HW_RTC_BASE & !(0xfff)),
            None,
            0x1000,
            xous::MemoryFlags::W | xous::MemoryFlags::DEV,
        )
        .expect("couldn't map SYSC (RTC & WDT) CSR range");

        // Both RTC and WDT share the same page (SYSC)
        let rtc = Rtc::with_alt_base_addr(csr_rtc.as_ptr() as u32 + (HW_RTC_BASE & 0xfff) as u32);
        let wdt = Wdt::with_alt_base_addr(csr_rtc.as_ptr() as u32 + (HW_WDT_BASE & 0xfff) as u32);
        wdt.restart();

        let mut rtc_time = rtc.time().unwrap_or_default();
        let mut pending_rtc_set_time = 0;
        if rtc_time < MINIMUM_TIME_UTC {
            rtc_time = MINIMUM_TIME_UTC;
            pending_rtc_set_time = rtc_time;
            log::warn!("Resetting RTC time");
        }

        unsafe {
            (*core::ptr::addr_of_mut!(XTT)).replace(XousTickTimer {
                sleep_timer,
                uptime_timer,
                rtc,
                wdt,
                rtc_time: AtomicU32::new(rtc_time),
                rtc_time_sampled_at: Default::default(),
                pending_rtc_set_time: AtomicU32::new(pending_rtc_set_time),
                uptime_offset: Default::default(),
                connection,
            });
        }

        let xtt = unsafe { (*core::ptr::addr_of_mut!(XTT)).as_ref().expect("xtt") };

        xous::claim_interrupt(IrqNumber::Tc1, handle_irq, xtt as *const XousTickTimer as *mut usize)
            .expect("couldn't claim irq (tc1)");

        xous::claim_interrupt(IrqNumber::Sys, handle_irq_rtc, xtt as *const XousTickTimer as *mut usize)
            .expect("couldn't claim irq (tc1)");

        xtt.uptime_timer.set_period(FREE_RUNNING_IRQ_PERIOD as u32);
        xtt.uptime_timer.set_interrupt(true);
        xtt.uptime_timer.restart();

        xtt.sleep_timer.set_interrupt(true);

        xtt.rtc.enable_interrupts();
        // Needed in edge cases where the software stopped the RTC and then died somehow
        xtt.rtc.start();

        xtt
    }

    fn uptime_slow_cycles(&self) -> u64 {
        self.uptime_offset.load(Ordering::SeqCst) + self.uptime_timer.counter() as u64
    }

    pub fn elapsed_ns(&self) -> u64 { slow_cycles_to_ns(self.uptime_slow_cycles()) }

    pub fn start_sleep(&self, ns: u64) {
        // Sleep for a maximum of 1 hour to prevent overflows.
        // If we wake up and there's nothing to do, we'll go back
        // to sleep.
        let cycles = ns_to_slow_cycles(ns).min(SLOW_CLOCK_SPEED * 3600);
        self.sleep_timer.set_period(cycles as u32);
        // Start the timer
        self.sleep_timer.restart();
    }

    pub fn get_system_time_ns(&self) -> u64 {
        let offset_cycles =
            self.uptime_slow_cycles().saturating_sub(self.rtc_time_sampled_at.load(Ordering::SeqCst));
        self.rtc_time.load(Ordering::SeqCst) as u64 * NS_IN_SEC + slow_cycles_to_ns(offset_cycles)
    }

    pub fn set_system_time_ns(&self, ns: u64) {
        self.pending_rtc_set_time.store((ns / NS_IN_SEC) as u32, Ordering::SeqCst);
    }

    pub fn restart_wdt(&self) { self.wdt.restart(); }
}

pub const fn ns_to_slow_cycles(ns: u64) -> u64 {
    (ns as u128 * SLOW_CLOCK_SPEED as u128 / NS_IN_SEC as u128) as u64
}

pub const fn slow_cycles_to_ns(sc: u64) -> u64 {
    (sc as u128 * NS_IN_SEC as u128 / SLOW_CLOCK_SPEED as u128) as u64
}
