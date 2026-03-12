// SPDX-FileCopyrightText: 2022 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: Apache-2.0

use atsama5d27::{pmc::PeripheralId, sfc::Sfc, shdwc::WakeupOptions};
use fuse::BoardRevision;
use keyos::SFC_KERNEL_ADDR;

use crate::mem::MemoryManager;

pub mod aic;
pub mod cache;
pub mod idle;
pub mod pmc;
pub mod rand;
pub mod uart;

mod icm;
pub mod page_zeroer;
mod rxlp;
mod sckc;
mod shdwc;
#[cfg(feature = "trace-systemview")]
pub mod systemview;
mod tc0;
pub(crate) mod wdt;

/// atsama5d2 specific initialization.
pub fn init() {
    wdt::restart();
    cache::init_l1();
    cache::init_l2();

    // The order of these init calls is important,
    // don't rearrange them if you don't know what you're doing!

    pmc::init_clocks(&[
        // These should already be enabled, keep them that way:
        PeripheralId::Mpddrc, // RAM controller
        PeripheralId::Aesb,   // Encrypted RAM
        PeripheralId::Lcdc,   // Keep LCD on until GUI server comes online to prevent visual glitches
        PeripheralId::Uart1,  // Serial logging
        // Other peripherals used by the kernel
        PeripheralId::Aic,
        PeripheralId::Trng,
        PeripheralId::Tc0,     // Preemption
        PeripheralId::Securam, // In the panic handler by SecuramManager
        PeripheralId::Sfc,     // Board revision etection
        // XXX: Enabling peripherals without power-manager to avoid dependency cycles.
        PeripheralId::Adc,  // Trng
        PeripheralId::Tc1,  // Ticktimer
        PeripheralId::Twi0, // I2c
        // TODO: PIO should be initialized by their respective servers
        // Note: it's enough to enable power to PIOA, as B-D are interrupt sources and not clock domains
        PeripheralId::Pioa,
        PeripheralId::Xdmac1, // Page zeroer
        PeripheralId::Icm,    // Integrity Check Monitor
    ]);

    aic::init();
    sckc::init();
    tc0::init();
    icm::init();

    uart::claim_interrupt();

    idle::init_idle();
    page_zeroer::init();
    MemoryManager::with_mut(page_zeroer::start);
}

pub fn setup_preemption(max_runtime_ms: usize) { tc0::set_timeout(max_runtime_ms); }

pub fn cancel_preemption() -> usize { tc0::stop() }

pub fn start_measuring_idle() { tc0::start_freerunning() }

pub fn shutdown() -> ! {
    let sfc = Sfc::with_alt_base_addr(SFC_KERNEL_ADDR as u32);

    if fuse::get_board_revision(&sfc) == BoardRevision::RevD1 {
        // Workaround for a spurious wake-up from WKUP0 by using RXLP instead (SFT-5196)
        rxlp::init();
        shdwc::shutdown(WakeupOptions::RXLP);
    } else {
        // Use WKUP0 for Rev-D6 boards
        shdwc::shutdown(WakeupOptions::WKUP0);
    }
}
