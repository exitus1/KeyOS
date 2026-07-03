// SPDX-License-Identifier: Apache-2.0
//! Adapter smoke test: proves the `decred_core::…` paths KeyOS apps compile
//! against still reach the real dcr-rs implementation, end to end — decode a
//! deployed-format package (with and without the account fingerprint),
//! validate it, run the trustless review from an account xpub, sign it, and
//! verify the produced signature against a recomputed sighash. The exhaustive
//! consensus vectors (dcrd KATs, BIP32 chains, a real mainnet tx) live
//! upstream in dcr-rs and run in its CI; duplicating them here would only let
//! the copies drift.

use decred_core::address::p2pkh_script;
use decred_core::airgap::{decode_sign_request, sign_request};
use decred_core::hashing::hash160;
use decred_core::hd::{ExtPrivKey, BRANCH_EXTERNAL};
use decred_core::secp256k1::{ecdsa::Signature, Message, PublicKey, Secp256k1};
use decred_core::sighash::signature_hash_all;
use decred_core::tx::MsgTx;
use decred_core::Network;

/// 7-element (pre-fingerprint) package from the original Pulse interop.
const LEGACY7: &str = "87010100000081889820185418d918e518bf18ca183b18461889186418de1897071858185718b003182718fd18c0189318df184418fd18c103182d1118ef16184c1856183700001affffffff1a05f5e10000019819187618a9141823187c18f418a818651850186d18a5183018761118cf1856186318cb18731843188518c41892188818ac81841a05a995c0009819187618a91418c6187118a00118f21118d6183e0b0b1847189118e618d3184318c8185b181e187218f0188818acf4";
/// The same package with an appended account fingerprint [1,2,3,4].
const WITHFP: &str = "88010100000081889820185418d918e518bf18ca183b18461889186418de1897071858185718b003182718fd18c0189318df184418fd18c103182d1118ef16184c1856183700001affffffff1a05f5e10000019819187618a9141823187c18f418a818651850186d18a5183018761118cf1856186318cb18731843188518c41892188818ac81841a05a995c0009819187618a91418c6187118a00118f21118d6183e0b0b1847189118e618d3184318c8185b181e187218f0188818acf48401020304";

const ENTROPY_HEX: &str = "348360ae0a69b1883b0dfc060136108dfcabe9f4bf8af3e866b742fb53f1caa5";

#[test]
fn deployed_wire_format_decodes_through_adapter() {
    let legacy = decode_sign_request(&hex::decode(LEGACY7).unwrap()).unwrap();
    assert_eq!(legacy.account_fp, None);
    legacy.validate().unwrap();

    let with_fp = decode_sign_request(&hex::decode(WITHFP).unwrap()).unwrap();
    assert_eq!(with_fp.account_fp, Some([1, 2, 3, 4]));
    with_fp.validate().unwrap();
}

#[test]
fn review_sign_and_verify_end_to_end() {
    let secp = Secp256k1::new();
    let entropy = hex::decode(ENTROPY_HEX).unwrap();
    let master = ExtPrivKey::from_entropy(&entropy, "", Network::Mainnet).unwrap();
    let account = master.account_key(&secp, 0).unwrap();

    // Build a request spending this wallet's own external/0 key.
    let key0 = account.address_key(&secp, BRANCH_EXTERNAL, 0).unwrap();
    let pk0 = key0.compressed_pubkey(&secp);
    let script0 = p2pkh_script(&hash160(&pk0)).to_vec();

    let mut req = decode_sign_request(&hex::decode(LEGACY7).unwrap()).unwrap();
    req.inputs[0].prev_script = script0.clone();
    req.inputs[0].branch = BRANCH_EXTERNAL;
    req.inputs[0].index = 0;

    // Trustless review from the neutered account key: no private material.
    let xpub = account.neuter(&secp);
    req.check_owned_inputs(&secp, &xpub).unwrap();
    let summary = req.review_owned(&secp, &xpub).unwrap();
    assert_eq!(summary.fee, summary.input_total - summary.output_total);

    // Sign, reparse, and verify the first signature against our own sighash.
    let signed = sign_request(&secp, &master, &req).unwrap();
    let tx = MsgTx::parse_full(&signed).unwrap();
    let ss = &tx.tx_in[0].signature_script;
    let l1 = ss[0] as usize;
    assert_eq!(ss[l1], 0x01, "SigHashAll");
    let der = &ss[1..l1];
    let pubkey = &ss[2 + l1..2 + l1 + ss[1 + l1] as usize];
    assert_eq!(pubkey, &pk0[..], "signed with the re-derived key");

    let sighash = signature_hash_all(&tx, 0, &script0).unwrap();
    let sig = Signature::from_der(der).unwrap();
    let pk = PublicKey::from_slice(pubkey).unwrap();
    secp.verify_ecdsa(&Message::from_digest(sighash), &sig, &pk).expect("adapter-path signature verifies");
}
