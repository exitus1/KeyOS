use decred_core::hd::{ExtPrivKey, p2pkh_address};
use secp256k1::Secp256k1;

fn main() {
    // 32 bytes of BIP39 entropy (24-word mnemonic). Matches the device seam:
    // app's keys.rs gets BIP39 entropy from the secure element -> from_entropy.
    let mut entropy = [0u8; 32];
    getrandom_fill(&mut entropy);

    // Save it so we NEVER lose it.
    let hexstr = entropy.iter().map(|b| format!("{:02x}", b)).collect::<String>();
    std::fs::write(concat!(env!("HOME"), "/decred_test_seed.json"),
        format!("{{\"entropy_hex\": \"{}\"}}\n", hexstr)).ok();
    // also print so it's in the terminal
    println!("ENTROPY (SAVE THIS): {}", hexstr);

    let secp = Secp256k1::new();
    let master = ExtPrivKey::from_entropy(&entropy, "").expect("master");

    // m/44'/42'/0'/0/0  -> first receive address
    let account = master.account_key(&secp, 0).expect("account");
    let key = account.address_key(&secp, 0, 0).expect("addr key");
    let addr = p2pkh_address(&secp, &key);

    println!("FUND THIS ADDRESS: {}", addr);
    println!("(account 0, branch 0, index 0)");
}

// getrandom is already a dep via bip39/secp; use std if available, else a simple source
fn getrandom_fill(buf: &mut [u8]) {
    // read from /dev/urandom
    use std::io::Read;
    let mut f = std::fs::File::open("/dev/urandom").expect("urandom");
    f.read_exact(buf).expect("read random");
}
