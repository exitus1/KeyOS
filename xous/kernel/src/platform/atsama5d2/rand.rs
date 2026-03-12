// SPDX-FileCopyrightText: 2022 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: Apache-2.0

use atsama5d27::trng::{Enabled, StatefulTrng, Trng};
use keyos::TRNG_KERNEL_ADDR;

pub static mut TRNG_KERNEL: Option<TrngKernel> = None;

pub struct TrngKernel {
    base_addr: usize,
    pub inner: Option<StatefulTrng<Enabled>>,
}

impl TrngKernel {
    pub fn new(addr: usize) -> TrngKernel { TrngKernel { base_addr: addr, inner: None } }

    pub fn init(&mut self) { self.inner = Some(Trng::with_alt_base_addr(self.base_addr as u32).enable()); }

    pub fn get_u32(&mut self) -> u32 {
        if let Some(trng) = &self.inner {
            return trng.read_u32();
        }

        unreachable!()
    }
}

/// Initialize TRNG driver.
///
/// Needed so that the kernel can allocate names.
pub fn init() { init_trng(); }

fn init_trng() {
    // Assumes that the TRNG peripheral is already mapped by the loader

    let mut trng_kernel = TrngKernel::new(TRNG_KERNEL_ADDR);
    trng_kernel.init();

    unsafe {
        TRNG_KERNEL = Some(trng_kernel);
    }
}

/// Retrieve random `u32`.
pub fn get_u32() -> u32 {
    unsafe {
        (&mut *core::ptr::addr_of_mut!(TRNG_KERNEL))
            .as_mut()
            .expect("TRNG_KERNEL driver not initialized")
            .get_u32()
    }
}
