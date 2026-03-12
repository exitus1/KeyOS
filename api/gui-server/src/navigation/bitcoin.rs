// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

#[derive(Debug, rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)]
pub struct OpenBitcoinOptions {
    pub action: BitcoinAction,
}

#[derive(Debug, rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)]
pub enum BitcoinAction {
    Scan,
}

impl Default for OpenBitcoinOptions {
    fn default() -> Self { Self { action: BitcoinAction::Scan } }
}

impl OpenBitcoinOptions {
    pub fn new() -> Self { Self::default() }

    pub fn from_slice(data: &[u8]) -> Option<Self> {
        let Ok(archived) = rkyv::access::<ArchivedOpenBitcoinOptions, rkyv::rancor::Error>(data) else {
            return None;
        };
        rkyv::deserialize::<Self, rkyv::rancor::Error>(archived).ok()
    }

    pub fn serialize(&self) -> Vec<u8> { rkyv::to_bytes::<rkyv::rancor::Error>(self).unwrap().to_vec() }
}
