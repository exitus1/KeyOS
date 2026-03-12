// SPDX-FileCopyrightText: 2022 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: Apache-2.0

use atsama5d27::sckc::{Sckc, SclkType};
use keyos::{PAGE_SIZE, RSTC_KERNEL_ADDR};
use utralib::{HW_RSTC_BASE, HW_SCKC_BASE};

pub fn init() {
    const _: () = assert!(
        HW_RSTC_BASE == HW_SCKC_BASE & !(PAGE_SIZE - 1),
        "HW_SCKC_BASE and HW_RSTC_BASE are not on the same page"
    );
    const OFFSET: usize = HW_SCKC_BASE & (PAGE_SIZE - 1);
    let sckc_offseted_base = RSTC_KERNEL_ADDR + OFFSET;
    let mut sckc = Sckc::with_alt_base_addr(sckc_offseted_base as u32);
    if sckc.selected_clock() != SclkType::Crystal {
        sckc.select_clock(SclkType::Crystal);
    }
}
