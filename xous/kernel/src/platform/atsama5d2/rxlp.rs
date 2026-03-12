// SPDX-FileCopyrightText: 2022 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: Apache-2.0

use atsama5d27::rxlp::{Parity, Rxlp};
use keyos::RXLP_KERNEL_ADDR;

pub(crate) fn init() {
    let mut rxlp = Rxlp::with_alt_base_addr(RXLP_KERNEL_ADDR as u32);
    rxlp.init(1, Parity::No);
    rxlp.set_comparison(0x01, 0xff);
    rxlp.read();
    unsafe {
        core::arch::asm!("dsb");
    }
}
