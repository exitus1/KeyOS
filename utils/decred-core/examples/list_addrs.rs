// SPDX-License-Identifier: Apache-2.0
//
use decred_core::hd::{ExtPrivKey, p2pkh_address};
use secp256k1::Secp256k1;
fn main() {
    let hexstr = "348360ae0a69b1883b0dfc060136108dfcabe9f4bf8af3e866b742fb53f1caa5";
    let entropy: Vec<u8> = (0..hexstr.len()).step_by(2)
        .map(|i| u8::from_str_radix(&hexstr[i..i+2], 16).unwrap()).collect();
    let secp = Secp256k1::new();
    let master = ExtPrivKey::from_entropy(&entropy, "").unwrap();
    let acct = master.account_key(&secp, 0).unwrap();
    println!("=== your 348360ae seed ===");
    for i in 0..6 {
        let k = acct.address_key(&secp, 0, i).unwrap();
        println!("index {}: {}", i, p2pkh_address(&secp, &k));
    }
    let zmaster = ExtPrivKey::from_entropy(&[0u8;32], "").unwrap();
    let zacct = zmaster.account_key(&secp, 0).unwrap();
    println!("=== all-zero seed ===");
    for i in 0..3 {
        let zk = zacct.address_key(&secp, 0, i).unwrap();
        println!("index {}: {}", i, p2pkh_address(&secp, &zk));
    }
}
