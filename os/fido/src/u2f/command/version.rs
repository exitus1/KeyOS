// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

#[derive(Debug)]
pub struct VersionResponse {
    pub version: String,
}
impl VersionResponse {
    pub fn prime() -> Self {
        Self {
            version: "U2F_V2".to_string(), // CTAP1/U2F
        }
    }

    pub fn to_vec(&self) -> Vec<u8> { self.version.as_bytes().to_vec() }
}
