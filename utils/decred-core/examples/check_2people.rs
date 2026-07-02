use decred_core::airgap::decode_sign_request;
use decred_core::hd::ExtPrivKey;
use secp256k1::Secp256k1;
fn main() {
    let bytes = std::fs::read("/home/mike/fuzz/unsigned-tx-2-people.dcrtx").unwrap();
    println!("{} bytes, first=0x{:02x}", bytes.len(), bytes[0]);
    let req = decode_sign_request(&bytes).unwrap();
    let entropy: Vec<u8> = (0..32).step_by(2).map(|i| u8::from_str_radix(&"7b7599979387940fe09d71286d6b4812"[i..i+2],16).unwrap()).collect();
    let secp = Secp256k1::new();
    let master = ExtPrivKey::from_entropy(&entropy, "").unwrap();
    println!("inputs={} outputs={}", req.inputs.len(), req.outputs.len());
    for (i,o) in req.outputs.iter().enumerate() {
        println!("  out[{}] value={} ({} DCR) is_change={}", i, o.value, o.value as f64/1e8, o.is_change);
    }
    let s = req.review_owned(&secp, &master, 200).unwrap();
    println!("--- review_owned ---");
    println!("recipients shown: {}", s.recipients.len());
    for (a,amt) in &s.recipients { println!("  SENDING TO {} ({} DCR)", a, *amt as f64/1e8); }
    println!("change_total: {} ({} DCR)", s.change_total, s.change_total as f64/1e8);
    println!("FLAGGED LIES: {}", s.flagged_mismatches.len());
}
