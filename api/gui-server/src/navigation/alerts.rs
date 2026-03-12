// SPDX-FileCopyrightText: 2024-2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Various global alerts navigation.

#[derive(Debug, rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)]
pub struct InvokeAlert {
    pub app_title: Option<String>,
    pub title: String,
    pub icon: String,
    pub line1: String,
    pub line2: Option<String>,
    pub button1_title: String,
    pub button2_title: Option<String>,
    pub button3_title: Option<String>,
}

impl InvokeAlert {
    pub fn new_warning(
        title: &str,
        line1: &str,
        line2: &str,
        button1_title: &str,
        button2_title: &str,
    ) -> Self {
        InvokeAlert {
            app_title: None,
            title: title.to_string(),
            icon: "alert".to_string(),
            line1: line1.to_string(),
            line2: Some(line2.to_string()),
            button1_title: button1_title.to_string(),
            button2_title: Some(button2_title.to_string()),
            button3_title: None,
        }
    }
}

#[derive(Debug, rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)]
pub enum AlertResult {
    Button1Pressed,
    Button2Pressed,
    Button3Pressed,
    Canceled,
}

impl InvokeAlert {
    pub fn from_slice(data: &[u8]) -> Option<Self> {
        let Ok(archived) = rkyv::access::<ArchivedInvokeAlert, rkyv::rancor::Error>(data) else {
            return None;
        };
        rkyv::deserialize::<Self, rkyv::rancor::Error>(archived).ok()
    }

    pub fn serialize(&self) -> Vec<u8> { rkyv::to_bytes::<rkyv::rancor::Error>(self).unwrap().to_vec() }
}

impl AlertResult {
    pub fn from_slice(data: &[u8]) -> Option<Self> {
        let Ok(archived) = rkyv::access::<ArchivedAlertResult, rkyv::rancor::Error>(data) else {
            return None;
        };
        rkyv::deserialize::<Self, rkyv::rancor::Error>(archived).ok()
    }

    pub fn serialize(&self) -> Vec<u8> { rkyv::to_bytes::<rkyv::rancor::Error>(self).unwrap().to_vec() }
}
