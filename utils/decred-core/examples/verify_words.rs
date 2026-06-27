fn main() {
    let words = "cruel brand clog below only series umbrella sauce alcohol answer loud brief skirt visual spray vanish view drip punch magnet release wedding clever route";
    match bip39::Mnemonic::parse(words) {
        Ok(m) => {
            println!("VALID BIP39 mnemonic, {} words", m.words().count());
            let ent = m.to_entropy();
            println!("entropy: {}", ent.iter().map(|b| format!("{:02x}",b)).collect::<String>());
        }
        Err(e) => println!("INVALID: {:?}", e),
    }
}
