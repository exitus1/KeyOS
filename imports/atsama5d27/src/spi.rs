use {
    crate::dma::{
        DmaChunkSize,
        DmaDataWidth,
        DmaPeripheralId,
        DmaPeripheralTransferConfig,
        DmaTransferDirection,
    },
    bitflags::bitflags,
    utralib::{utra::spi0::*, CSR, HW_SPI0_BASE, HW_SPI1_BASE},
};

bitflags! {
    #[derive(Debug, Copy, Clone)]
    pub struct SPIStatus: u32 {
        /// Receive Data Register Full (cleared by reading `SPI_RDR`)
        const RDRF        = 1 << 0;
        /// Transmit Data Register Empty (cleared by writing `SPI_TDR`)
        const TDRE        = 1 << 1;
        /// Mode Fault Error (cleared on read)
        const MODF        = 1 << 2;
        /// Overrun Error Status (cleared on read)
        const OVRES       = 1 << 3;
        /// NSS Rising (cleared on read)
        const NSSR        = 1 << 8;
        /// Transmission Registers Empty (cleared by writing SPI_TDR)
        const TXEMPTY     = 1 << 9;
        /// Underrun Error Status (Client mode only) (cleared on read)
        const UNDES       = 1 << 10;
        /// Comparison Status (cleared on read)
        const CMP         = 1 << 11;
        /// SPI Enable Status
        const SPIENS      = 1 << 16;
        /// Transmit FIFO Empty Flag (cleared on read)
        const TXFEF       = 1 << 24;
        /// Transmit FIFO Full Flag (cleared on read)
        const TXFFF       = 1 << 25;
        /// Transmit FIFO Threshold Flag (cleared on read)
        const TXFTHF      = 1 << 26;
        /// Receive FIFO Empty Flag
        const RXFEF       = 1 << 27;
        /// Receive FIFO Full Flag
        const RXFFF       = 1 << 28;
        /// Receive FIFO Threshold Flag
        const RXFTHF      = 1 << 29;
        /// Transmit FIFO Pointer Error Flag
        const TXFPTEF     = 1 << 30;
        /// Receive FIFO Pointer Error Flag
        const RXFPTEF     = 1 << 31;
    }
}

const DEFAULT_SPI_TIMEOUT_CYCLES: usize = 100_000;
const WPKEY: u32 = 0x53_50_49; // "SPI"

const RDR_OFFSET: u32 = 0x08;
const TDR_OFFSET: u32 = 0x0C;

#[derive(Debug)]
pub enum SpiError {
    Error,
    Timeout,
}

#[cfg(feature = "eh-1")]
impl eh_1::spi::Error for SpiError {
    fn kind(&self) -> eh_1::spi::ErrorKind {
        eh_1::spi::ErrorKind::Other
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ChipSelect {
    Cs0 = 0b0000,
    Cs1 = 0b0001,
    Cs2 = 0b0011,
    Cs3 = 0b0111,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BitsPerTransfer {
    Bits8 = 0,
    Bits9 = 1,
    Bits10 = 2,
    Bits11 = 3,
    Bits12 = 4,
    Bits13 = 5,
    Bits14 = 6,
    Bits15 = 7,
    Bits16 = 8,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ClockPhase {
    /// CPHA = 0, Data is changed on the leading edge of SPCK and captured on the
    /// following edge of SPCK.
    Capture = 0,
    /// CPHA = 1, Data is captured on the leading edge of SPCK and changed on the
    /// following edge of SPCK.
    Change = 1,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ClockPolarity {
    /// The inactive state value of SPCK is logic level zero.
    Low = 0,
    /// The inactive state value of SPCK is logic level one.
    High = 1,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SpiMode {
    Mode0,
    Mode1,
    Mode2,
    Mode3,
}

pub struct Spi {
    base_addr: u32,
}

impl Spi {
    pub const TX_DMA_CONFIG_SPI0_8: DmaPeripheralTransferConfig = DmaPeripheralTransferConfig {
        peripheral_id: DmaPeripheralId::Spi0Tx,
        direction: DmaTransferDirection::MemoryToPeripheral,
        data_width: DmaDataWidth::D8,
        chunk_size: DmaChunkSize::C1,
    };
    pub const RX_DMA_CONFIG_SPI0_8: DmaPeripheralTransferConfig = DmaPeripheralTransferConfig {
        peripheral_id: DmaPeripheralId::Spi0Rx,
        direction: DmaTransferDirection::PeripheralToMemory,
        data_width: DmaDataWidth::D8,
        chunk_size: DmaChunkSize::C1,
    };
    #[inline]
    pub fn spi0() -> Self {
        Spi {
            base_addr: HW_SPI0_BASE as u32,
        }
    }

    #[inline]
    pub fn spi1() -> Self {
        Spi {
            base_addr: HW_SPI1_BASE as u32,
        }
    }

    #[inline]
    pub fn with_alt_base_addr(base_addr: u32) -> Self {
        Self { base_addr }
    }

    #[inline]
    pub fn set_enabled(&mut self, enabled: bool) {
        let mut csr = CSR::new(self.base_addr as *mut u32);
        if enabled {
            csr.wfo(CR_SPIEN, 1);
        } else {
            csr.wfo(CR_SPIDIS, 1);
        }
    }

    #[inline]
    pub fn with_cs<T>(&mut self, cs: ChipSelect, f: impl FnOnce(&mut Self) -> T) -> T {
        self.select_cs(cs);
        let result = f(self);
        self.release_cs();
        result
    }

    fn select_cs(&mut self, cs: ChipSelect) {
        let mut csr = CSR::new(self.base_addr as *mut u32);
        csr.rmwf(MR_PCS, cs as u32);
    }

    fn release_cs(&mut self) {
        let mut csr = CSR::new(self.base_addr as *mut u32);
        csr.wfo(CR_LASTXFER, 1);
    }

    #[inline]
    pub fn set_mode(&mut self, cs: ChipSelect, mode: SpiMode) {
        use {ClockPhase::*, ClockPolarity::*};
        let (cpol, ncpha) = match mode {
            SpiMode::Mode0 => (Low, Change),
            SpiMode::Mode1 => (Low, Capture),
            SpiMode::Mode2 => (High, Change),
            SpiMode::Mode3 => (High, Capture),
        };

        let mut csr = CSR::new(self.get_csr_for_cs(cs) as *mut u32);
        csr.rmwf(CSR_CPOL, cpol as u32);
        csr.rmwf(CSR_NCPHA, ncpha as u32);
    }

    #[inline]
    pub fn set_bits(&mut self, cs: ChipSelect, bits: BitsPerTransfer) {
        let mut csr = CSR::new(self.get_csr_for_cs(cs) as *mut u32);
        csr.rmwf(CSR_BITS, bits as u32);
    }

    /// `true` - The Peripheral Chip Select Line does not rise after the last transfer is
    /// achieved. It remains active until a new transfer is requested on a different
    /// chip select. `false` - The Peripheral Chip Select Line rises systematically
    /// after each transfer performed on the same client.
    #[inline]
    pub fn set_cs_active_after_xfer(&mut self, cs: ChipSelect, active: bool) {
        let mut csr = CSR::new(self.get_csr_for_cs(cs) as *mut u32);

        if active {
            csr.rmwf(CSR_CSAAT, 1);
        } else {
            csr.rmwf(CSR_CSAAT, 0);
            csr.rmwf(CSR_CSNAAT, 1);
        }
    }

    #[inline]
    pub fn init(&mut self) {
        let mut csr = CSR::new(self.base_addr as *mut u32);
        self.set_enabled(false);
        csr.wfo(CR_SWRST, 1);
        csr.rmwf(MR_WDRBT, 1);
        csr.rmwf(MR_MODFDIS, 1);
        let _ = csr.r(RDR); // Dummy read the RX register
    }

    #[inline]
    pub fn master_enable(&mut self, enable: bool) {
        let mut csr = CSR::new(self.base_addr as *mut u32);
        csr.rmwf(MR_MSTR, enable as u32);
    }

    #[inline]
    pub fn init_cs(
        &mut self,
        cs: ChipSelect,
        bits: BitsPerTransfer,
        mode: SpiMode,
        cs_active_after_xfer: bool,
    ) {
        self.set_bits(cs, bits);
        self.set_mode(cs, mode);
        self.set_cs_active_after_xfer(cs, cs_active_after_xfer);
    }

    /// Delay between consecutive transfers.
    #[inline]
    pub fn set_dlybct(&mut self, pclk: u32, cs: ChipSelect, delay: u32) {
        let dlybct = ((pclk / 32000) * delay) / 100;
        assert!(dlybct <= 255, "Invalid DLYBCT delay value causing overflow");

        let mut csr = CSR::new(self.get_csr_for_cs(cs) as *mut u32);
        csr.rmwf(CSR_DLYBCT, dlybct);
    }

    /// Delay before SPCK. Unit is 10ns.
    #[inline]
    pub fn set_dlybs(&mut self, pclk: u32, cs: ChipSelect, delay: u32) {
        let dlybs = ((pclk / 1000000) * delay) / 100;
        assert!(dlybs <= 255, "Invalid DLYBS delay value causing overflow");

        let mut csr = CSR::new(self.get_csr_for_cs(cs) as *mut u32);
        csr.rmwf(CSR_DLYBS, dlybs);
    }

    #[inline]
    pub fn set_bitrate(&mut self, pclk: u32, cs: ChipSelect, bitrate: u32) {
        assert_ne!(bitrate, 0, "bitrate can't be zero");
        let scbr = pclk / bitrate;
        assert!(
            scbr <= 255,
            "Bitrate is too low, SCBR would overflow. Try selecting higher bitrate"
        );

        let mut csr = CSR::new(self.get_csr_for_cs(cs) as *mut u32);
        csr.rmwf(CSR_SCBR, scbr);
    }

    /// Chooses `CSR[0]`...`CSR[3]` for the specific chip select and provides its offset.
    fn get_csr_for_cs(&self, cs: ChipSelect) -> u32 {
        self.base_addr
            + match cs {
                ChipSelect::Cs0 => 0,
                ChipSelect::Cs1 => 0x04,
                ChipSelect::Cs2 => 2 * 0x04,
                ChipSelect::Cs3 => 3 * 0x04,
            }
    }

    #[inline]
    pub fn lock(&mut self) {
        let mut csr = CSR::new(self.base_addr as *mut u32);
        csr.wo(WPMR, WPKEY | 0b01);
    }

    #[inline]
    pub fn unlock(&mut self) {
        let mut csr = CSR::new(self.base_addr as *mut u32);
        csr.wo(WPMR, WPKEY);
    }

    #[inline]
    pub fn is_wp_violated(&self) -> bool {
        let csr = CSR::new(self.base_addr as *mut u32);
        csr.rf(WPSR_WPVS) != 0
    }

    #[inline]
    pub fn write_8(&mut self, hw: u8) -> Result<(), SpiError> {
        self.wait_status(SPIStatus::TDRE)?;
        let reg_addr = self.base_addr + TDR_OFFSET;

        unsafe {
            core::arch::asm!(
                "strb {}, [{}]",
                in(reg) hw,
                in(reg) reg_addr,
            );
        }

        self.wait_status(SPIStatus::TXEMPTY)?;

        Ok(())
    }

    #[inline]
    pub fn write_16(&mut self, hw: u16) -> Result<(), SpiError> {
        self.wait_status(SPIStatus::TDRE)?;
        let reg_addr = self.base_addr + TDR_OFFSET;

        unsafe {
            core::arch::asm!(
                "strh {}, [{}]",
                in(reg) hw,
                in(reg) reg_addr,
            );
        }

        self.wait_status(SPIStatus::TXEMPTY)?;

        Ok(())
    }

    #[inline]
    pub fn read_16(&mut self) -> Result<u16, SpiError> {
        self.wait_status(SPIStatus::RDRF)?;

        let reg_addr = self.base_addr + RDR_OFFSET;
        let mut hw;
        unsafe {
            core::arch::asm!(
                "ldrh {}, [{}]",
                out(reg) hw,
                in(reg) reg_addr,
            );
        }

        Ok(hw)
    }

    #[inline]
    pub fn read_8(&mut self) -> Result<u8, SpiError> {
        self.wait_status(SPIStatus::RDRF)?;

        let reg_addr = self.base_addr + RDR_OFFSET;
        let mut hw;
        unsafe {
            core::arch::asm!(
                "ldrb {}, [{}]",
                out(reg) hw,
                in(reg) reg_addr,
            );
        }

        Ok(hw)
    }

    #[inline]
    pub fn dma_tx_addr(&self) -> usize {
        (self.base_addr + TDR_OFFSET) as usize
    }
    #[inline]
    pub fn dma_rx_addr(&self) -> usize {
        (self.base_addr + RDR_OFFSET) as usize
    }

    #[inline]
    pub fn status(&mut self) -> SPIStatus {
        let csr = CSR::new(self.base_addr as *mut u32);
        let bits = csr.r(SR);
        SPIStatus::from_bits_retain(bits)
    }

    fn wait_status(&mut self, status: SPIStatus) -> Result<(), SpiError> {
        let mut timeout = DEFAULT_SPI_TIMEOUT_CYCLES;
        while timeout > 0 {
            let curr_status = self.status();

            if curr_status.contains(status) {
                return Ok(());
            }

            timeout -= 1;
        }

        Err(SpiError::Timeout)
    }
}
