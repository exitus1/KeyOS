// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use server::Message;

#[derive(Debug, Message)]
pub struct ChildCrashed(pub u32);

#[derive(Debug, Message)]
pub struct Disconnected(xous::CID);
