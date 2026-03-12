// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use xous::APP_ID_SIZE;

use crate::{error::NavigationError, ModalStyle};

#[derive(Debug, server::Message, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[response(NavigationResult)]
pub struct ShowModal {
    pub modal_style: ModalStyle,
    pub app_id: [u8; APP_ID_SIZE],
    pub args: Vec<u8>,
}

#[derive(Debug, server::Message, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[response(NavigationResult)]
pub struct NavigateTo {
    pub app_id: [u8; APP_ID_SIZE],
    pub args: Vec<u8>,
}

#[derive(Debug, server::Message, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[response(())]
pub struct FinishResponse {
    pub response: Vec<u8>,
}

impl FinishResponse {
    pub fn as_slice(&self) -> &[u8] { &self.response }
}

pub type NavigationResult = Result<FinishResponse, NavigationError>;

#[derive(Debug, server::Message)]
pub struct NavigationCancel;

#[derive(Debug, server::Message, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[response(Option<Vec<u8>>)]
pub struct GetPendingNavRequest;

#[derive(Debug, Copy, Clone, server::Message)]
pub struct LoginSuccess;
