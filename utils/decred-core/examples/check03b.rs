// SPDX-License-Identifier: Apache-2.0
//
use decred_core::airgap::decode_sign_request;
use decred_core::hd::ExtPrivKey;
use secp256k1::Secp256k1;
fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| "fuzz/03_external_as_change.dcrtx".to_string());
    let bytes = std::fs::read(&path).unwrap();
    let req = decode_sign_request(&bytes).unwrap();
    let entropy: Vec<u8> = (0..32).step_by(2).map(|i| u8::from_str_radix(&"7b7599979387940fe09d71286d6b4812"[i..i+2],16).unwrap()).collect();
    let secp = Secp256k1::new();
    let master = ExtPrivKey::from_entropy(&entropy, "").unwrap();
    let s = req.review_owned(&secp, &master, 200).unwrap();
    println!("recipients: {}", s.recipients.len());
    println!("FLAGGED MISMATCHES (companion lies): {}", s.flagged_mismatches.len());
    for (a,amt) in &s.flagged_mismatches { println!("  LIE: {} ({}) claimed change but NOT ours", a, amt); }
}
