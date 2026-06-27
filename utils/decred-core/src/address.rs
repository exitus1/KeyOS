// SPDX-License-Identifier: Apache-2.0
//! Decred v0 P2PKH addresses (the only type this signer handles).
//!
//! dcrd mainnet constants (`chaincfg/mainnetparams.go`):
//!   PubKeyHashAddrID = 0x073f  → "Ds…"
//! Payment script (`txscript/stdaddr`): DUP HASH160 <20> EQUALVERIFY CHECKSIG.

use secp256k1::Secp256k1;

use crate::hashing::{check_decode, check_encode, hash160, DecodeError};
use crate::hd::ExtPrivKey;

/// Mainnet pay-to-pubkey-hash (ecdsa-secp256k1) address ID.
pub const PKH_ADDR_ID_MAINNET: [u8; 2] = [0x07, 0x3f];

const OP_DUP: u8 = 0x76;
const OP_HASH160: u8 = 0xa9;
const OP_DATA_20: u8 = 0x14;
const OP_EQUALVERIFY: u8 = 0x88;
const OP_CHECKSIG: u8 = 0xac;

/// Build the 25-byte v0 P2PKH script for a 20-byte pubkey hash.
pub fn p2pkh_script(hash160: &[u8; 20]) -> [u8; 25] {
    let mut s = [0u8; 25];
    s[0] = OP_DUP;
    s[1] = OP_HASH160;
    s[2] = OP_DATA_20;
    s[3..23].copy_from_slice(hash160);
    s[23] = OP_EQUALVERIFY;
    s[24] = OP_CHECKSIG;
    s
}

/// Encode a mainnet "Ds…" address from a compressed pubkey.
pub fn p2pkh_from_pubkey(pubkey: &[u8]) -> String {
    check_encode(&hash160(pubkey), PKH_ADDR_ID_MAINNET)
}

/// Address for an HD key.
pub fn p2pkh_from_key(secp: &Secp256k1<secp256k1::All>, key: &ExtPrivKey) -> String {
    p2pkh_from_pubkey(&key.compressed_pubkey(secp))
}

/// Decode a mainnet P2PKH address to its 20-byte hash, validating prefix + checksum.
pub fn decode_p2pkh(addr: &str) -> Result<[u8; 20], DecodeError> {
    let (prefix, payload) = check_decode(addr)?;
    if prefix != PKH_ADDR_ID_MAINNET || payload.len() != 20 {
        return Err(DecodeError::BadChecksum);
    }
    let mut h = [0u8; 20];
    h.copy_from_slice(&payload);
    Ok(h)
}

#[cfg(test)]
mod tests {
    use super::*;

    const HASH160: [u8; 20] = [
        0x27, 0x89, 0xd5, 0x8c, 0xfa, 0x09, 0x57, 0xd2, 0x06, 0xf0, 0x25, 0xc2, 0xaf, 0x05, 0x6f,
        0xc8, 0xa7, 0x7c, 0xeb, 0xb0,
    ];

    #[test]
    fn p2pkh_script_matches_dcrd() {
        // dcrd payScript: 76a914<hash160>88ac
        let s = p2pkh_script(&HASH160);
        let hex: String = s.iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(hex, "76a9142789d58cfa0957d206f025c2af056fc8a77cebb088ac");
    }

    #[test]
    fn decode_roundtrip() {
        let addr = check_encode(&HASH160, PKH_ADDR_ID_MAINNET);
        assert_eq!(decode_p2pkh(&addr).unwrap(), HASH160);
    }
}
