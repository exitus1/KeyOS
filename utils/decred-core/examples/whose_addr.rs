use decred_core::hd::{ExtPrivKey, BRANCH_EXTERNAL, p2pkh_address};
use decred_core::hashing::hash160;
use secp256k1::Secp256k1;
fn main() {
    let entropy: Vec<u8> = (0..32).step_by(2).map(|i| u8::from_str_radix(&"7b7599979387940fe09d71286d6b4812"[i..i+2],16).unwrap()).collect();
    let secp = Secp256k1::new();
    let master = ExtPrivKey::from_entropy(&entropy, "").unwrap();
    let acct = master.account_key(&secp, 0).unwrap();
    let target = "c671a001f211d63e0b0b4791e6d343c85b1e72f0";
    println!("Looking for hash160 {} among your addresses...", target);
    for branch in [0u32, 1u32] {
        for i in 0..6u32 {
            let k = acct.address_key(&secp, branch, i).unwrap();
            let h = hex::encode(hash160(&k.compressed_pubkey(&secp)));
            let mark = if h == target { "  <<< MATCH (this is YOUR address)" } else { "" };
            println!("  branch {} index {}: {} {}{}", branch, i, p2pkh_address(&secp,&k), h, mark);
        }
    }
}
