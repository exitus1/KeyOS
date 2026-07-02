use foundation_ur::{UR, bytewords};
fn main() {
    let frame = "UR:DCR-SIGN-REQUEST/LTADADAEAEAELFLOMKCXCSGLCSIECSLECSCPCSWDCSHHCSMHCSCKCSUOCSSTCSGYCSIACSWDCSLTCSWKCHCSPDCSYLCSCECSPMCSRLCSSSCSWSCSEHCSTACSPRCSFDCSCNCSIMCSDRCSRTCSNEADAECYZMZMZMZMCYAEMKMTLAAEAEMKCFCSKOCSPTBBCSFTCSZSCSWMCSSNCSZCCSLKCSTNCSJPCSVACSLTCSWTCSVECSYLCSDLCSMYBKCSJEBBCSRKCSNECSLOCSPSLOMKCXBDCSZSCSJYCSWLCSNECSDSCSQDCSYACSKECSPKCSJSCSDECSONCSJPCSHYCSTNCSLPCSDYCSHGCSIHCSENCSECCSWFCSVSCSDSCSTBAHCSQZCSLUCSSACSPTATAEAECYZMZMZMZMCYAEAXBTFZAEAEMKCFCSKOCSPTBBCSFTCSZSCSWMCSSNCSZCCSLKCSTNCSJPCSVACSLTCSWTCSVECSYLCSDLCSMYBKCSJEBBCSRKCSNECSLOCSPSLFLRCYAEAOZTVOAEMKCFCSKOCSPTBBCSYLCSLDCSNTCSLNCSIMCSNTCSHYCSEOCSZECSRDCSLUAMCSCXBACSGLCSWMCSPYCSMDCSUOCSJTCSLOCSPSYKLRCYAEMKMTLAAEMKCFCSKOCSPTBBCSSWCSJSCSNBADCSWZBYCSTBCSFMBDBDCSFLCSMECSVACSTECSFXCSSPCSHPCSCKCSJPCSWTCSLOCSPSWKKKPYNSFS";
    let lower = frame.to_lowercase();
    let ur = match UR::parse(&lower) {
        Ok(u) => u,
        Err(e) => { println!("parse error: {:?}", e); return; }
    };
    println!("ur_type: {}", ur.as_type());
    println!("is_single_part: {}", ur.is_single_part());
    let msg = match ur {
        UR::SinglePart { message, .. } => message,
        other => { println!("unexpected variant, single={}", other.is_single_part()); return; }
    };
    match bytewords::decode(msg, bytewords::Style::Minimal) {
        Ok(bytes) => {
            println!("DECODED {} bytes, first=0x{:02x}", bytes.len(), bytes[0]);
            println!("hex: {}", hex::encode(&bytes));
            match decred_core::airgap::decode_sign_request(&bytes) {
                Ok(req) => {
                    println!("\n>>> PARSED AS SignRequest! account={} inputs={} outputs={}",
                        req.account, req.inputs.len(), req.outputs.len());
                    for (i,o) in req.outputs.iter().enumerate() {
                        println!("  out[{}] value={} is_change={}", i, o.value, o.is_change);
                    }
                }
                Err(e) => println!("\n!!! not a SignRequest (yet): {:?}", e),
            }
        }
        Err(e) => println!("bytewords decode error: {:?}", e),
    }
}
