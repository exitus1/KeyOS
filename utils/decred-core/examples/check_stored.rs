fn main() {
    let hexstr = "e2dd56fb34312964c142143735de8810";
    let entropy: Vec<u8> = (0..hexstr.len()).step_by(2)
        .map(|i| u8::from_str_radix(&hexstr[i..i+2], 16).unwrap()).collect();
    use decred_core::hd::{ExtPrivKey, p2pkh_address};
    use secp256k1::Secp256k1;
    let m = bip39::Mnemonic::from_entropy(&entropy).unwrap();
    println!("words: {}", m.words().collect::<Vec<_>>().join(" "));
    let secp = Secp256k1::new();
    let master = ExtPrivKey::from_entropy(&entropy, "").unwrap();
    let acct = master.account_key(&secp, 0).unwrap();
    for i in 0..2 {
        let k = acct.address_key(&secp, 0, i).unwrap();
        println!("index {}: {}", i, p2pkh_address(&secp, &k));
    }
}
