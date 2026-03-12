// SPDX-FileCopyrightText: 2024-2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

//! QR code scanner navigation request and response formats.

/// Options for the QR Scanner navigation request.
///
/// Example with a left back arrow and a simple message:
///
/// ```rust
/// # use navigation::api::qrscanner::{ScanQrOptions};
/// let options = ScanQrOptions::default()
///     .with_start_location(Location::External)
///     .with_allowed_locations(AllowedLocations::specific(&[Location::External]))
///     .with_allowed_extensions(AllowedExtensions::specific(&["bin"]));
/// ```
#[derive(Debug, rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)]
pub struct ScanQrOptions {
    pub header_title: String,
    pub header_left_icon: String,
    pub header_left_text: String,
    pub header_right_icon: String,
    pub header_right_text: String,
    pub message: String,
    pub button_icon: String,
    pub button_text: String,
}

impl Default for ScanQrOptions {
    fn default() -> Self {
        Self {
            header_title: String::new(),
            header_left_icon: String::from("chevron-left"),
            header_left_text: String::new(),
            header_right_icon: String::new(),
            header_right_text: String::new(),
            message: String::new(),
            button_icon: String::new(),
            button_text: String::new(),
        }
    }
}

impl ScanQrOptions {
    pub fn new() -> Self {
        Self {
            header_title: String::new(),
            header_left_icon: String::new(),
            header_left_text: String::new(),
            header_right_icon: String::new(),
            header_right_text: String::new(),
            message: String::new(),
            button_icon: String::new(),
            button_text: String::new(),
        }
    }

    pub fn from_slice(data: &[u8]) -> Option<Self> {
        let Ok(archived) = rkyv::access::<ArchivedScanQrOptions, rkyv::rancor::Error>(data) else {
            return None;
        };
        rkyv::deserialize::<Self, rkyv::rancor::Error>(archived).ok()
    }

    pub fn serialize(&self) -> Vec<u8> { rkyv::to_bytes::<rkyv::rancor::Error>(self).unwrap().to_vec() }
}

#[derive(Debug, rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)]
pub enum ScanQrResult {
    Qr(Vec<u8>),
    Ur2(String, Vec<u8>),
    LeftClicked,
    RightClicked,
    ButtonClicked,
}

impl ScanQrResult {
    pub fn new_qr(data: &[u8]) -> Self { Self::Qr(data.to_vec()) }

    pub fn new_ur2(ur_type: String, data: &[u8]) -> Self { Self::Ur2(ur_type, data.to_vec()) }

    pub fn new_cancelled() -> Self { Self::LeftClicked }

    pub fn from_slice(data: &[u8]) -> Option<Self> {
        let Ok(archived) = rkyv::access::<ArchivedScanQrResult, rkyv::rancor::Error>(data) else {
            return None;
        };
        rkyv::deserialize::<Self, rkyv::rancor::Error>(archived).ok()
    }

    pub fn serialize(&self) -> Vec<u8> { rkyv::to_bytes::<rkyv::rancor::Error>(self).unwrap().to_vec() }
}
