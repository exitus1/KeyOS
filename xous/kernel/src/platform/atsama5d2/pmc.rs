// SPDX-FileCopyrightText: 2022 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: Apache-2.0

use core::convert::TryFrom;

use atsama5d27::pmc::{PeripheralId, Pmc};
use keyos::PMC_KERNEL_ADDR;
use utralib::HW_PMC_BASE;
use xous::MemoryFlags;

use crate::mem::MemoryManager;

/// Enable peripheral clocks (in order) and disable the rest.
pub fn init_clocks(peripherals_to_enable: &[PeripheralId]) {
    let pmc_virt = MemoryManager::with_mut(|memory_manager| {
        memory_manager
            .map_range(HW_PMC_BASE, PMC_KERNEL_ADDR as _, 0x1000, MemoryFlags::W | MemoryFlags::DEV, false)
            .expect("unable to map PMC")
    });
    let mut pmc = Pmc::with_alt_base_addr(pmc_virt.as_ptr() as u32);

    pmc.disable_system_clock_lcdc();
    pmc.disable_rc_oscillator();
    // Startup time is 60 us, which is 2 slow clock cycles. Added +1 for good measure.
    pmc.set_plla_period(3);

    for pid in peripherals_to_enable {
        pmc.enable_peripheral_clock(*pid);
    }
    for pid in 2..60 {
        let Ok(pid) = PeripheralId::try_from(pid) else { continue };
        if !peripherals_to_enable.contains(&pid) {
            pmc.disable_peripheral_clock(pid);
        }
    }
}
