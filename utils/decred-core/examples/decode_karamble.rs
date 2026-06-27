use decred_core::airgap::decode_sign_request;
fn main() {
    let bytes = std::fs::read("/home/mike/karamble_unsigned.dcrtx").unwrap();
    let req = decode_sign_request(&bytes).expect("decode failed");
    println!("format_version: {}  tx_version: {}  account: {}", req.format_version, req.tx_version, req.account);
    let mut in_total = 0i64;
    for inp in &req.inputs {
        in_total += inp.value_in;
        let mut disp = inp.prev_hash; disp.reverse();
        println!("INPUT prevout {}:{} value_in={} path m/44(0x2a)/{}'/{}/{}", hex::encode(disp), inp.prev_index, inp.value_in, req.account, inp.branch, inp.index);
        println!("  prev_script: {}", hex::encode(&inp.prev_script));
    }
    let mut out_total = 0i64;
    for o in &req.outputs {
        out_total += o.value;
        println!("OUTPUT value={} is_change={} script={}", o.value, o.is_change, hex::encode(&o.pk_script));
    }
    println!("FEE: {} atoms", in_total - out_total);
    let s = req.review();
    println!("--- REVIEW (what the screen shows) ---");
    for (addr, amt) in &s.recipients { println!("  SENDING TO: {}  ({} atoms)", addr, amt); }
    println!("  change_total: {}  fee: {}", s.change_total, s.fee);
}
