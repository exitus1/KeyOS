// SPDX-License-Identifier: GPL-3.0-or-later
//
// Companion-reported balances. The device is airgapped and cannot see the
// chain, so an online companion (DCR Pulse) reports per-account balances plus
// the DCR/USD exchange rate over one of three transports - all carrying the
// SAME CBOR BalanceUpdate package (decred-core::balance), all landing in the
// same display:
//
//   * SD card:   a `balance.dcr` file on the card (read at launch)
//   * QR:        a `UR:DCR-BALANCE/...` single-part QR scanned by camera
//   * Bluetooth: QuantumLink push (hardware)
//
// Accounts are identified ONLY by their account fingerprint (the dpub's
// hash160 shorty, the same value SignRequest::account_fp carries) - never by
// name or index, which are local labels on both sides. Matching uses the
// fingerprints cached in the account store (backfilled whenever a flow already
// holds the master key: receive, dpub export, sign ingest), so rendering a
// balance NEVER touches the secure element. Entries matching no cached
// fingerprint are counted and surfaced as unknown-dpub entries, never guessed.
//
// Balance is passive - no funds ever move based on it - so a stale or wrong
// figure is at worst cosmetic, never a risk. Signing re-derives everything.
//
// The UR envelope lives here in the app (not in decred-core) so the crypto
// core stays pure - it owns curve math and CBOR, not QR transport. This
// mirrors the sign-request path, where UR unwrapping also happens app-side.

use slint_keyos_platform::slint::ComponentHandle;
use slint_keyos_platform::StoredValue;

use decred_core::balance::decode_balance_update;

use crate::state::AppState;
use crate::Balance;

pub fn init(state: StoredValue<AppState>) {
    // Try the SD card at launch; QR arrives later via ingest_qr().
    match std::fs::read(
        std::path::Path::new(crate::sign_tx::card_dir_pub()).join("balance.dcr"),
    ) {
        Ok(bytes) => {
            if let Err(e) = apply_bytes(state, &bytes, "SD card") {
                log::info!("balance file present but unusable: {e}");
            }
        }
        Err(e) => log::info!("no companion balance file: {e}"),
    }
}

/// Entry point for a scanned `UR:DCR-BALANCE/...` payload. On hardware the
/// camera hands us the UR string; in the sim it's reachable from the debug
/// paste path. Decodes UR -> bytewords -> CBOR bytes -> apply.
pub fn ingest_qr(state: StoredValue<AppState>, ur_string: &str) -> anyhow::Result<()> {
    let bytes = decode_balance_ur(ur_string)?;
    apply_bytes(state, &bytes, "QR")
}

/// Wrap BalanceUpdate CBOR bytes into a `UR:DCR-BALANCE/...` string (single part).
pub fn encode_balance_ur(bytes: &[u8]) -> String {
    let enc = foundation_ur::bytewords::encode(bytes, foundation_ur::bytewords::Style::Minimal);
    format!("UR:DCR-BALANCE/{}", enc.to_uppercase())
}

/// Decode a `UR:DCR-BALANCE/...` string back to the raw CBOR bytes.
fn decode_balance_ur(ur_string: &str) -> anyhow::Result<Vec<u8>> {
    let lower = ur_string.trim().to_lowercase();
    let ur = foundation_ur::UR::parse(&lower)
        .map_err(|e| anyhow::anyhow!("bad UR: {e:?}"))?;
    if ur.as_type() != "dcr-balance" {
        return Err(anyhow::anyhow!("not a DCR-BALANCE UR (got {})", ur.as_type()));
    }
    let msg = match ur {
        foundation_ur::UR::SinglePart { message, .. } => message,
        _ => return Err(anyhow::anyhow!("expected single-part UR")),
    };
    foundation_ur::bytewords::decode(msg, foundation_ur::bytewords::Style::Minimal)
        .map_err(|e| anyhow::anyhow!("bytewords: {e:?}"))
}

/// Decode the package, remember it in AppState, and render. `via` names the
/// transport for the card's source note.
fn apply_bytes(state: StoredValue<AppState>, bytes: &[u8], via: &str) -> anyhow::Result<()> {
    let upd = decode_balance_update(bytes).map_err(|e| anyhow::anyhow!("bad balance package: {e}"))?;
    log::info!(
        "companion balance: {} account entr{} via {via}",
        upd.accounts.len(),
        if upd.accounts.len() == 1 { "y" } else { "ies" },
    );
    state.borrow_mut().set_companion_balance(upd, via);
    render(state);
    Ok(())
}

/// Recompute the home-card figures from the stored update and the account
/// store's cached fingerprints. Called after ingest, after an account switch,
/// and after a fingerprint backfill - never derives keys itself.
pub fn render(state: StoredValue<AppState>) {
    struct Card {
        dcr: String,
        fiat: String,
        note: String,
        extra: String,
    }
    let card = {
        let s = state.borrow();
        let Some((upd, via)) = s.companion_balance() else { return };

        let active_idx = s.accounts.active;
        let active_fp = s
            .accounts
            .accounts
            .iter()
            .find(|a| a.index == active_idx)
            .and_then(|a| a.fp);
        let known: Vec<[u8; 4]> = s.accounts.accounts.iter().filter_map(|a| a.fp).collect();
        let active_entry = active_fp.and_then(|fp| upd.accounts.iter().find(|e| e.fp == fp));
        let unknown = upd.accounts.iter().filter(|e| !known.contains(&e.fp)).count();
        let rate = upd.rate_micro_usd;

        let dcr = match active_entry {
            Some(e) => crate::sign_tx::fmt_dcr(e.balance_atoms as i64),
            None => "—".to_string(),
        };
        let fiat = match active_entry {
            Some(e) if rate > 0 => format!("${}", thousands(atoms_usd(e.balance_atoms, rate))),
            _ => String::new(),
        };
        let note = format!("Updated via {via} · {}", fmt_unix_utc(upd.asof));

        // Secondary line: the summed wallet total (whenever the single figure
        // above doesn't already tell the whole story), why the active account
        // has no figure, and entries we could not attribute to any account.
        let mut extra: Vec<String> = Vec::new();
        if upd.accounts.len() > 1 || active_entry.is_none() {
            let mut total = format!("Wallet total {}", crate::sign_tx::fmt_dcr(upd.total_atoms() as i64));
            if rate > 0 {
                total.push_str(&format!(" · ${}", thousands(atoms_usd(upd.total_atoms(), rate))));
            }
            extra.push(total);
        }
        if active_fp.is_none() {
            extra.push("Account not matched yet (open Receive once)".into());
        } else if !known.is_empty() && unknown == 1 {
            extra.push("1 entry for an unknown dpub ignored".into());
        } else if !known.is_empty() && unknown > 1 {
            extra.push(format!("{unknown} entries for unknown dpubs ignored"));
        }

        Card { dcr, fiat, note, extra: extra.join(" · ") }
    };

    let ui = state.borrow().ui();
    let b = ui.global::<Balance>();
    b.set_dcr_amount(card.dcr.into());
    b.set_fiat_amount(card.fiat.into());
    b.set_source_note(card.note.into());
    b.set_as_of(card.extra.into());
    b.set_is_mock(false);
}

/// USD value of `atoms` at `rate_micro_usd` (USD per DCR x 1e6). Display-only,
/// so f64 precision is fine.
fn atoms_usd(atoms: u64, rate_micro_usd: u64) -> f64 {
    atoms as f64 / 1e8 * (rate_micro_usd as f64 / 1e6)
}

/// "YYYY-MM-DD HH:MM" (UTC) from unix seconds. An absolute stamp needs no
/// device clock, unlike "Nh ago". Civil-from-days per Hinnant's algorithm.
fn fmt_unix_utc(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    let (hh, mm) = (rem / 3600, (rem % 3600) / 60);
    let z = days + 719_468;
    let shifted = if z >= 0 { z } else { z - 146_096 };
    let era = shifted / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02} {hh:02}:{mm:02}")
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

    fn unhex(s: &str) -> Vec<u8> {
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
            .collect()
    }

    /// The same golden fixture decred-core and DCR Pulse pin: asof 1751537520,
    /// rate $16.5647, fp 7f3a9c21 = 1200 DCR, fp b44e0d5a = 34.56 DCR.
    const GOLDEN_HEX: &str = "84011a686657701a00fcc1dc828284187f183a189c18211b0000001bf08eb000828418b4184e0d185a1acdfe6000";
    const GOLDEN_UR: &str = "ur:dcr-balance/lradcyisiyhgjocyaeztseuolflflrcslbcsftcsnscsclcwaeaeaecwwtmnpfaelflrcsqzcsglbtcshtcysnzehnaeluwlregl";

    #[test]
    fn roundtrips() {
        let bytes = unhex(GOLDEN_HEX);
        let ur = encode_balance_ur(&bytes);
        assert!(ur.starts_with("UR:DCR-BALANCE/"));
        assert_eq!(decode_balance_ur(&ur).unwrap(), bytes);
    }

    #[test]
    fn matches_companion_encoder() {
        // Pulse emits lowercase; case must not matter end to end.
        assert_eq!(encode_balance_ur(&unhex(GOLDEN_HEX)).to_lowercase(), GOLDEN_UR);
        assert_eq!(decode_balance_ur(GOLDEN_UR).unwrap(), unhex(GOLDEN_HEX));
    }

    #[test]
    fn rejects_wrong_type() {
        assert!(decode_balance_ur("UR:DCR-SIGN-REQUEST/LTADADAEAEAE").is_err());
    }

    #[test]
    fn unix_formatting() {
        assert_eq!(fmt_unix_utc(1_751_537_520), "2025-07-03 10:12");
        assert_eq!(fmt_unix_utc(0), "1970-01-01 00:00");
        assert_eq!(fmt_unix_utc(4_102_444_800), "2100-01-01 00:00");
    }

    /// Prints the reference package + UR to hand to a companion implementation.
    /// Run: cargo test -p gui-app-decred print_reference_ur -- --nocapture
    #[test]
    fn print_reference_ur() {
        println!(
            "\n=== DCR-BALANCE v2 reference ===\nCBOR hex:\n{GOLDEN_HEX}\n\nUR (companion renders this as the QR):\n{}\n",
            encode_balance_ur(&unhex(GOLDEN_HEX)),
        );
    }
}
