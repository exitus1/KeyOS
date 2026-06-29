// SPDX-License-Identifier: Apache-2.0
//
fn main() {
    let words = "city quit shell buddy sponsor giant blast device oak bonus viable consider";
    let m = bip39::Mnemonic::parse(words).expect("parse");
    let entropy = m.to_entropy();
    println!("entropy: {}", entropy.iter().map(|b| format!("{:02x}",b)).collect::<String>());
    use decred_core::hd::{ExtPrivKey, p2pkh_address};
    use secp256k1::Secp256k1;
    let secp = Secp256k1::new();
    let master = ExtPrivKey::from_entropy(&entropy, "").unwrap();
    let acct = master.account_key(&secp, 0).unwrap();
    for i in 0..3 {
        let k = acct.address_key(&secp, 0, i).unwrap();
        println!("index {}: {}", i, p2pkh_address(&secp, &k));
    }
}
