// SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

#[cfg(keyos)]
mod app;

#[cfg(keyos)]
pub use app::main;

#[cfg(not(keyos))]
pub fn main() {}
