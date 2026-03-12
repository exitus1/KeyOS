// SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

#[cfg(not(keyos))]
mod hosted;
#[cfg(not(keyos))]
pub use hosted::*;

#[cfg(keyos)]
mod atsama5d2;
#[cfg(keyos)]
pub use atsama5d2::*;
