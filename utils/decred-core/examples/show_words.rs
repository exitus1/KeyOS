fn main() {
    let hexstr = "348360ae0a69b1883b0dfc060136108dfcabe9f4bf8af3e866b742fb53f1caa5";
    let entropy: Vec<u8> = (0..hexstr.len()).step_by(2)
        .map(|i| u8::from_str_radix(&hexstr[i..i+2], 16).unwrap()).collect();
    let m = bip39::Mnemonic::from_entropy(&entropy).expect("mnemonic");
    let words: Vec<&str> = m.words().collect();
    println!("word count: {}", words.len());
    for (i, w) in words.iter().enumerate() {
        println!("{:2}. {}", i+1, w);
    }
}
