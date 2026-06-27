// SPDX-License-Identifier: Apache-2.0
//! ECDSA signing for Decred P2PKH inputs.
//!
//! Standard Decred spends use DER-encoded ECDSA secp256k1 signatures with the
//! sighash-type byte appended, pushed alongside the compressed pubkey:
//! `sigScript = PUSH(der_sig ‖ hashType) PUSH(compressed_pubkey)`.
//!
//! Signatures are RFC6979-deterministic and low-S normalized (consensus
//! requires canonical S; the `secp256k1` crate normalizes on signing).

use secp256k1::{ecdsa::Signature, Message, Secp256k1, SecretKey};

use crate::sighash::{signature_hash_all, SIGHASH_ALL};
use crate::tx::MsgTx;
use crate::Error;

/// Sign input `idx` (P2PKH) and return the complete signature script.
pub fn sign_p2pkh_input(
    secp: &Secp256k1<secp256k1::All>,
    tx: &MsgTx,
    idx: usize,
    prevout_script: &[u8],
    secret: &SecretKey,
    compressed_pubkey: &[u8; 33],
) -> Result<Vec<u8>, Error> {
    let sighash = signature_hash_all(tx, idx, prevout_script)?;
    let msg = Message::from_digest(sighash);
    let sig: Signature = secp.sign_ecdsa(&msg, secret);
    // Defense in depth — normalize even though sign_ecdsa already produces low-S.
    let mut sig = sig;
    sig.normalize_s();

    let mut sig_bytes = sig.serialize_der().to_vec();
    sig_bytes.push(SIGHASH_ALL as u8); // append hashtype

    Ok(build_sig_script(&sig_bytes, compressed_pubkey))
}

/// `PUSH(sig) PUSH(pubkey)` with canonical single-byte pushes (both operands
/// are < 76 bytes, so the length byte is the push opcode).
fn build_sig_script(sig_with_type: &[u8], pubkey: &[u8]) -> Vec<u8> {
    let mut s = Vec::with_capacity(2 + sig_with_type.len() + pubkey.len());
    debug_assert!(sig_with_type.len() < 76 && pubkey.len() < 76);
    s.push(sig_with_type.len() as u8);
    s.extend_from_slice(sig_with_type);
    s.push(pubkey.len() as u8);
    s.extend_from_slice(pubkey);
    s
}
