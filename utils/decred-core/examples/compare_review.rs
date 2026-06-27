use decred_core::airgap::decode_sign_request;
use decred_core::hd::ExtPrivKey;
use secp256k1::Secp256k1;
fn main() {
    let bytes = std::fs::read("/home/mike/karamble_unsigned.dcrtx").unwrap();
    let req = decode_sign_request(&bytes).unwrap();
    let entropy: Vec<u8> = (0..32).step_by(2).map(|i| u8::from_str_radix(&"7b7599979387940fe09d71286d6b4812"[i..i+2],16).unwrap()).collect();
    let secp = Secp256k1::new();
    let master = ExtPrivKey::from_entropy(&entropy, "").unwrap();

    println!("=== OLD review() — TRUSTS companion is_change ===");
    let old = req.review();
    println!("  recipients: {}", old.recipients.len());
    for (a,amt) in &old.recipients { println!("    SENDING TO {} ({})", a, amt); }
    println!("  change_total: {}  fee: {}", old.change_total, old.fee);

    println!("\n=== NEW review_owned() — RE-DERIVES ownership ===");
    let new = req.review_owned(&secp, &master, 1000).unwrap();
    println!("  recipients: {}", new.recipients.len());
    for (a,amt) in &new.recipients { println!("    SENDING TO {} ({})", a, amt); }
    println!("  change_total: {}  fee: {}", new.change_total, new.fee);
}
