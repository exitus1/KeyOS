// SPDX-FileCopyrightText: 2026 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

pub use error::{EhciError, UsbError};

#[cfg(keyos)]
pub mod device;
pub mod error;
#[cfg(keyos)]
pub mod host;
