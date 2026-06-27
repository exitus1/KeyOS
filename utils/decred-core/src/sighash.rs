// SPDX-License-Identifier: Apache-2.0
//! Decred signature hash — `txscript/sighash.go` `calcSignatureHash`.
//!
//! This is **not** Bitcoin's BIP143. The message signed for input `idx` is:
//!
//! ```text
//! prefixHash  = blake256( version|(1<<16) ‖ inputs(outpoint+seq) ‖ outputs ‖ locktime ‖ expiry )
//! witnessHash = blake256( version|(3<<16) ‖ count ‖ per-input: signScript for idx else empty )
//! sighash     = blake256( LE32(hashType) ‖ prefixHash ‖ witnessHash )
//! ```
//!
//! `signScript` is the prevout pkScript being spent (for our P2PKH inputs, the
//! 25-byte DUP HASH160 … CHECKSIG script). Only SigHashAll is implemented —
//! a send/receive wallet never needs the other modes.

use crate::blake256;
use crate::tx::{put_varint, MsgTx};
use crate::Error;

pub const SIGHASH_ALL: u32 = 0x1;

const SIGHASH_SERIALIZE_PREFIX: u32 = 1;
const SIGHASH_SERIALIZE_WITNESS: u32 = 3;

/// Compute the SigHashAll signature hash for input `idx`, spending a prevout
/// whose pkScript is `sign_script`.
pub fn signature_hash_all(
    tx: &MsgTx,
    idx: usize,
    sign_script: &[u8],
) -> Result<[u8; 32], Error> {
    if idx >= tx.tx_in.len() {
        return Err(Error::SigHashIndex);
    }

    // ---- prefix hash (commits to all inputs and all outputs) ----
    let mut prefix = Vec::new();
    let pver = (tx.version as u32) | (SIGHASH_SERIALIZE_PREFIX << 16);
    prefix.extend_from_slice(&pver.to_le_bytes());
    put_varint(&mut prefix, tx.tx_in.len() as u64);
    for ti in &tx.tx_in {
        prefix.extend_from_slice(&ti.previous_outpoint.hash);
        prefix.extend_from_slice(&ti.previous_outpoint.index.to_le_bytes());
        prefix.push(ti.previous_outpoint.tree);
        prefix.extend_from_slice(&ti.sequence.to_le_bytes());
    }
    put_varint(&mut prefix, tx.tx_out.len() as u64);
    for to in &tx.tx_out {
        prefix.extend_from_slice(&(to.value as u64).to_le_bytes());
        prefix.extend_from_slice(&to.version.to_le_bytes());
        put_varint(&mut prefix, to.pk_script.len() as u64);
        prefix.extend_from_slice(&to.pk_script);
    }
    prefix.extend_from_slice(&tx.lock_time.to_le_bytes());
    prefix.extend_from_slice(&tx.expiry.to_le_bytes());
    let prefix_hash = blake256::sum256(&prefix);

    // ---- witness hash (commits sign_script for idx, empty for the rest) ----
    let mut witness = Vec::new();
    let wver = (tx.version as u32) | (SIGHASH_SERIALIZE_WITNESS << 16);
    witness.extend_from_slice(&wver.to_le_bytes());
    put_varint(&mut witness, tx.tx_in.len() as u64);
    for (i, _) in tx.tx_in.iter().enumerate() {
        if i == idx {
            put_varint(&mut witness, sign_script.len() as u64);
            witness.extend_from_slice(sign_script);
        } else {
            put_varint(&mut witness, 0);
        }
    }
    let witness_hash = blake256::sum256(&witness);

    // ---- final hash ----
    let mut buf = Vec::with_capacity(4 + 64);
    buf.extend_from_slice(&SIGHASH_ALL.to_le_bytes());
    buf.extend_from_slice(&prefix_hash);
    buf.extend_from_slice(&witness_hash);
    Ok(blake256::sum256(&buf))
}
