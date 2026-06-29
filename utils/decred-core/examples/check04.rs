// SPDX-License-Identifier: Apache-2.0
//
use decred_core::airgap::decode_sign_request;
use decred_core::hd::ExtPrivKey;
use secp256k1::Secp256k1;
fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| "fuzz/04_mixed_change.dcrtx".to_string());
    let bytes = std::fs::read(&path).unwrap();
    let req = decode_sign_request(&bytes).unwrap();
    let entropy: Vec<u8> = (0..32).step_by(2).map(|i| u8::from_str_radix(&"7b7599979387940fe09d71286d6b4812"[i..i+2],16).unwrap()).collect();
    let secp = Secp256k1::new();
    let master = ExtPrivKey::from_entropy(&entropy, "").unwrap();
    println!("outputs in file:");
    for o in &req.outputs {
        println!("  value={} is_change={} script={}", o.value, o.is_change, hex::encode(&o.pk_script));
    }
    let s = req.review_owned(&secp, &master, 200).unwrap();
    println!("--- review_owned result ---");
    println!("recipients (shown): {}", s.recipients.len());
    for (a,amt) in &s.recipients { println!("  SENDING TO {} ({})", a, amt); }
    println!("change_total (real, ours): {}", s.change_total);
    println!("FLAGGED LIES: {}", s.flagged_mismatches.len());
    for (a,amt) in &s.flagged_mismatches { println!("  LIE: {} ({})", a, amt); }
}
