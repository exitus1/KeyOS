use decred_core::airgap::{encode_sign_request, InputMeta, OutputMeta, SignRequest, FORMAT_VERSION};
fn main() {
    let req = SignRequest {
        format_version: FORMAT_VERSION,
        tx_version: 1,
        account: 0,
        lock_time: 0,
        expiry: 0,
        inputs: vec![InputMeta {
            prev_hash: [0x11u8; 32],
            prev_index: 0,
            tree: 0,
            sequence: 0xffff_ffff,
            value_in: 94_000,
            branch: 0,
            index: 1,
            prev_script: hex::decode("76a914c671a001f211d63e0b0b4791e6d343c85b1e72f088ac").unwrap(),
        }],
        outputs: vec![OutputMeta {
            value: 91_830,
            version: 0,
            pk_script: hex::decode("76a914c671a001f211d63e0b0b4791e6d343c85b1e72f088ac").unwrap(),
            is_change: false,
        }],
    };
    let bytes = encode_sign_request(&req).unwrap();
    println!("=== CBOR SignRequest, {} bytes ===", bytes.len());
    println!("{}", hex::encode(&bytes));
    println!();
    println!("This is ARRAY-encoded (minicbor default, no #[cbor(map)]):");
    println!("  SignRequest = 7-element CBOR array");
    println!("  inputs[i]   = 8-element CBOR array");
    println!("  outputs[i]  = 4-element CBOR array");
}
