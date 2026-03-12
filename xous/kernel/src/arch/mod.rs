// SPDX-FileCopyrightText: 2020 Sean Cross <sean@xobs.io>
// SPDX-License-Identifier: Apache-2.0

#[cfg(keyos)]
mod arm;
#[cfg(keyos)]
pub use crate::arch::arm::*;

#[cfg(any(windows, unix))]
mod hosted;
#[cfg(any(windows, unix))]
pub use hosted::*;

#[cfg(all(target_arch = "x86_64", not(any(windows, unix))))]
mod x86_64;
#[cfg(all(target_arch = "x86_64", not(any(windows, unix))))]
pub use x86_64::*;
