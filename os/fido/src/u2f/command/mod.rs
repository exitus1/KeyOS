// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

mod authenticate;
mod register;
mod version;

pub use authenticate::{AuthenticateRequest, AuthenticateResponse};
pub use register::{RegisterRequest, RegisterResponse};
pub use version::VersionResponse;

use super::error::Error;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Command {
    Register,
    Authenticate,
    Version,
}
impl TryFrom<u8> for Command {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Error> {
        match value {
            0x01 => Ok(Command::Register),
            0x02 => Ok(Command::Authenticate),
            0x03 => Ok(Command::Version),
            v => Err(Error::InstructionNotSupported(v)),
        }
    }
}

#[derive(Debug)]
pub struct KeyHandle {
    pub security_key_index: usize,
    pub registered_key_index: usize,
}
impl KeyHandle {
    pub(crate) fn to_vec(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&self.security_key_index.to_be_bytes());
        data.extend_from_slice(&self.registered_key_index.to_be_bytes());
        data
    }

    pub(crate) fn from_bytes(data: &[u8; 8]) -> Self {
        Self {
            security_key_index: usize::from_be_bytes(data[..4].try_into().unwrap()),
            registered_key_index: usize::from_be_bytes(data[4..].try_into().unwrap()),
        }
    }
}
