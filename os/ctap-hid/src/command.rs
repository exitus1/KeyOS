// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use num_derive::{FromPrimitive, ToPrimitive};

#[derive(Debug, Default, Clone, Copy, PartialEq, FromPrimitive, ToPrimitive)]
pub enum Command {
    Ping = 0x01,
    #[default]
    Init = 0x06,
    Message = 0x03,
    Wink = 0x08,
    Cbor = 0x10,
    Cancel = 0x11,
    Error = 0x3f,
    Lock = 0x04,
    KeepAlive = 0x3b,
}
