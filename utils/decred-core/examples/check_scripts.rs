fn main() {
    let entropy: Vec<u8> = (0..32).step_by(2)
        .map(|i| u8::from_str_radix(&"7b7599979387940fe09d71286d6b4812"[i..i+2], 16).unwrap()).collect();
    use decred_core::hd::{ExtPrivKey, BRANCH_EXTERNAL};
    use decred_core::hashing::hash160;
    use decred_core::p2pkh_script;
    use secp256k1::Secp256k1;
    let secp = Secp256k1::new();
    let master = ExtPrivKey::from_entropy(&entropy, "").unwrap();
    let acct = master.account_key(&secp, 0).unwrap();
    let k0 = acct.address_key(&secp, BRANCH_EXTERNAL, 0).unwrap();
    let k1 = acct.address_key(&secp, BRANCH_EXTERNAL, 1).unwrap();
    println!("index0 script: {}", hex::encode(p2pkh_script(&hash160(&k0.compressed_pubkey(&secp)))));
    println!("  (chain says: 76a9143afaebcdfd8cda72e687f0e4f72f8f0a6b14bb9f88ac)");
    println!("index1 script: {}", hex::encode(p2pkh_script(&hash160(&k1.compressed_pubkey(&secp)))));
}
