// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

pub mod api;
mod command;
pub mod error;
mod header;
mod implementation;
pub mod messages;

pub fn listen() { server::listen(implementation::CtapHidServer::new().unwrap()) }
