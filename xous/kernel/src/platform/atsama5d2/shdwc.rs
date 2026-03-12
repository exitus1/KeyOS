// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: Apache-2.0

use atsama5d27::shdwc::{Shdwc, WakeupOptions};
use keyos::{PAGE_SIZE, RSTC_KERNEL_ADDR};
use utralib::{HW_RSTC_BASE, HW_SHDWC_BASE};

pub fn shutdown(wakeup_options: WakeupOptions) -> ! {
    const _: () = assert!(
        HW_RSTC_BASE == HW_SHDWC_BASE & !(PAGE_SIZE - 1),
        "HW_SHDWC_BASE and HW_RSTC_BASE are not on the same page"
    );
    const OFFSET: usize = HW_SHDWC_BASE & (PAGE_SIZE - 1);
    let shdwc_offset_base = RSTC_KERNEL_ADDR + OFFSET;
    let shdwc = Shdwc::with_alt_base_addr(shdwc_offset_base as u32);
    shdwc.do_shutdown(wakeup_options);
}
