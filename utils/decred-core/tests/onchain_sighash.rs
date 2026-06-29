// SPDX-License-Identifier: Apache-2.0
//
//! Independent correctness oracle for the sighash + tx parsing path.
//!
//! This is the strongest test in the crate: it takes a REAL Decred mainnet
//! transaction (one the live network already validated and mined), recomputes
//! our `signature_hash_all` for each input, and checks that the signature
//! embedded in the on-chain witness verifies against *our* sighash. If our
//! sighash byte layout disagreed with dcrd by even one byte, the on-chain
//! signature would fail to verify here.
//!
//! Fixtures (mainnet, fetched from dcrdata):
//!   spending tx  37564c16ef112d03c1fd44df93c0fd2703b057580797de6489463bcabfe5d954
//!   both inputs spend P2PKH outputs with pkScript 76a914…43f488ac

use decred_core::sighash::signature_hash_all;
use decred_core::tx::MsgTx;
use secp256k1::{ecdsa::Signature, Message, PublicKey, Secp256k1};

fn hexb(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}

// Full (prefix+witness) serialization of the mainnet spending tx.
const SPEND_TX_HEX: &str = "0100000002fb224c02e29fe8ef01db6642ac95859275afb98054bb7ced04afb9a8e1f7a6450100000000ffffffffb74693e2169e31e13cc6f8173cbba1de1ab510c3ad07aebdb4ede4b093a6cfcc0100000000ffffffff02a08601000000000000001976a9143afaebcdfd8cda72e687f0e4f72f8f0a6b14bb9f88ac741d00000000000000001976a914375e93a7837bb62d3d1ce1193476431661bc75ea88ac000000000000000002387601000000000001ab1000100000006a473044022032ee361b77874b6a69dc4764631231b2f571c3f8fc2027972f39a835a694d5e002201502b44f5833c672881cbcf21f2b2f7967b401bda8987f55d5b42b7d9d130a760121034acc9646d41489c7756bfd6e74ef9fb8928a54b27bfb5ca602a26c47af2558b21c3e0000000000000dab1000060000006b483045022100dc8bffcbdfc240195f30e47de9855978f0facd9dba24665fe83781421884a952022020f63bcfc76a80d1c705cf85eeb0b399a2148924e5a13b0ec6c271c7b3bdd6d0012102f1c87bee1183a241b857e97f05d0fac9d0a1ea32b1b0cc59b20e7bd12a986125";

// Prevout pkScript for each input (both spend the same address here).
const PREVOUT_SCRIPTS: [&str; 2] = [
    "76a91407151738f9d10dd5912dc26e6fc5606daebb43f488ac",
    "76a914a055c8f4f3d5a173d1aa3051fa11c305ab59bd6d88ac",
];

/// Split a standard P2PKH sigScript `PUSH(der||hashtype) PUSH(pubkey)` into its
/// DER signature, the trailing sighash-type byte, and the compressed pubkey.
fn split_sig_script(ss: &[u8]) -> (Vec<u8>, u8, Vec<u8>) {
    let l1 = ss[0] as usize;
    let sig_and_type = &ss[1..1 + l1];
    let hashtype = sig_and_type[l1 - 1];
    let der = sig_and_type[..l1 - 1].to_vec();
    let l2 = ss[1 + l1] as usize;
    let pubkey = ss[2 + l1..2 + l1 + l2].to_vec();
    (der, hashtype, pubkey)
}

#[test]
fn onchain_tx_signatures_verify_against_our_sighash() {
    let secp = Secp256k1::new();
    let tx = MsgTx::parse_full(&hexb(SPEND_TX_HEX)).expect("parse mainnet tx");
    assert_eq!(tx.tx_in.len(), 2, "fixture has two inputs");

    for (idx, prevout_hex) in PREVOUT_SCRIPTS.iter().enumerate() {
        let prevout_script = hexb(prevout_hex);
        let (der, hashtype, pubkey) = split_sig_script(&tx.tx_in[idx].signature_script);
        assert_eq!(hashtype, 0x01, "input {idx} is SigHashAll");

        let sighash = signature_hash_all(&tx, idx, &prevout_script).expect("sighash");
        let msg = Message::from_digest(sighash);
        let mut sig = Signature::from_der(&der).expect("der sig");
        sig.normalize_s();
        let pk = PublicKey::from_slice(&pubkey).expect("pubkey");

        secp.verify_ecdsa(&msg, &sig, &pk).unwrap_or_else(|e| {
            panic!("input {idx}: on-chain signature failed against our sighash: {e}")
        });
    }
}

/// Round-trips the same real tx through our parser and serializer: parse_full →
/// serialize_full must reproduce the exact network bytes.
#[test]
fn onchain_tx_serialize_roundtrip() {
    let raw = hexb(SPEND_TX_HEX);
    let tx = MsgTx::parse_full(&raw).expect("parse");
    assert_eq!(tx.serialize_full(), raw, "serialize_full must be byte-exact");
}

/// The txid (BLAKE256 of the prefix serialization) must match the network txid.
#[test]
fn onchain_txid_matches() {
    let tx = MsgTx::parse_full(&hexb(SPEND_TX_HEX)).expect("parse");
    // Network/display txid is the reverse of the internal hash.
    let mut id = tx.tx_hash();
    id.reverse();
    assert_eq!(
        hex::encode(id),
        "37564c16ef112d03c1fd44df93c0fd2703b057580797de6489463bcabfe5d954"
    );
}
