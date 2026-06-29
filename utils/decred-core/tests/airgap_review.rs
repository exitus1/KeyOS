// SPDX-License-Identifier: Apache-2.0
//
//! Tests for the air-gap package and the trustless-review logic — the security
//! core of the app. Covers: CBOR round-trip + version gating, ownership
//! classification (change vs recipient vs mislabelled-change), end-to-end
//! signing self-consistency, and the anti-tamper prev_script tripwire.

use decred_core::address::p2pkh_script;
use decred_core::airgap::{
    decode_sign_request, encode_sign_request, sign_request, InputMeta, OutputMeta, SignRequest,
    FORMAT_VERSION,
};
use decred_core::hashing::hash160;
use decred_core::hd::{ExtPrivKey, BRANCH_EXTERNAL};
use decred_core::sighash::signature_hash_all;
use decred_core::tx::MsgTx;
use decred_core::Error;
use secp256k1::{ecdsa::Signature, Message, PublicKey, Secp256k1};

const ENTROPY_HEX: &str = "348360ae0a69b1883b0dfc060136108dfcabe9f4bf8af3e866b742fb53f1caa5";

fn master() -> ExtPrivKey {
    let entropy: Vec<u8> = (0..ENTROPY_HEX.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&ENTROPY_HEX[i..i + 2], 16).unwrap())
        .collect();
    ExtPrivKey::from_entropy(&entropy, "").unwrap()
}

/// A 25-byte P2PKH script that is NOT ours (arbitrary hash160).
fn foreign_script(tag: u8) -> Vec<u8> {
    p2pkh_script(&[tag; 20]).to_vec()
}

#[test]
fn cbor_roundtrip_and_version_gate() {
    let req = SignRequest {
        format_version: FORMAT_VERSION,
        tx_version: 1,
        account: 0,
        lock_time: 0,
        expiry: 0,
        inputs: vec![InputMeta {
            prev_hash: [7u8; 32],
            prev_index: 1,
            tree: 0,
            sequence: 0xffff_ffff,
            value_in: 12345,
            branch: 0,
            index: 0,
            prev_script: foreign_script(0xaa),
        }],
        outputs: vec![OutputMeta {
            value: 12000,
            version: 0,
            pk_script: foreign_script(0xbb),
            is_change: false,
        }],
    };
    let bytes = encode_sign_request(&req).unwrap();
    let back = decode_sign_request(&bytes).unwrap();
    assert_eq!(back.inputs.len(), 1);
    assert_eq!(back.outputs[0].value, 12000);
    assert_eq!(back.inputs[0].value_in, 12345);

    // A package declaring an unknown FORMAT_VERSION must be rejected.
    let mut bad = req;
    bad.format_version = FORMAT_VERSION + 1;
    let bad_bytes = encode_sign_request(&bad).unwrap();
    assert!(matches!(
        decode_sign_request(&bad_bytes),
        Err(Error::UnsupportedVersion)
    ));
}

#[test]
fn review_owned_classifies_change_recipient_and_mislabel() {
    let secp = Secp256k1::new();
    let m = master();

    // Our own change address (account 0, external/3) — device should recognize
    // it as change regardless of the companion's flag.
    let acct = m.account_key(&secp, 0).unwrap();
    let own = acct.address_key(&secp, BRANCH_EXTERNAL, 3).unwrap();
    let own_script = p2pkh_script(&hash160(&own.compressed_pubkey(&secp))).to_vec();

    let req = SignRequest {
        format_version: FORMAT_VERSION,
        tx_version: 1,
        account: 0,
        lock_time: 0,
        expiry: 0,
        inputs: vec![InputMeta {
            prev_hash: [1u8; 32],
            prev_index: 0,
            tree: 0,
            sequence: 0xffff_ffff,
            value_in: 10_000,
            branch: 0,
            index: 0,
            prev_script: own_script.clone(),
        }],
        outputs: vec![
            // Owned, but companion DIDN'T flag it as change — device must still
            // count it as change (it pays us).
            OutputMeta { value: 1_000, version: 0, pk_script: own_script, is_change: false },
            // Genuine external recipient.
            OutputMeta { value: 5_000, version: 0, pk_script: foreign_script(0xcc), is_change: false },
            // Foreign address the companion LIED about (claimed change). This is
            // the attack the trustless review exists to catch.
            OutputMeta { value: 2_000, version: 0, pk_script: foreign_script(0xdd), is_change: true },
        ],
    };

    let summary = req.review_owned(&secp, &m, 20).unwrap();
    assert_eq!(summary.change_total, 1_000, "only the owned output is change");
    assert_eq!(summary.recipients.len(), 2, "both foreign outputs are recipients");
    assert_eq!(summary.flagged_mismatches.len(), 1, "the mislabelled output is flagged");
    assert_eq!(summary.flagged_mismatches[0].1, 2_000);
    assert_eq!(summary.fee, 10_000 - (1_000 + 5_000 + 2_000));
}

#[test]
fn sign_request_is_self_consistent_and_low_s() {
    let secp = Secp256k1::new();
    let m = master();
    let acct = m.account_key(&secp, 0).unwrap();
    let key0 = acct.address_key(&secp, BRANCH_EXTERNAL, 0).unwrap();
    let pk0 = key0.compressed_pubkey(&secp);
    let script0 = p2pkh_script(&hash160(&pk0)).to_vec();

    let req = SignRequest {
        format_version: FORMAT_VERSION,
        tx_version: 1,
        account: 0,
        lock_time: 0,
        expiry: 0,
        inputs: vec![InputMeta {
            prev_hash: [9u8; 32],
            prev_index: 0,
            tree: 0,
            sequence: 0xffff_ffff,
            value_in: 100_000,
            branch: 0,
            index: 0,
            prev_script: script0.clone(),
        }],
        outputs: vec![OutputMeta {
            value: 90_000,
            version: 0,
            pk_script: foreign_script(0xee),
            is_change: false,
        }],
    };

    let signed = sign_request(&secp, &m, &req).unwrap();
    let tx = MsgTx::parse_full(&signed).unwrap();

    // Extract sig + pubkey from the produced sigScript and verify it against the
    // sighash we recompute — proves sign and sighash agree end to end.
    let ss = &tx.tx_in[0].signature_script;
    let l1 = ss[0] as usize;
    let hashtype = ss[l1]; // last byte of the first push
    let der = &ss[1..l1];
    let l2 = ss[1 + l1] as usize;
    let pubkey = &ss[2 + l1..2 + l1 + l2];
    assert_eq!(hashtype, 0x01, "SigHashAll");
    assert_eq!(pubkey, &pk0[..], "signs with the re-derived key");

    let sighash = signature_hash_all(&tx, 0, &script0).unwrap();
    let mut sig = Signature::from_der(der).unwrap();
    let pk = PublicKey::from_slice(pubkey).unwrap();
    secp.verify_ecdsa(&Message::from_digest(sighash), &sig, &pk)
        .expect("self-produced signature verifies");

    // Already low-S, so normalizing is a no-op (consensus requires canonical S).
    let before = sig;
    sig.normalize_s();
    assert_eq!(before, sig, "signature is already low-S");
}

#[test]
fn sign_request_refuses_prev_script_mismatch() {
    let secp = Secp256k1::new();
    let m = master();

    // prev_script claims a different address than the key at branch/index owns:
    // the anti-tamper tripwire must fire instead of signing.
    let req = SignRequest {
        format_version: FORMAT_VERSION,
        tx_version: 1,
        account: 0,
        lock_time: 0,
        expiry: 0,
        inputs: vec![InputMeta {
            prev_hash: [9u8; 32],
            prev_index: 0,
            tree: 0,
            sequence: 0xffff_ffff,
            value_in: 100_000,
            branch: 0,
            index: 0,
            prev_script: foreign_script(0x11), // not the script for m/44'/42'/0'/0/0
        }],
        outputs: vec![OutputMeta {
            value: 90_000,
            version: 0,
            pk_script: foreign_script(0xee),
            is_change: false,
        }],
    };

    assert_eq!(sign_request(&secp, &m, &req), Err(Error::ScriptMismatch));
}
