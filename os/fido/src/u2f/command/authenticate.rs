// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{Error, KeyHandle};

#[derive(Debug)]
pub struct AuthenticateRequest {
    pub challenge_parameter: [u8; 32],
    pub application_parameter: [u8; 32],
    pub key_handle: KeyHandle,
}
impl AuthenticateRequest {
    pub fn from_apdu(data: &[u8]) -> Result<AuthenticateRequest, Error> {
        if data.len() <= 64 {
            return Err(Error::WrongLength);
        }
        let mut data_len = data[0] as usize;
        let data_offset = if data_len == 0 {
            data_len = (data[1] as usize) * 256 + data[2] as usize;
            3
        } else {
            1
        };
        if data.len() <= 64 + data_offset {
            return Err(Error::WrongLength);
        }
        let key_handle_len = data[data_offset + 64] as usize;
        if key_handle_len != 8 {
            return Err(Error::WrongLength);
        }
        if data_len != 64 + 1 + key_handle_len {
            return Err(Error::WrongLength);
        }
        if data.len() < 64 + data_offset + 1 + key_handle_len {
            return Err(Error::WrongLength);
        }
        Ok(AuthenticateRequest {
            challenge_parameter: data[data_offset..data_offset + 32].try_into().unwrap(),
            application_parameter: data[data_offset + 32..data_offset + 64].try_into().unwrap(),
            key_handle: KeyHandle::from_bytes(
                &data[data_offset + 64 + 1..data_offset + 64 + 1 + key_handle_len].try_into().unwrap(),
            ),
        })
    }
}

#[derive(Debug)]
pub struct AuthenticateResponse {
    pub user_presence: u8,
    pub counter: u32,
    pub signature: Vec<u8>,
}
impl AuthenticateResponse {
    pub fn new(user_present: bool) -> AuthenticateResponse {
        AuthenticateResponse {
            user_presence: if user_present { 1 } else { 0 },
            counter: 0,
            signature: Vec::new(),
        }
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let mut data = vec![self.user_presence];
        data.extend_from_slice(&self.counter.to_be_bytes());
        data.extend_from_slice(&self.signature);
        data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_authenticate_request() {
        let data_authenticate_request = [
            0x00, 0x00, 0x81, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42,
            0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42,
            0x42, 0x42, 0x42, 0x20, 0xa8, 0x3b, 0x42, 0xcd, 0x54, 0x0c, 0x2f, 0xcd, 0xee, 0x61, 0xe4, 0xf1,
            0x47, 0x93, 0xed, 0xe7, 0x74, 0xfa, 0x82, 0x58, 0x83, 0x56, 0x83, 0xa7, 0xc8, 0xe8, 0x85, 0xa8,
            0xc2, 0x70, 0xc2, 0x40, 0x57, 0x02, 0x7b, 0xf4, 0x94, 0x97, 0xb6, 0x84, 0x2a, 0x9b, 0x65, 0x8e,
            0xe8, 0x9f, 0x6e, 0x71, 0x8e, 0x2b, 0xac, 0x63, 0xe7, 0xd7, 0xc8, 0xc7, 0x3f, 0x85, 0xe3, 0x20,
            0xcc, 0xca, 0xd6, 0xa1, 0x38, 0x50, 0x56, 0x06, 0x20, 0xd1, 0x60, 0x87, 0x4d, 0x46, 0x93, 0xaf,
            0xb1, 0x23, 0xfd, 0x0b, 0x2f, 0x87, 0xe6, 0xf7, 0x6e, 0xd3, 0xad, 0x95, 0xc1, 0x2d, 0x2d, 0x72,
            0x4b, 0x60, 0x4c, 0x86, 0x00, 0x00,
        ];
        let authenticate_request = AuthenticateRequest::from_apdu(&data_authenticate_request);
        println!("{:02x?}", authenticate_request);
        assert!(authenticate_request.is_err());
        assert_eq!(authenticate_request.unwrap_err(), Error::WrongLength);
    }
}
