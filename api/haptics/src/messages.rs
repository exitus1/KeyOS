// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::HapticPattern;

/// Produce a given haptic feedback pattern.
#[derive(Debug, server::Message)]
pub struct Vibrate(pub HapticPattern);
