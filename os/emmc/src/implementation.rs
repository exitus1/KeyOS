// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

#![cfg(keyos)]

use atsama5d27::sdmmc::DMADescAttr;
use server::{MessageId as _, ScalarHandler};
use ticktimer::TicktimerCallback;
use xous::keyos::{is_address_encrypted, PAGE_SIZE, TOTAL_FLASH_BLOCKS};
use {
    crate::BLOCK_SIZE,
    atsama5d27::sdmmc::{
        ADMADesc, DataDirection, DmaParams, ErrorStatus, NormalStatus, SDCmd, SDCmdInner, SDRespType, Sdmmc,
    },
    utralib::HW_SDMMC0_BASE,
    xous::{arch::irq::IrqNumber, MemoryRange, Message, CID, SID},
};

use crate::{error::EmmcError, messages::*, SD_BUFFER_BLOCKS};

power_manager::use_api!();

const DMA_DESC_TABLE_SIZE: usize = SD_BUFFER_BLOCKS;
const DMA_TABLE_SIZE_BYTES: usize = {
    if core::mem::size_of::<ADMADesc>() * DMA_DESC_TABLE_SIZE > 4096 {
        panic!("DMA table won't fit into a single page");
    } else {
        4096
    }
};
const POWER_SAVE_AFTER_MS: usize = 500;
/// The relative card address, shifted to a position where most commands expect it.
const RCA_SHIFTED: u32 = 1 << 16;

#[derive(server::Server)]
#[name = "os/emmc"]
pub(crate) struct EmmcServer {
    sdmmc: Sdmmc,
    dma_table_phys_addr: u32,
    dma_table: MemoryRange,
    dummy_sid: SID,
    power_manager: PowerManagerApi,
    enabled: bool,
    suspend_callback: Option<TicktimerCallback>,
    pub(crate) crypto_api: Option<crate::CryptoApi>,
    pub(crate) tmp_buf: MemoryRange,
}

struct InterruptContext {
    sdmmc: Sdmmc,
    cid: CID,
}

impl server::Server for EmmcServer {
    fn on_start(&mut self, context: &mut server::ServerContext<Self>) {
        let suspend_callback = TicktimerCallback::new(context.sid()).expect("Could not connect to ticktimer");
        suspend_callback.request(POWER_SAVE_AFTER_MS, Suspend::ID, 0);
        self.suspend_callback = Some(suspend_callback);
    }
}

impl EmmcServer {
    pub fn new() -> Result<Self, EmmcError> {
        let dummy_sid = xous::create_server()?;
        let dummy_cid = xous::connect(dummy_sid)?;
        let power_manager = PowerManagerApi::default();
        power_manager.enable_peripheral(atsama5d27::pmc::PeripheralId::Sdmmc0).unwrap();

        log::debug!("Mapping SDMMC0 peripheral");

        let mem = xous::map_memory(
            xous::MemoryAddress::new(HW_SDMMC0_BASE),
            None,
            0x1000,
            xous::MemoryFlags::W | xous::MemoryFlags::DEV,
        )?;
        let sdmmc_addr = mem.as_ptr() as u32;

        log::debug!("Mapped SDMMC0 to 0x{:08x}", sdmmc_addr);
        let mut sdmmc = Sdmmc::with_alt_base_addr(sdmmc_addr);

        let dma_table = xous::map_memory(
            None,
            None,
            DMA_TABLE_SIZE_BYTES,
            xous::MemoryFlags::W
                | xous::MemoryFlags::POPULATE
                | xous::MemoryFlags::PLAINTEXT
                | xous::MemoryFlags::NO_CACHE,
        )?;
        let dma_table_phys_addr = xous::virt_to_phys(dma_table.as_ptr() as usize)? as u32;
        log::debug!("Allocated DMA table at {:08x?}, phys: {:08x}", dma_table, dma_table_phys_addr);

        log::debug!("Claiming SDMMC0 IRQ");

        let int_ctx = Box::into_raw(Box::new(InterruptContext {
            sdmmc: Sdmmc::with_alt_base_addr(sdmmc_addr),
            cid: dummy_cid,
        }));
        xous::claim_interrupt(IrqNumber::Sdmmc0, sdmmc0_irq_handler, int_ctx as *mut usize)?;
        sdmmc.enable_error_interrupt_signal(ErrorStatus::all());

        let tmp_buf = xous::map_memory(
            None,
            None,
            SD_BUFFER_BLOCKS * BLOCK_SIZE,
            xous::MemoryFlags::W | xous::MemoryFlags::POPULATE | xous::MemoryFlags::PLAINTEXT,
        )?;

        Ok(EmmcServer {
            sdmmc,
            dma_table_phys_addr,
            dma_table,
            dummy_sid,
            power_manager,
            enabled: true,
            suspend_callback: None,
            crypto_api: None,
            tmp_buf,
        })
    }

    fn enable(&mut self) {
        self.power_manager
            .enable_peripheral(atsama5d27::pmc::PeripheralId::Sdmmc0)
            .expect("Could not enable SDMMC0 clock");
        self.sdmmc.enable_clock();
        self.enabled = true;

        let resp = self.sdmmc.send_command(SDCmd::Sd(SDCmdInner::Sleep), SDRespType::R1B, RCA_SHIFTED, None);
        log::debug!("Response to wake: {resp:08x?}");
        let resp =
            self.sdmmc.send_command(SDCmd::Sd(SDCmdInner::SelectCard), SDRespType::R1, RCA_SHIFTED, None);
        log::debug!("Response to select: {resp:08x?}");
        let resp =
            self.sdmmc.send_command(SDCmd::Sd(SDCmdInner::SendStatus), SDRespType::R1, RCA_SHIFTED, None);
        log::debug!("Current status: {resp:08x?}");
    }

    fn disable(&mut self) {
        self.sdmmc.send_command(SDCmd::Sd(SDCmdInner::SelectCard), SDRespType::NoResp, 0, None).ok();
        const SLEEP_BIT: u32 = 1 << 15;
        let resp = self.sdmmc.send_command(
            SDCmd::Sd(SDCmdInner::Sleep),
            SDRespType::R1B,
            RCA_SHIFTED | SLEEP_BIT,
            None,
        );
        log::debug!("Response to sleep: {resp:08x?}");
        self.sdmmc.disable_clock();
        self.power_manager
            .disable_peripheral(atsama5d27::pmc::PeripheralId::Sdmmc0)
            .expect("Could not enable SDMMC0 clock");
        self.enabled = false;
    }

    pub(crate) fn hardware_request(
        &mut self,
        request_direction: Direction,
        block_index: u32,
        blocks: usize,
        buffer: *mut u8,
    ) -> Result<usize, EmmcError> {
        if !self.enabled {
            self.enable()
        }

        if blocks > SD_BUFFER_BLOCKS {
            return Err(EmmcError::BufferTooLarge);
        }

        let (cmd, direction) = match request_direction {
            Direction::Read => (SDCmdInner::ReadMultipleBlocks, DataDirection::Read),
            Direction::Write => (SDCmdInner::WriteMultipleBlocks, DataDirection::Write),
        };
        let dma_params = DmaParams {
            dma_desc_table_phys_addr: self.dma_table_phys_addr,
            direction,
            blocks,
            block_size: BLOCK_SIZE as u16,
        };

        assert_eq!(buffer as usize & (PAGE_SIZE - 1), 0, "Buffer must be aligned");
        let buffer_len = blocks * BLOCK_SIZE;
        let pages = buffer_len.next_multiple_of(PAGE_SIZE) / PAGE_SIZE;

        for (i, dma_desc) in (0..pages).zip(self.dma_table.as_slice_mut::<ADMADesc>().iter_mut()) {
            // Last descriptor must have the end bit
            if i == pages - 1 {
                // 0x23
                dma_desc.attr = DMADescAttr::TRAN | DMADescAttr::VALID | DMADescAttr::END;
            } else {
                // 0x21
                dma_desc.attr = DMADescAttr::TRAN | DMADescAttr::VALID;
            }
            dma_desc.len = (buffer_len - i * PAGE_SIZE).min(PAGE_SIZE) as u16;
            dma_desc.addr = xous::virt_to_phys(buffer as usize + i * PAGE_SIZE)? as u32;
        }

        // Clear status so we don't get an interrupt from a previous status bit
        self.sdmmc.clear_normal_status(self.sdmmc.normal_status());
        self.sdmmc.enable_interrupt_signal(NormalStatus::TRFC); // Interrupt on DMA transfer completion
        match self.sdmmc.send_command(SDCmd::Sd(cmd), SDRespType::R1, block_index, Some(dma_params)) {
            Ok(response) => {
                log::trace!("SD response: {:08x?}", response);
            }
            Err(e) => {
                log::error!("Couldn't perform SDMMC request: {:?}", e);
                return Err(EmmcError::SdmmcError);
            }
        }
        // This will block until the DMA transfer is finished.
        // The DMA IRQ handler sends a dummy message to this server which will unblock this thread.
        xous::receive_message(self.dummy_sid)?;
        // Disable interrupts so we don't get spurious ones
        self.sdmmc.enable_interrupt_signal(NormalStatus::empty());

        self.suspend_callback.as_ref().unwrap().request(POWER_SAVE_AFTER_MS, Suspend::ID, 0);
        Ok(blocks)
    }
}

impl server::LendMutHandler<ReadBlocks> for EmmcServer {
    fn handle(
        &mut self,
        msg: ReadBlocks,
        _sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> Result<usize, EmmcError> {
        log::trace!("{msg:?}");
        if msg.block_count * BLOCK_SIZE > msg.buf.len() || msg.block_count > SD_BUFFER_BLOCKS {
            return Err(EmmcError::BufferTooLarge);
        }
        if (msg.block_index as usize).saturating_add(msg.block_count) > TOTAL_FLASH_BLOCKS {
            return Err(EmmcError::OutOfRange);
        }
        if is_address_encrypted(xous::virt_to_phys(msg.buf.as_ptr() as usize)?) {
            // The SDMMC DMA does not have access to encrypted RAM. We have to DMA data into plaintext RAM and
            // copy that to the encrypted RAM buffer.
            let tmp_buf = self.tmp_buf.subrange(0, msg.block_count * BLOCK_SIZE).unwrap();
            xous::flush_cache(tmp_buf, xous::CacheOperation::Invalidate)?;
            self.hardware_request(Direction::Read, msg.block_index, msg.block_count, tmp_buf.as_mut_ptr())?;
            msg.buf
                .subrange(0, msg.block_count * BLOCK_SIZE)
                .unwrap()
                .as_slice_mut::<[u32; BLOCK_SIZE / 4]>()
                .copy_from_slice(tmp_buf.as_slice());
            Ok(msg.block_count)
        } else {
            xous::flush_cache(
                msg.buf.subrange(0, msg.block_count * BLOCK_SIZE).unwrap(),
                xous::CacheOperation::Invalidate,
            )?;
            self.hardware_request(Direction::Read, msg.block_index, msg.block_count, msg.buf.as_mut_ptr())
        }
    }
}

impl server::LendMutHandler<WriteBlocks> for EmmcServer {
    fn handle(
        &mut self,
        msg: WriteBlocks,
        _sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> Result<usize, EmmcError> {
        log::trace!("{msg:?}");
        if msg.block_count * BLOCK_SIZE > msg.buf.len() || msg.block_count > SD_BUFFER_BLOCKS {
            return Err(EmmcError::BufferTooLarge);
        }
        if (msg.block_index as usize).saturating_add(msg.block_count) > TOTAL_FLASH_BLOCKS {
            return Err(EmmcError::OutOfRange);
        }

        if is_address_encrypted(xous::virt_to_phys(msg.buf.as_ptr() as usize)?) {
            // The SDMMC DMA does not have access to encrypted RAM. We have to copy data into plaintext RAM
            // and use that to DMA.
            let mut tmp_buf = self.tmp_buf.subrange(0, msg.block_count * BLOCK_SIZE).unwrap();
            tmp_buf
                .as_slice_mut::<[u32; BLOCK_SIZE / 4]>()
                .copy_from_slice(msg.buf.subrange(0, msg.block_count * BLOCK_SIZE).unwrap().as_slice());
            xous::flush_cache(tmp_buf, xous::CacheOperation::Clean)?;
            self.hardware_request(Direction::Write, msg.block_index, msg.block_count, tmp_buf.as_mut_ptr())?;
            Ok(msg.block_count)
        } else {
            xous::flush_cache(
                msg.buf.subrange(0, msg.block_count * BLOCK_SIZE).unwrap(),
                xous::CacheOperation::Clean,
            )?;

            self.hardware_request(Direction::Write, msg.block_index, msg.block_count, msg.buf.as_mut_ptr())
        }
    }
}

impl server::BlockingScalarHandler<BlockCount> for EmmcServer {
    fn handle(
        &mut self,
        _msg: BlockCount,
        _sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> usize {
        // Note: we could probably ask the flash chip itself
        TOTAL_FLASH_BLOCKS
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Direction {
    Read,
    Write,
}

impl ScalarHandler<Suspend> for EmmcServer {
    fn handle(&mut self, _msg: Suspend, _sender: xous::PID, _context: &mut server::ServerContext<Self>) {
        if !self.enabled {
            log::error!("Suspend called twice");
            return;
        }
        self.disable();
    }
}

// Note: the IRQ handler must never panic
fn sdmmc0_irq_handler(_irq_no: usize, arg: *mut usize) {
    let ctx = unsafe { &mut *(arg as *mut InterruptContext) };
    let ns = ctx.sdmmc.normal_status();

    // Had an error
    if ns.contains(NormalStatus::ERRINT) {
        let es = ctx.sdmmc.error_status();
        ctx.sdmmc.clear_error_status(es);
        log::error!("EMMC Error: {ns:?}, {es:?}");
    }

    // DMA transfer finished?
    if ns.contains(NormalStatus::TRFC) {
        // Acknowledge interrupt
        ctx.sdmmc.clear_normal_status(ns & NormalStatus::TRFC);
        // Signal DMA finished
        xous::try_send_message(ctx.cid, Message::new_scalar(0, 0, 0, 0, 0)).ok();
    }
}
