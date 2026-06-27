use decred_core::airgap::{encode_sign_request, InputMeta, OutputMeta, SignRequest, FORMAT_VERSION};
use decred_core::address::{decode_p2pkh, p2pkh_script};

fn main() {
    let txid_display = "45a6f7e1a8b9af04ed7cbb5480b9af75928595ac4266db01efe89fe2024c22fb";
    let mut prev_hash = [0u8; 32];
    let raw = hex_to_bytes(txid_display);
    for (i, b) in raw.iter().rev().enumerate() {
        prev_hash[i] = *b;
    }
    let prev_script = hex_to_bytes("76a914b2c6669bfa9c6766413bcd26c04bfa5f6254d8e088ac");
    let input = InputMeta {
        prev_hash,
        prev_index: 0,
        tree: 0,
        sequence: 0xffff_ffff,
        value_in: 100_000,
        branch: 0,
        index: 0,
        prev_script,
    };
    let dest = "Dsj4BQDcu3xNCTNMwvBbCigQcWiRFaNqaKK";
    let h160 = decode_p2pkh(dest).expect("decode dest address");
    let out_script = p2pkh_script(&h160).to_vec();
    let output = OutputMeta {
        value: 97_460,
        version: 0,
        pk_script: out_script,
        is_change: false,
    };
    let req = SignRequest {
        format_version: FORMAT_VERSION,
        tx_version: 1,
        account: 0,
        lock_time: 0,
        expiry: 0,
        inputs: vec![input],
        outputs: vec![output],
    };
    let bytes = encode_sign_request(&req).expect("encode");
    std::fs::write("unsigned.dcrtx", &bytes).expect("write file");
    println!("wrote unsigned.dcrtx ({} bytes)", bytes.len());
    println!("  input:  100000 atoms  (txid {}:0)", txid_display);
    println!("  output:  97460 atoms -> {}", dest);
    println!("  fee:      2540 atoms");
}

fn hex_to_bytes(s: &str) -> Vec<u8> {
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i+2], 16).unwrap()).collect()
}
