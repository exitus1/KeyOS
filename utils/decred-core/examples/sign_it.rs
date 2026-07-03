// SPDX-License-Identifier: Apache-2.0
//! Sign a hardcoded single-input request with a known test seed and print the
//! broadcast-ready hex. Companion piece to `build_unsigned.rs` for exercising
//! the airgap format outside the device.

use decred_core::address::{p2pkh_script, Address, AddressKind};
use decred_core::airgap::{sign_request, InputMeta, OutputMeta, SignRequest, FORMAT_VERSION};
use decred_core::hd::ExtPrivKey;
use decred_core::secp256k1::Secp256k1;
use decred_core::Network;

fn main() {
    let entropy_hex = "348360ae0a69b1883b0dfc060136108dfcabe9f4bf8af3e866b742fb53f1caa5";
    let entropy = hexb(entropy_hex);

    let secp = Secp256k1::new();
    let master = ExtPrivKey::from_entropy(&entropy, "", Network::Mainnet).expect("master");

    // sanity: derive the funding address, must match DsSXAf...
    let acct = master.account_key(&secp, 0).unwrap();
    let k = acct.address_key(&secp, 0, 0).unwrap();
    let addr = k.p2pkh_address(&secp);
    println!("derived addr: {}", addr);
    assert_eq!(addr, "DsSXAfxCeGfPgWEHKLmp6HQJJJDvJFDPfFL", "ADDRESS MISMATCH");
    println!("address matches funding address — correct seed confirmed");

    // input: our 100000-atom UTXO
    let txid_display = "bb07b9db73e0bdaa59efa316a1c7d0cf0d9cd8867a4e69efd0e8a44a3e1480c0";
    let mut prev_hash = [0u8; 32];
    for (i, b) in hexb(txid_display).iter().rev().enumerate() {
        prev_hash[i] = *b;
    }

    let input = InputMeta {
        prev_hash,
        prev_index: 0,
        tree: 0,
        sequence: 0xffff_ffff,
        value_in: 100_000,
        branch: 0,
        index: 0,
        prev_script: hexb("76a91411121bff8980463387ee14fa61a31b83487627ef88ac"),
    };

    // output: 97460 atoms -> Dsj4... (fee 2540)
    let dest = "Dsj4BQDcu3xNCTNMwvBbCigQcWiRFaNqaKK";
    let dest_addr = Address::decode(dest).unwrap();
    assert_eq!(dest_addr.kind, AddressKind::P2pkh);
    let output = OutputMeta {
        value: 97_460,
        version: 0,
        pk_script: p2pkh_script(&dest_addr.hash).to_vec(),
        is_change: false,
    };

    let req = SignRequest {
        format_version: FORMAT_VERSION,
        tx_version: 1,
        account: 0,
        lock_time: 0,
        expiry: 0,
        inputs: vec![input],
        outputs: vec![output],
        account_fp: None,
    };

    let signed = sign_request(&secp, &master, &req).expect("SIGN FAILED");
    let signed_hex: String = signed.iter().map(|b| format!("{:02x}", b)).collect();
    println!("\n=== SIGNED TX HEX (broadcast this) ===");
    println!("{}", signed_hex);
    std::fs::write("signed_tx.hex", &signed_hex).ok();
    println!("\n(saved to ./signed_tx.hex)");
}

fn hexb(s: &str) -> Vec<u8> {
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap()).collect()
}
