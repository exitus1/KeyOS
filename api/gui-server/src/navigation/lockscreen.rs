// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

#[derive(Debug, rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)]
pub struct VerifyPinOptions {
    pub title: Option<String>,
    pub want_security_words: bool,
}

impl VerifyPinOptions {
    pub fn from_slice(data: &[u8]) -> Option<Self> {
        let Ok(archived) = rkyv::access::<ArchivedVerifyPinOptions, rkyv::rancor::Error>(data) else {
            return None;
        };
        rkyv::deserialize::<Self, rkyv::rancor::Error>(archived).ok()
    }

    pub fn serialize(&self) -> Vec<u8> { rkyv::to_bytes::<rkyv::rancor::Error>(self).unwrap().to_vec() }
}

#[derive(Debug, rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)]
pub struct VerifyPinResult {
    pub success: bool,
    pub security_words: Option<[String; 2]>,
}

impl VerifyPinResult {
    pub fn from_slice(data: &[u8]) -> Option<Self> {
        let Ok(archived) = rkyv::access::<ArchivedVerifyPinResult, rkyv::rancor::Error>(data) else {
            return None;
        };
        rkyv::deserialize::<Self, rkyv::rancor::Error>(archived).ok()
    }

    pub fn serialize(&self) -> Vec<u8> { rkyv::to_bytes::<rkyv::rancor::Error>(self).unwrap().to_vec() }
}
