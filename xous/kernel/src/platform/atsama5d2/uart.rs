// SPDX-FileCopyrightText: 2022 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: Apache-2.0

use atsama5d27::uart::{Uart as UartHw, Uart1};
use keyos::UART_ADDR;
use xous::arch::irq::IrqNumber;

use crate::io::{SerialRead, SerialWrite};

pub type UartType = UartHw<Uart1>;
const UART_IRQ_NUM: IrqNumber = IrqNumber::Uart1;

/// UART instance.
///
/// Initialized by [`init`].
pub static mut UART: Option<Uart> = None;

/// UART peripheral driver.
pub struct Uart {
    uart_csr: UartType,
    callback: fn(&mut Self),
}

impl Uart {
    pub fn new(addr: usize, callback: fn(&mut Self)) -> Uart {
        Uart { uart_csr: UartHw::with_alt_base_addr(addr), callback }
    }

    pub fn init(&mut self) {
        // no-op, it should be already initialized by the 2nd stage bootloader
    }

    pub fn irq(_irq_number: usize, arg: *mut usize) {
        let uart = unsafe { &mut *(arg as *mut Uart) };
        (uart.callback)(uart);
    }

    pub fn enable_rx_irq(&mut self) {
        self.uart_csr.set_rx_interrupt(true);
        self.uart_csr.set_rx(true);
    }
}

impl SerialWrite for Uart {
    fn putc(&mut self, c: u8) { self.uart_csr.write_byte(c); }
}

impl SerialRead for Uart {
    fn getc(&mut self) -> Option<u8> { self.uart_csr.getc_nonblocking() }
}

/// Initialize UART driver and debug shell.
pub fn init() {
    // This assumes the UART is already mapped by the loader

    #[cfg(any(not(feature = "production"), feature = "log-serial"))]
    let mut uart = Uart::new(UART_ADDR, crate::debug::serial::process_characters);
    #[cfg(all(feature = "production", not(feature = "log-serial")))]
    let mut uart = Uart::new(UART_ADDR, |_| ());

    uart.init();

    unsafe {
        UART = Some(uart);

        #[cfg(any(not(feature = "production"), feature = "log-serial"))]
        crate::debug::serial::init((&mut *core::ptr::addr_of_mut!(UART)).as_mut().unwrap());
    }
}

pub fn claim_interrupt() {
    unsafe {
        // Claim UART interrupt
        klog!("Claiming IRQ {:?} via syscall...", UART_IRQ_NUM);
        xous::claim_interrupt(
            UART_IRQ_NUM,
            Uart::irq,
            ((&mut *core::ptr::addr_of_mut!(UART)).as_mut().unwrap() as *mut Uart) as *mut usize,
        )
        .expect("Couldn't claim debug interrupts");
        ((&mut *core::ptr::addr_of_mut!(UART)).as_mut().unwrap() as &mut Uart).enable_rx_irq();
    }
}
