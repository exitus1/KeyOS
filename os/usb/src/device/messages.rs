// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use atsama5d27::udphs::DmaStatus;
use server::{AsScalar, FromScalar};

// === Internal messages ===
#[derive(Debug, server::Message, Clone)]
pub struct EndOfReset;

#[derive(Debug, server::Message, Clone)]
pub struct Ep0TxComplete;

#[derive(Debug, server::Message, Clone)]
pub struct Ep0RxComplete;

#[derive(Debug, server::Message, Clone)]
pub struct DmaInterrupt {
    pub endpoint: u8,
    pub status: DmaStatus,
}

impl AsScalar<2> for DmaInterrupt {
    fn as_scalar(&self) -> [u32; 2] { [self.endpoint as u32, self.status.0] }
}

impl FromScalar<2> for DmaInterrupt {
    fn from_scalar(value: [u32; 2]) -> Self { Self { endpoint: value[0] as u8, status: DmaStatus(value[1]) } }
}
