// SPDX-FileCopyrightText: 2022 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: Apache-2.0

#[cfg(keyos)]
pub mod atsama5d2;

#[cfg(keyos)]
pub use atsama5d2::{
    cancel_preemption, idle::set_dram_idle_mode, page_zeroer, setup_preemption, start_measuring_idle,
};

pub mod rand;

/// Platform specific initialization.
#[cfg(keyos)]
pub fn init() { self::atsama5d2::init(); }
