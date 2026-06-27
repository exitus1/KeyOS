// SPDX-License-Identifier: Apache-2.0
//! Decred hashing helpers built on BLAKE-256.
//!
//! * `hash160` = `ripemd160(blake256(x))` — the pubkey/script hash used in
//!   addresses (dcrd `txscript/stdaddr.Hash160`).
//! * base58check uses a **double-BLAKE256** checksum (`blake256(blake256)[..4]`),
//!   NOT Bitcoin's double-SHA256 (`github.com/decred/base58`).

use ripemd::{Digest, Ripemd160};

use crate::blake256;

/// `ripemd160(blake256(buf))` → 20 bytes.
pub fn hash160(buf: &[u8]) -> [u8; 20] {
    let b = blake256::sum256(buf);
    let mut h = Ripemd160::new();
    h.update(b);
    let out = h.finalize();
    let mut r = [0u8; 20];
    r.copy_from_slice(&out);
    r
}

/// base58check-encode `[version_prefix || payload]` with a 4-byte
/// double-BLAKE256 checksum. `prefix` is the 2-byte Decred net/address ID.
pub fn check_encode(payload: &[u8], prefix: [u8; 2]) -> String {
    let mut buf = Vec::with_capacity(2 + payload.len() + 4);
    buf.extend_from_slice(&prefix);
    buf.extend_from_slice(payload);
    let cksum = blake256::sum256d(&buf);
    buf.extend_from_slice(&cksum[..4]);
    bs58::encode(buf).into_string()
}

#[derive(Debug, PartialEq, Eq)]
pub enum DecodeError {
    Base58,
    TooShort,
    BadChecksum,
}

/// Decode and verify a base58check string, returning `(2-byte prefix, payload)`.
pub fn check_decode(s: &str) -> Result<([u8; 2], Vec<u8>), DecodeError> {
    let raw = bs58::decode(s).into_vec().map_err(|_| DecodeError::Base58)?;
    if raw.len() < 6 {
        return Err(DecodeError::TooShort);
    }
    let (body, cksum) = raw.split_at(raw.len() - 4);
    let expect = blake256::sum256d(body);
    if expect[..4] != *cksum {
        return Err(DecodeError::BadChecksum);
    }
    let prefix = [body[0], body[1]];
    Ok((prefix, body[2..].to_vec()))
}

#[cfg(test)]
mod tests {
    use super::*;

    // dcrd reference vector: payScript 76a914<hash160>88ac for
    // DsUZxxoHJSty8DCfwfartwTYbuhmVct7tJu (mainnet P2PKH prefix 0x073f).
    const HASH160: [u8; 20] = [
        0x27, 0x89, 0xd5, 0x8c, 0xfa, 0x09, 0x57, 0xd2, 0x06, 0xf0, 0x25, 0xc2, 0xaf, 0x05, 0x6f,
        0xc8, 0xa7, 0x7c, 0xeb, 0xb0,
    ];

    #[test]
    fn p2pkh_address_roundtrip() {
        let addr = check_encode(&HASH160, [0x07, 0x3f]);
        assert_eq!(addr, "DsUZxxoHJSty8DCfwfartwTYbuhmVct7tJu");
        let (prefix, payload) = check_decode(&addr).unwrap();
        assert_eq!(prefix, [0x07, 0x3f]);
        assert_eq!(payload, HASH160);
    }

    #[test]
    fn bad_checksum_rejected() {
        assert_eq!(
            check_decode("DsUZxxoHJSty8DCfwfartwTYbuhmVct7tJv"),
            Err(DecodeError::BadChecksum)
        );
    }
}
