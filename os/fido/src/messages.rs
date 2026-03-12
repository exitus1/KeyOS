// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use server::{AsScalar, FromScalar};

use crate::error::FidoError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub enum Transport {
    Usb,
    Nfc,
}

// === External messages ===

#[derive(Debug, server::Message)]
#[response(Result<bool, FidoError>)]
pub struct IsLive(pub usize);

#[derive(Debug, server::Message)]
#[response(Option<usize>)]
pub struct GetSelectedSecurityKey;

/// Fire-and-forget message for selecting a security key.
#[derive(Debug, server::Message)]
pub struct SelectSecurityKey(pub Option<usize>);

/// Fire-and-forget message for creating a security key.
#[derive(Debug, server::Message)]
pub struct CreateSecurityKey;

impl AsScalar<0> for CreateSecurityKey {
    fn as_scalar(&self) -> [u32; 0] { [] }
}
impl FromScalar<0> for CreateSecurityKey {
    fn from_scalar(_: [u32; 0]) -> Self { Self }
}

#[derive(Debug, server::Message)]
#[response(usize)]
pub struct NextSecurityKeyIndex;

/// Fire-and-forget message for setting security key liveness.
#[derive(Debug, server::Message)]
pub struct SetLive {
    pub index: usize,
    pub live: bool,
}

impl AsScalar<2> for SetLive {
    fn as_scalar(&self) -> [u32; 2] { [self.index as u32, self.live as u32] }
}
impl FromScalar<2> for SetLive {
    fn from_scalar([a, b]: [u32; 2]) -> Self { Self { index: a as usize, live: b != 0 } }
}

#[derive(Debug, Clone, server::Message, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[response(Vec<u8>)]
pub struct U2fProcessApdu {
    pub msg: Vec<u8>,
    pub transport: Transport,
}

#[derive(Debug, Clone, server::Message, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[response(Vec<u8>)]
pub struct CtapProcessCbor {
    pub cmd: u8,
    pub raw: Vec<u8>,
}

// === Test messages ===

#[cfg(feature = "test-app")]
#[derive(Debug, server::Message)]
#[response(Result<(), FidoError>)]
pub struct ResetState;
