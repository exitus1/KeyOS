// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc <hello@foundation.xyz>
// SPDX-License-Identifier: Apache-2.0

use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use atsama5d27::dma::{DmaChannel, Xdmac, XdmacChannel};
use keyos::{PAGE_SIZE, XDMAC1_KERNEL_ADDR};
use utralib::HW_XDMAC1_BASE;
use xous::{arch::irq::IrqNumber, MemoryFlags};

use crate::{irq::interrupt_claim_kernel, mem::MemoryManager};

pub static RUNNING: AtomicBool = AtomicBool::new(false);
static INITIALIZED: AtomicBool = AtomicBool::new(false);

static CURRENT_PAGE: AtomicUsize = AtomicUsize::new(0);
static CURRENT_PAGE_NUM: AtomicUsize = AtomicUsize::new(0);

pub fn init() {
    MemoryManager::with_mut(|memory_manager| {
        memory_manager
            .map_range(
                HW_XDMAC1_BASE,
                XDMAC1_KERNEL_ADDR as *mut usize,
                0x2000,
                MemoryFlags::W | MemoryFlags::DEV,
                false,
            )
            .expect("unable to map XDMAC1 to kernel")
    });
    interrupt_claim_kernel(IrqNumber::Xdmac1, xdmac_interrupt);
    let channel = dma_channel();
    channel.set_interrupt(true);
    channel.set_bi_interrupt(true);
    channel.set_di_interrupt(true);
    channel.configure_memset_transfer(atsama5d27::dma::DmaDataWidth::D32);
    INITIALIZED.store(true, Ordering::SeqCst);
}

pub fn start(mm: &mut MemoryManager) {
    if RUNNING.load(Ordering::SeqCst) || !INITIALIZED.load(Ordering::SeqCst) {
        return;
    }
    let Some((phys, pages)) = mm.take_dirty_pages() else {
        return;
    };

    RUNNING.store(true, Ordering::SeqCst);
    CURRENT_PAGE.store(phys, Ordering::SeqCst);
    CURRENT_PAGE_NUM.store(pages, Ordering::SeqCst);
    dma_channel().execute_transfer(0, phys as u32, pages * PAGE_SIZE / core::mem::size_of::<u32>());
}

pub fn xdmac_interrupt() {
    // Ack the interrupt by reading it
    dma_channel().interrupt_status();
    MemoryManager::with_mut(|mm| {
        mm.set_pages_to_zeroed(
            CURRENT_PAGE.swap(0, Ordering::SeqCst),
            CURRENT_PAGE_NUM.swap(0, Ordering::SeqCst),
        );
        RUNNING.store(false, Ordering::SeqCst);
        start(mm);
    });
}

fn dma_channel() -> XdmacChannel {
    Xdmac::with_alt_base_addr(XDMAC1_KERNEL_ADDR).channel(DmaChannel::Channel0)
}
