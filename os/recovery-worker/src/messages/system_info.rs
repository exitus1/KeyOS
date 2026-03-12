// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

#[derive(Debug, Clone, server::Message, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[response(BootloaderInfo)]
pub struct GetBootloaderInfo;

#[derive(Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct BootloaderInfo {
    pub hash: [u8; 32],
    pub hash_str: String,
}

#[derive(Debug, Clone, server::Message, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[response(KeyOsInfo)]
pub struct GetKeyOsInfo;

#[derive(Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct KeyOsInfo {
    pub hash: [u8; 32],
    pub hash_str: String,
    pub date_str: String,
    pub version: String,
}

#[derive(Debug, Clone, server::Message, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[response(RecoveryInfo)]
pub struct GetRecoveryInfo;

#[derive(Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct RecoveryInfo {
    pub hash: [u8; 32],
    pub hash_str: String,
    pub date_str: String,
    pub version: String,
}
