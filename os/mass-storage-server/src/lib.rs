// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

#![cfg(keyos)]

pub mod api;
pub mod error;
mod implementation;
pub mod messages;

pub use error::MassStorageError;
use server::{AsScalar, FromScalar};

pub fn listen() { server::listen(implementation::MassStorageServer::default()); }

#[derive(Debug, Copy, Clone)]
pub enum MassStorageEvent {
    Connect { block_size: usize, block_count: usize },
    Disconnect,
}

impl FromScalar<3> for MassStorageEvent {
    fn from_scalar(value: [u32; 3]) -> Self {
        match value[0] {
            0 => Self::Connect { block_size: value[1] as usize, block_count: value[2] as usize },
            _ => Self::Disconnect,
        }
    }
}

impl AsScalar<3> for MassStorageEvent {
    fn as_scalar(&self) -> [u32; 3] {
        match self {
            MassStorageEvent::Connect { block_size, block_count } => {
                [0, *block_size as u32, *block_count as u32]
            }
            MassStorageEvent::Disconnect => [1, 0, 0],
        }
    }
}
