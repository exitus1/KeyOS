// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

/// The command completed successfully without error.
const SW_NO_ERROR: [u8; 2] = [0x90, 0x00];
/// The request was rejected due to test-of-user-presence being required.
const SW_CONDITIONS_NOT_SATISFIED: [u8; 2] = [0x69, 0x85];
/// The request was rejected due to an invalid key handle.
const SW_WRONG_DATA: [u8; 2] = [0x6A, 0x80];
/// The length of the request was invalid.
const SW_WRONG_LENGTH: [u8; 2] = [0x67, 0x00];
/// The Class byte of the request is not supported.
const SW_CLA_NOT_SUPPORTED: [u8; 2] = [0x6E, 0x00];
/// The Instruction of the request is not supported.
const SW_INS_NOT_SUPPORTED: [u8; 2] = [0x6D, 0x00];
/// The Parameter of the request is not supported.
const SW_WRONG_P1P2: [u8; 2] = [0x6B, 0x00];
/// An unknown error occurred.
const SW_UNKNOWN: [u8; 2] = [0x6F, 0x00];

#[derive(Debug, PartialEq)]
pub enum Error {
    ConditionNotSatified,
    WrongData,
    WrongLength,
    ClassNotSupported,
    InstructionNotSupported(u8),
    WrongParameter,
    Other,
    MemoryMapping,
    Hashing,
    Signing,
}

impl From<crate::FidoError> for Error {
    fn from(e: crate::FidoError) -> Error {
        match e {
            crate::FidoError::InvalidIndex => Error::ConditionNotSatified,
            _ => Error::Other,
        }
    }
}

pub struct Status([u8; 2]);

impl From<Status> for [u8; 2] {
    fn from(s: Status) -> [u8; 2] { s.0 }
}

impl<T> From<&Result<T, Error>> for Status {
    fn from(r: &Result<T, Error>) -> Status {
        match r {
            Ok(_) => Status(SW_NO_ERROR),
            Err(Error::ConditionNotSatified) => Status(SW_CONDITIONS_NOT_SATISFIED),
            Err(Error::WrongData) => Status(SW_WRONG_DATA),
            Err(Error::WrongLength) => Status(SW_WRONG_LENGTH),
            Err(Error::ClassNotSupported) => Status(SW_CLA_NOT_SUPPORTED),
            Err(Error::InstructionNotSupported(_)) => Status(SW_INS_NOT_SUPPORTED),
            Err(Error::WrongParameter) => Status(SW_WRONG_P1P2),
            Err(Error::Other) => Status(SW_UNKNOWN),
            Err(Error::MemoryMapping) => Status(SW_UNKNOWN),
            Err(Error::Hashing) => Status(SW_UNKNOWN),
            Err(Error::Signing) => Status(SW_UNKNOWN),
        }
    }
}

impl Status {
    pub fn to_vec(&self, payload: &[u8]) -> Vec<u8> {
        let mut v = payload.to_vec();
        v.extend_from_slice(&self.0);
        v
    }
}
