// SPDX-License-Identifier: GPL-3.0-or-later
//
// Companion-reported balance. The device is airgapped and cannot see the
// chain, so an online companion (DCR Pulse) reports the balance over one of
// three transports — all carrying the SAME key=value text, all landing in the
// same display:
//
//   * SD card:   a `balance.dcr` file on the card (read at launch)
//   * QR:        a `UR:DCR-BALANCE/...` animated/single QR scanned by camera
//   * Bluetooth: QuantumLink push (hardware; sets via=Bluetooth)
//
// Balance is passive — no funds ever move based on it — so a stale or wrong
// figure is at worst cosmetic, never a risk. Signing re-derives everything.
// The payload is still untrusted input rendered on the trust-anchor screen,
// though, so every field is parsed/sanitized before display: the amounts must
// be finite in-range numbers (re-formatted canonically, never echoed) and the
// free-text fields are length-capped and stripped to printable characters.
//
// Payload format (identical across all transports):
//   dcr=1234.56
//   usd=20450.32       (optional)
//   via=SD card        (optional transport word; sender names its own channel)
//   asof=2h ago        (optional, free text)
// Rendered on the home card as: "Updated via SD card · 2h ago".

use slint_keyos_platform::slint::ComponentHandle;
use slint_keyos_platform::StoredValue;

use crate::state::AppState;
use crate::Balance;

pub fn init(state: StoredValue<AppState>) {
    // Try the SD card at launch; QR arrives later via ingest_qr().
    match crate::sign_tx::read_card_file("balance.dcr") {
        Ok(bytes) => match String::from_utf8(bytes) {
            Ok(text) => {
                if let Err(e) = apply_text(state, &text, "SD card") {
                    log::info!("balance file present but unusable: {e}");
                }
            }
            Err(e) => log::info!("balance file is not UTF-8: {e}"),
        },
        Err(e) => log::info!("no companion balance file: {e}"),
    }
}

/// Entry point for a scanned `UR:DCR-BALANCE/...` payload. On hardware the
/// camera hands us the UR string; in the sim it's reachable from the debug
/// paste path. Decodes UR -> bytewords -> UTF-8 text -> apply.
///
/// The UR envelope lives here in the app (not in decred-core) so the crypto
/// core stays pure — it owns curve math and CBOR, not QR transport. This
/// mirrors the sign-request path, where UR unwrapping also happens app-side.
#[allow(dead_code)] // wired up when the camera/UR pump lands (see begin_scan)
pub fn ingest_qr(state: StoredValue<AppState>, ur_string: &str) -> anyhow::Result<()> {
    let text = decode_balance_ur(ur_string)?;
    apply_text(state, &text, "QR")
}

/// Wrap balance key=value text into a `UR:DCR-BALANCE/...` string (single part).
/// Exercised by the round-trip tests; Pulse carries the mirror encoder.
#[allow(dead_code)]
pub fn encode_balance_ur(text: &str) -> String {
    let enc = foundation_ur::bytewords::encode(text.as_bytes(), foundation_ur::bytewords::Style::Minimal);
    format!("UR:DCR-BALANCE/{}", enc.to_uppercase())
}

/// Decode a `UR:DCR-BALANCE/...` string back to its key=value text.
fn decode_balance_ur(ur_string: &str) -> anyhow::Result<String> {
    let lower = ur_string.trim().to_lowercase();
    let ur = foundation_ur::UR::parse(&lower).map_err(|e| anyhow::anyhow!("bad UR: {e:?}"))?;
    if ur.as_type() != "dcr-balance" {
        return Err(anyhow::anyhow!("not a DCR-BALANCE UR (got {})", ur.as_type()));
    }
    let msg = match ur {
        foundation_ur::UR::SinglePart { message, .. } => message,
        _ => return Err(anyhow::anyhow!("expected single-part UR")),
    };
    let bytes = foundation_ur::bytewords::decode(msg, foundation_ur::bytewords::Style::Minimal)
        .map_err(|e| anyhow::anyhow!("bytewords: {e:?}"))?;
    String::from_utf8(bytes).map_err(|e| anyhow::anyhow!("utf8: {e}"))
}

/// Keep a companion-supplied free-text field displayable: printable chars
/// only, hard length cap. The screen is the device's trust anchor; a hostile
/// balance file must not be able to fake UI chrome with control characters or
/// pad itself into covering other elements.
fn sanitize(field: &str, max_chars: usize) -> String {
    field.chars().filter(|c| !c.is_control()).take(max_chars).collect::<String>().trim().to_string()
}

/// Parse the shared key=value text and push it to the display. `default_via`
/// is the transport word used when the payload doesn't name its own.
pub(crate) fn apply_text(state: StoredValue<AppState>, text: &str, default_via: &str) -> anyhow::Result<()> {
    let mut dcr = String::new();
    let mut usd = String::new();
    let mut asof = String::new();
    let mut via = String::new();
    for line in text.lines().take(16) {
        if let Some((k, v)) = line.split_once('=') {
            match k.trim() {
                "dcr" => dcr = v.trim().to_string(),
                "usd" => usd = v.trim().to_string(),
                "asof" => asof = sanitize(v, 24),
                "via" => via = sanitize(v, 16),
                _ => {}
            }
        }
    }

    // The DCR amount must be a real number in supply range — and it is
    // re-formatted from the parsed value, never echoed as sent.
    let dcr_val: f64 = dcr.parse().map_err(|_| anyhow::anyhow!("dcr field is not a number"))?;
    if !dcr_val.is_finite() || !(0.0..=21_000_000.0).contains(&dcr_val) {
        return Err(anyhow::anyhow!("dcr amount out of range"));
    }
    let dcr_str = format!("{dcr_val} DCR");

    // Optional fiat figure: same treatment, silently dropped if unusable.
    let fiat_str = usd
        .parse::<f64>()
        .ok()
        .filter(|n| n.is_finite() && (0.0..1e12).contains(n))
        .map(|n| format!("${}", thousands(n)))
        .unwrap_or_default();

    let transport = if via.is_empty() { default_via.to_string() } else { via };
    let note = if asof.is_empty() {
        format!("Updated via {transport}")
    } else {
        format!("Updated via {transport} · {asof}")
    };

    let ui = state.borrow().ui();
    let b = ui.global::<Balance>();
    b.set_dcr_amount(dcr_str.into());
    b.set_fiat_amount(fiat_str.into());
    b.set_source_note(note.into());
    b.set_as_of("".into());
    b.set_is_mock(false);
    log::info!("companion balance: {dcr_val} DCR via {transport}");
    Ok(())
}

fn thousands(n: f64) -> String {
    let s = format!("{n:.2}");
    let (int, frac) = s.split_once('.').unwrap_or((&s, "00"));
    let neg = int.starts_with('-');
    let digits = int.trim_start_matches('-');
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    format!("{}{}.{}", if neg { "-" } else { "" }, out, frac)
}

#[cfg(test)]
mod balance_ur_tests {
    use super::*;

    #[test]
    fn roundtrips() {
        let text = "dcr=1234.56\nusd=20450.32\nvia=QR\nasof=just now";
        let ur = encode_balance_ur(text);
        assert!(ur.starts_with("UR:DCR-BALANCE/"));
        assert_eq!(decode_balance_ur(&ur).unwrap(), text);
    }

    #[test]
    fn rejects_wrong_type() {
        assert!(decode_balance_ur("UR:DCR-SIGN-REQUEST/LTADADAEAEAE").is_err());
    }

    #[test]
    fn minimal_payload() {
        let ur = encode_balance_ur("dcr=5.00");
        assert_eq!(decode_balance_ur(&ur).unwrap(), "dcr=5.00");
    }

    #[test]
    fn sanitize_strips_control_and_caps_length() {
        assert_eq!(sanitize("2h\x1b[31m ago", 24), "2h[31m ago");
        assert_eq!(sanitize(&"x".repeat(100), 16).len(), 16);
        assert_eq!(sanitize("  padded  ", 24), "padded");
    }

    /// Prints a reference UR string to hand to the companion (DCR Pulse) so
    /// their encoder can be checked against ours. Run:
    ///   cargo test -p gui-app-decred print_reference_ur -- --nocapture
    #[test]
    fn print_reference_ur() {
        let payload = "dcr=1234.56\nusd=20450.32\nvia=QR\nasof=just now";
        let ur = encode_balance_ur(payload);
        println!("\n=== DCR-BALANCE reference ===\npayload:\n{payload}\n\nUR (Pulse renders this as the QR):\n{ur}\n");
    }
}
