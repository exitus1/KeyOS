// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: Apache-2.0

use atsama5d27::wdt::Wdt;
use keyos::{PAGE_SIZE, RSTC_KERNEL_ADDR};
use utralib::{HW_RSTC_BASE, HW_WDT_BASE};

pub fn restart() {
    const _: () = assert!(
        HW_RSTC_BASE == HW_WDT_BASE & !(PAGE_SIZE - 1),
        "HW_WDT_BASE and HW_RSTC_BASE are not on the same page"
    );
    const OFFSET: usize = HW_WDT_BASE & (PAGE_SIZE - 1);
    let wdt_offset_base = RSTC_KERNEL_ADDR + OFFSET;
    let wdt = Wdt::with_alt_base_addr(wdt_offset_base as u32);

    wdt.restart();
}
