use slint_keyos_platform::slint::ComponentHandle;
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Signing flow. This is where the airgap package comes in (QR or SD), gets
// reviewed on-screen, gets signed with keys re-derived from the secure element,
// and goes back out (QR or SD).
//
// Decred has no PSBT. decred-core defines a compact CBOR "unsigned-tx package"
// (airgap::SignRequest, FORMAT_VERSION = 1) carrying: the unsigned tx bytes,
// and per input the prev_script + amount + derivation path. Cake Wallet is a
// watch-only wallet here: it knows the UTXOs, scripts, paths and builds that
// package. The device re-derives each input key, recomputes its P2PKH script,
// and REFUSES to sign if the recomputed script != the script the host claimed
// (decred_core::Error::ScriptMismatch). That check is the anti-tamper tripwire.
//
// Two transports, one core:
//   QR : foundation-ur animated UR frames, type "dcr-sign-request" in,
//        "dcr-signed-tx" out.
//   SD : read  unsigned.dcrtx (raw CBOR bytes) from the Airlock/USB scope,
//        write signed.dcrtx (raw serialized full tx) back.

use anyhow::{anyhow, Context, Result};
use slint_keyos_platform::StoredValue;

use crate::keys::load_master_key;
use crate::state::AppState;
// Slint-generated globals/enums (emitted into the crate root by `app!`).
use crate::{OriginView, SignState, SignTx};
use decred_core::airgap::{decode_sign_request, sign_request, ReviewSummary, SignRequest};

/// Where a given signing request arrived from. Mirrors the Bitcoin app's
/// PsbtOrigin (File | Qr | QuantumLink); we drop QuantumLink for now.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Origin {
    Qr,
    SdCard,
}

/// Install Slint callbacks for the signing screens.
pub fn init(state: StoredValue<AppState>) {
    let ui = state.borrow().ui();
    let sign = ui.global::<SignTx>();

    // User tapped "Scan QR".
    sign.on_start_qr_scan({
        move || {
            if let Err(e) = begin_scan(state) {
                log::error!("qr scan start failed: {e:?}");
            }
        }
    });

    // User tapped "Load from SD card".
    sign.on_load_from_sd({
        move || {
            if let Err(e) = load_from_sd(state) {
                log::error!("sd load failed: {e:?}");
                show_error(state, &e.to_string());
            }
        }
    });

    // User reviewed the summary and tapped "Approve & Sign".
    sign.on_approve({
        move || {
            if let Err(e) = approve_and_sign(state) {
                log::error!("signing failed: {e:?}");
                show_error(state, &e.to_string());
            }
        }
    });

    // User backed out (Reject / Cancel): clear pending and return to Idle.
    sign.on_reject({
        move || {
            state.borrow_mut().clear_pending();
            let ui = state.borrow().ui();
            ui.global::<SignTx>().set_state(SignState::Idle);
        }
    });

    // Animated-QR frames for the signed tx. Returns the UR parts once a tx has
    // been signed; empty until then. (Chunking into UR parts is the TODO in
    // emit_qr — this is the read side the DynamicQrCode pulls.)
    sign.on_signed_qr_parts({
        move || -> slint_keyos_platform::slint::ModelRc<slint_keyos_platform::slint::SharedString> {
            let parts = state.borrow().signed_qr_parts();
            slint_keyos_platform::slint::ModelRc::new(slint_keyos_platform::slint::VecModel::from(
                parts
                    .into_iter()
                    .map(slint_keyos_platform::slint::SharedString::from)
                    .collect::<Vec<_>>(),
            ))
        }
    });
}

/// Deep-link / button entry: start the animated-QR scanner. The actual frame
/// pump lives in a spawn_local loop that feeds foundation-ur until a complete
/// "dcr-sign-request" is assembled, then calls `ingest`.
pub fn begin_scan(state: StoredValue<AppState>) -> Result<()> {
    let ui = state.borrow().ui();
    ui.global::<SignTx>().set_origin(OriginView::Qr);
    ui.global::<SignTx>().set_state(SignState::Scanning);
    // NOTE: camera/UR frame loop omitted in this scaffold — see
    // gui-app-qr-scanner for the animated-UR decode pattern to copy. On a
    // completed UR payload, call `ingest(state, Origin::Qr, &bytes)`.
    Ok(())
}

/// Read the unsigned package off removable media.
fn load_from_sd(state: StoredValue<AppState>) -> Result<()> {
    // `fs` is the KeyOS filesystem API. Path/scoping mirrors the Bitcoin app's
    // file PSBT load; we read raw CBOR bytes, not a PSBT.
    use std::io::Read;
    let mut opened = fs::FileSystem::<crate::fs_permissions::FileSystemPermissions>::default()
        .open_file("unsigned.dcrtx", fs::Location::Airlock, fs::OpenFlags { read: true, write: false, create: false })
        .context("opening unsigned.dcrtx")?;
    let mut bytes = Vec::new();
    opened.read_to_end(&mut bytes).context("reading unsigned.dcrtx")?;
    ingest(state, Origin::SdCard, &bytes)
}

/// Common path for both transports: decode the package, derive a review
/// summary for the on-device confirmation screen, and stash it pending
/// approval. We decode (cheap) here but DEFER key derivation until the user
/// approves, so the secure-element prompt only fires on real intent.
pub fn ingest(state: StoredValue<AppState>, origin: Origin, bytes: &[u8]) -> Result<()> {
    let req: SignRequest = decode_sign_request(bytes).map_err(|e| anyhow!("bad package: {e}"))?;
    let summary: ReviewSummary = req.review();

    // Persist the raw bytes + origin so approve_and_sign can re-decode and sign.
    {
        let mut s = state.borrow_mut();
        s.set_pending(origin, bytes.to_vec());
    }

    render_review(state, &summary);
    Ok(())
}

/// Push the human-readable review (recipients, change, fee) into Slint.
/// Amounts are formatted to DCR strings here because Slint's `int` is 32-bit
/// and atom values (1 DCR = 1e8 atoms) overflow it; the UI shows "1.2345 DCR".
fn render_review(state: StoredValue<AppState>, summary: &ReviewSummary) {
    let ui = state.borrow().ui();
    let sign = ui.global::<SignTx>();
    let send_total: i64 = summary.recipients.iter().map(|(_, amt)| *amt).sum();
    sign.set_send_total(fmt_dcr(send_total).into());
    sign.set_fee(fmt_dcr(summary.fee).into());
    sign.set_change(fmt_dcr(summary.change_total).into());
    sign.set_recipient_count(summary.recipients.len() as i32);
    // recipients rows (summary.recipients) would be mapped into a Slint model
    // here so each destination address + amount is shown individually.
    sign.set_state(SignState::Review);
}

/// Format atoms (1e8 = 1 DCR) as a trimmed decimal DCR string.
fn fmt_dcr(atoms: i64) -> String {
    let neg = atoms < 0;
    let a = atoms.unsigned_abs();
    let whole = a / 100_000_000;
    let frac = a % 100_000_000;
    // 8dp, then trim trailing zeros (keep at least 4dp for readability).
    let mut s = format!("{whole}.{frac:08}");
    while s.ends_with('0') && !s.ends_with(".0000") && s.len() > s.find('.').unwrap() + 5 {
        s.pop();
    }
    format!("{}{} DCR", if neg { "-" } else { "" }, s)
}

/// The actual signing. Fires the secure-element seed prompt, re-derives every
/// input key, verifies prev_scripts, signs, and serializes the full tx. Then
/// hands the result to whichever transport it came from.
fn approve_and_sign(state: StoredValue<AppState>) -> Result<()> {
    let (origin, bytes) = {
        let s = state.borrow();
        s.pending().ok_or_else(|| anyhow!("nothing to sign"))?
    };

    let signed: Vec<u8> = {
        let s = state.borrow();
        // Re-decode the package we stashed at ingest time.
        let req: SignRequest =
            decode_sign_request(&bytes).map_err(|e| anyhow!("bad package: {e}"))?;
        // load_master_key triggers the on-device user confirmation gate and is
        // the single seam that touches the seed (empty passphrase = no BIP39
        // 25th word; wire a prompt here if you support passphrases).
        let master = load_master_key(&s.secp, &s.security, "")
            .map_err(|e| anyhow!("seed error: {e}"))?;
        // sign_request re-derives per-input keys from `master`, verifies each
        // prev_script (ScriptMismatch => refuse), signs SigHashAll low-S, and
        // returns the fully serialized network tx bytes.
        sign_request(&s.secp, &master, &req).map_err(|e| anyhow!("sign failed: {e}"))?
    };

    match origin {
        Origin::Qr => emit_qr(state, &signed),
        Origin::SdCard => {
            use std::io::Write;
            let mut file = fs::FileSystem::<crate::fs_permissions::FileSystemPermissions>::default()
                .open_file("signed.dcrtx", fs::Location::Airlock, fs::OpenFlags { read: false, write: true, create: true })
                .context("creating signed.dcrtx")?;
            file.write_all(&signed).context("writing signed.dcrtx")?;
            let ui = state.borrow().ui();
            ui.global::<SignTx>().set_saved_path("/sdcard/signed.dcrtx".into());
            ui.global::<SignTx>().set_state(SignState::Done);
            Ok(())
        }
    }
}

/// Render the signed tx as an animated UR QR ("dcr-signed-tx") for Cake Wallet
/// to scan and broadcast.
fn emit_qr(state: StoredValue<AppState>, signed: &[u8]) -> Result<()> {
    // TODO: encode `signed` as animated UR frames ("dcr-signed-tx") via
    // foundation-ur and store each frame string. For now store a single hex
    // frame so the DynamicQrCode has data to render in the simulator.
    let parts = vec![hex::encode(signed)];
    state.borrow_mut().set_signed_parts(parts);
    let ui = state.borrow().ui();
    ui.global::<SignTx>().set_state(SignState::ShowQr);
    Ok(())
}

fn show_error(state: StoredValue<AppState>, msg: &str) {
    let ui = state.borrow().ui();
    let sign = ui.global::<SignTx>();
    sign.set_error_text(msg.into());
    sign.set_state(SignState::Error);
}
