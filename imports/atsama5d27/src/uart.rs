//! UART controller.

use {
    crate::dma::{
        DmaChunkSize,
        DmaDataWidth,
        DmaPeripheralId,
        DmaPeripheralTransferConfig,
        DmaTransferDirection,
    },
    core::{fmt::Write, marker::PhantomData},
    utralib::{utra::uart0::*, *},
};

pub struct Uart0 {}
pub struct Uart1 {}
pub struct Uart2 {}
pub struct Uart3 {}
pub struct Uart4 {}

const RHR_OFFSET: u32 = 0x18;
const THR_OFFSET: u32 = 0x1C;

mod sealed {
    use crate::uart::*;

    pub trait Sealed {}
    impl Sealed for Uart0 {}
    impl Sealed for Uart1 {}
    impl Sealed for Uart2 {}
    impl Sealed for Uart3 {}
    impl Sealed for Uart4 {}
}

pub trait UartPeriph: sealed::Sealed {
    const BASE_ADDRESS: usize;
    const DMA_TX_ID: DmaPeripheralId;
    const DMA_RX_ID: DmaPeripheralId;
}

impl UartPeriph for Uart0 {
    const BASE_ADDRESS: usize = 0xf801c000;
    const DMA_TX_ID: DmaPeripheralId = DmaPeripheralId::Uart0Tx;
    const DMA_RX_ID: DmaPeripheralId = DmaPeripheralId::Uart0Rx;
}
impl UartPeriph for Uart1 {
    const BASE_ADDRESS: usize = 0xf8020000;
    const DMA_TX_ID: DmaPeripheralId = DmaPeripheralId::Uart1Tx;
    const DMA_RX_ID: DmaPeripheralId = DmaPeripheralId::Uart1Rx;
}
impl UartPeriph for Uart2 {
    const BASE_ADDRESS: usize = 0xf8024000;
    const DMA_TX_ID: DmaPeripheralId = DmaPeripheralId::Uart2Tx;
    const DMA_RX_ID: DmaPeripheralId = DmaPeripheralId::Uart2Rx;
}
impl UartPeriph for Uart3 {
    const BASE_ADDRESS: usize = 0xfc008000;
    const DMA_TX_ID: DmaPeripheralId = DmaPeripheralId::Uart3Tx;
    const DMA_RX_ID: DmaPeripheralId = DmaPeripheralId::Uart3Rx;
}
impl UartPeriph for Uart4 {
    const BASE_ADDRESS: usize = 0xfc00c000;
    const DMA_TX_ID: DmaPeripheralId = DmaPeripheralId::Uart4Tx;
    const DMA_RX_ID: DmaPeripheralId = DmaPeripheralId::Uart4Rx;
}

#[derive(Debug)]
pub enum Parity {
    Even = 0,
    Odd = 1,
    Space = 2,
    Mark = 3,
    No = 4,
}

#[derive(Default)]
pub struct Uart<U: UartPeriph> {
    base_addr: usize,
    inner: PhantomData<U>,
}

impl<U: UartPeriph> Uart<U> {
    pub const TX_DMA_CONFIG: DmaPeripheralTransferConfig = DmaPeripheralTransferConfig {
        peripheral_id: U::DMA_TX_ID,
        direction: DmaTransferDirection::MemoryToPeripheral,
        data_width: DmaDataWidth::D8,
        chunk_size: DmaChunkSize::C1,
    };
    pub const RX_DMA_CONFIG: DmaPeripheralTransferConfig = DmaPeripheralTransferConfig {
        peripheral_id: U::DMA_RX_ID,
        direction: DmaTransferDirection::PeripheralToMemory,
        data_width: DmaDataWidth::D8,
        chunk_size: DmaChunkSize::C1,
    };
    pub const BASE_ADDRESS: usize = U::BASE_ADDRESS;

    #[inline]
    pub fn new() -> Uart<U> {
        Uart {
            base_addr: Self::BASE_ADDRESS,
            inner: PhantomData,
        }
    }

    /// Creates a driver instance with an alternative base address.
    /// Useful when the UART peripheral is remapped to some other virtual address by the
    /// MMU.
    #[inline]
    pub fn with_alt_base_addr(base_addr: usize) -> Uart<U> {
        Uart {
            base_addr,
            inner: PhantomData,
        }
    }

    #[inline]
    pub fn init(&mut self, clock_speed: u32, baud_rate: u32, parity: Parity) {
        let mut csr = CSR::new(self.base_addr as *mut u32);

        // Reset everything
        csr.wfo(CR_RSTTX, 1);
        csr.wfo(CR_RSTRX, 1);
        csr.wfo(CR_RXDIS, 1);
        csr.wfo(CR_TXDIS, 1);
        csr.wfo(CR_RSTSTA, 1);

        // Set baud rate and parity
        csr.wo(BRGR, clock_speed / (16 * baud_rate));
        self.set_parity(parity);

        // Enable receiver and transmitter
        csr.wfo(CR_RXEN, 1);
        csr.wfo(CR_TXEN, 1);
    }

    #[inline]
    pub fn set_baud(&mut self, clock_speed: u32, baud_rate: u32) {
        let mut csr = CSR::new(self.base_addr as *mut u32);

        csr.wfo(CR_RSTTX, 1);
        csr.wfo(CR_RSTRX, 1);
        self.set_tx(false);
        self.set_rx(false);

        csr.wo(BRGR, clock_speed / (16 * baud_rate));
    }

    #[inline]
    pub fn set_parity(&mut self, parity: Parity) {
        let mut csr = CSR::new(self.base_addr as *mut u32);
        csr.rmwf(MR_PAR, parity as u32);
    }

    #[inline]
    pub fn write_byte(&mut self, byte: u8) {
        let mut uart_csr = CSR::new(self.base_addr as *mut u32);

        // Wait for the previous transfer to complete
        while uart_csr.rf(SR_TXRDY) == 0 {
            armv7::asm::nop();
        }

        // Send the byte
        uart_csr.wfo(THR_TXCHR, byte as u32);

        // Wait for the current transfer to complete
        while uart_csr.rf(SR_TXEMPTY) == 0 {
            armv7::asm::nop();
        }
    }

    #[inline]
    pub fn write_str(&mut self, s: &str) {
        for byte in s.as_bytes().iter() {
            self.write_byte(*byte);
        }
    }

    #[inline]
    pub fn set_rx(&mut self, enabled: bool) {
        let mut uart_csr = CSR::new(self.base_addr as *mut u32);
        if enabled {
            uart_csr.wfo(CR_RXEN, 1);
        } else {
            uart_csr.wfo(CR_RXDIS, 1);
        }
    }

    #[inline]
    pub fn set_tx(&mut self, enabled: bool) {
        let mut uart_csr = CSR::new(self.base_addr as *mut u32);
        if enabled {
            uart_csr.wfo(CR_TXEN, 1);
        } else {
            uart_csr.wfo(CR_TXDIS, 1);
        }
    }

    #[inline]
    pub fn set_rx_interrupt(&mut self, enabled: bool) {
        let mut uart_csr = CSR::new(self.base_addr as *mut u32);
        uart_csr.wfo(IER_RXRDY, enabled.into());
    }

    #[inline]
    pub fn getc_nonblocking(&mut self) -> Option<u8> {
        let uart_csr = CSR::new(self.base_addr as *mut u32);
        if uart_csr.rf(SR_RXRDY) != 0 {
            Some(uart_csr.rf(RHR_RXCHR) as u8)
        } else {
            None
        }
    }

    #[inline]
    pub fn getc(&mut self) -> u8 {
        let uart_csr = CSR::new(self.base_addr as *mut u32);

        // Wait for the character reception to complete
        while uart_csr.rf(SR_RXRDY) == 0 {
            armv7::asm::nop();
        }

        uart_csr.rf(RHR_RXCHR) as u8
    }

    #[inline]
    pub fn is_overrun(&self) -> bool {
        let uart_csr = CSR::new(self.base_addr as *mut u32);
        uart_csr.rf(SR_OVRE) != 0
    }

    #[inline]
    pub fn getc_isr(&self) -> u8 {
        let uart_csr = CSR::new(self.base_addr as *mut u32);
        uart_csr.rf(RHR_RXCHR) as u8
    }
    #[inline]
    pub fn dma_tx_addr(&self) -> usize {
        self.base_addr as usize + THR_OFFSET as usize
    }
    #[inline]
    pub fn dma_rx_addr(&self) -> usize {
        self.base_addr + RHR_OFFSET as usize
    }
}

impl<U: UartPeriph> Write for Uart<U> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        Uart::<U>::write_str(self, s);
        Ok(())
    }
}
