// SPDX-License-Identifier: GPL-3.0-or-later
//
// Signing flow. This is where the airgap package comes in (QR or SD), gets
// reviewed on-screen, gets signed with keys re-derived from the secure element,
// and goes back out (QR or SD).
//
// Decred has no PSBT. The shared dcr-rs library (re-exported as decred-core)
// defines a compact CBOR "unsigned-tx package" (airgap::SignRequest,
// FORMAT_VERSION = 1) carrying per input the prev_script + amount + derivation
// path. The companion (DCR Pulse / Cake Wallet) is watch-only: it knows the
// UTXOs, scripts, paths and builds that package. The device re-derives each
// input key, recomputes its P2PKH script, and REFUSES to sign if the
// recomputed script != the script the host claimed (Error::ScriptMismatch).
// That check is the anti-tamper tripwire, and it now runs at REVIEW time (from
// the cached account xpub — no seed access) as well as at sign time.
//
// Seed-prompt economy: review, wrong-wallet detection and change
// classification all run from the account-level PUBLIC key cached in AppState
// (one prompt per account per session). Only the actual signature loads the
// master key, uses it, and drops it — dcr-rs zeroizes it on drop.
//
// Two transports, one core:
//   QR : foundation-ur animated UR frames, type "dcr-sign-request" in,
//        "dcr-signed-tx" out.
//   SD : pick a *.dcrtx file (Airlock on hardware, $DECRED_FUZZ_DIR or
//        ~/fuzz in the hosted sim), write signed.dcrtx back the same way.
use anyhow::{anyhow, Result};
use slint_keyos_platform::gui_server_api::navigation::qrscanner::{ScanQrOptions, ScanQrResult};
use slint_keyos_platform::navigation::open_qr_scanner;
use decred_core::airgap::{decode_sign_request, sign_request, ReviewSummary, SignRequest};
use slint_keyos_platform::slint::ComponentHandle;
use slint_keyos_platform::slint::{ModelRc, VecModel};
use slint_keyos_platform::StoredValue;

use crate::keys::load_master_key;
use crate::state::AppState;
// Slint-generated globals/enums (emitted into the crate root by `app!`).
use crate::{OriginView, RecipientView, SignState, SignTx};

/// Where a given signing request arrived from. Mirrors the Bitcoin app's
/// PsbtOrigin (File | Qr | QuantumLink); we drop QuantumLink for now.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Origin {
    /// Constructed once the camera/UR pump is wired into `begin_scan`.
    #[allow(dead_code)]
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
    // Enter the picker: list the card's .dcrtx files and show them.
    sign.on_load_from_sd({
        move || {
            if let Err(e) = list_sd_files(state) {
                log::error!("list sd files failed: {e:?}");
                show_error(state, &e.to_string());
            }
        }
    });

    // User tapped a file in the picker.
    sign.on_pick_file({
        move |name| {
            if let Err(e) = load_named_file(state, name.as_str()) {
                log::error!("load {name} failed: {e:?}");
                show_error(state, &e.to_string());
            }
        }
    });

    // Hidden debug affordance: rotate through every file to fuzz-test.
    // Compiled to an error on hardware — it exists for the hosted sim only.
    sign.on_debug_cycle({
        move || {
            if let Err(e) = debug_inject_karamble_file(state) {
                log::error!("fuzz cycle failed: {e:?}");
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
    // been signed; empty until then.
    sign.on_signed_qr_parts({
        move || -> slint_keyos_platform::slint::ModelRc<slint_keyos_platform::slint::SharedString> {
            let parts = state.borrow().signed_qr_parts();
            slint_keyos_platform::slint::ModelRc::new(slint_keyos_platform::slint::VecModel::from(
                parts.into_iter().map(slint_keyos_platform::slint::SharedString::from).collect::<Vec<_>>(),
            ))
        }
    });
}

/// Deep-link / button entry: start the animated-QR scanner. The actual frame
/// pump lives in a spawn_local loop that feeds foundation-ur until a complete
/// "dcr-sign-request" is assembled, then calls `ingest`.
pub fn begin_scan(state: StoredValue<AppState>) -> Result<()> {
    // The OS provides the entire scanner: camera, QR detection, and
    // animated-UR fountain reassembly. We open it as a modal and receive the
    // finished payload — the same pattern the Bitcoin app uses for PSBTs.
    // (In the hosted simulator there is no camera; the modal opens and the
    // user can only cancel. Real frames arrive on hardware.)
    let opts = ScanQrOptions {
        header_title: "Scan from companion".into(),
        message: "Point the camera at the QR shown by your companion wallet.".into(),
        header_left_icon: String::new(),
        header_right_icon: String::from("close"),
        ..ScanQrOptions::default()
    };

    let scan = match open_qr_scanner::<crate::gui_permissions::GuiPermissions>(opts) {
        Ok(Some(s)) => s,
        Ok(None) => return Ok(()),
        Err(e) => return Err(anyhow!("scanner: {e:?}")),
    };

    match scan {
        // Animated/typed UR: the OS hands us the UR type + reassembled bytes.
        ScanQrResult::Ur2(ur_type, data) => match ur_type.as_str() {
            "dcr-sign-request" => {
                let ui = state.borrow().ui();
                ui.global::<SignTx>().set_origin(OriginView::Qr);
                ingest(state, Origin::Qr, &data)
            }
            "dcr-balance" => {
                // Balance payload is plain UTF-8 key=value text.
                let text = String::from_utf8(data)
                    .map_err(|e| anyhow!("balance payload not UTF-8: {e}"))?;
                crate::balance::apply_text(state, &text, "QR")
                    .map_err(|e| anyhow!("balance: {e}"))
            }
            other => {
                show_error(state, &format!("Unsupported QR type: {other}"));
                Ok(())
            }
        },
        // Plain (non-UR) QR: accept a single-part UR string as text.
        ScanQrResult::Qr(data) => {
            let text = String::from_utf8_lossy(&data);
            if text.trim().to_uppercase().starts_with("UR:DCR-BALANCE/") {
                crate::balance::ingest_qr(state, text.trim()).map_err(|e| anyhow!("{e}"))
            } else {
                show_error(state, "Unrecognized QR code.");
                Ok(())
            }
        }
        // Cancelled from the scanner UI: stay wherever we were.
        _ => Ok(()),
    }
}

// ---------------------------------------------------------------------------
// Card storage. On hardware the card is the Airlock (the vetted exchange area
// the manifest requests read/write access to). The hosted simulator has no
// Airlock disk image, so it uses a local directory instead: $DECRED_FUZZ_DIR
// if set, otherwise ~/fuzz. These four functions are the ONLY place the two
// worlds differ; everything above them is transport-agnostic.
// ---------------------------------------------------------------------------

/// One listed card file: name + human-readable size, newest first.
struct CardFile {
    name: String,
    detail: String,
}

#[cfg(not(target_os = "xous"))]
fn sim_card_dir() -> std::path::PathBuf {
    match std::env::var_os("DECRED_FUZZ_DIR") {
        Some(dir) => dir.into(),
        None => {
            let mut p: std::path::PathBuf =
                std::env::var_os("HOME").map(Into::into).unwrap_or_else(|| "/tmp".into());
            p.push("fuzz");
            p
        }
    }
}

/// List the card's .dcrtx files, newest first.
#[cfg(not(target_os = "xous"))]
fn list_card_files() -> Result<Vec<CardFile>> {
    let dir = sim_card_dir();
    let mut entries: Vec<(std::path::PathBuf, std::time::SystemTime, u64)> = std::fs::read_dir(&dir)
        .map_err(|e| anyhow!("no card / cannot read {}: {e}", dir.display()))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "dcrtx").unwrap_or(false))
        .map(|p| {
            let meta = std::fs::metadata(&p).ok();
            let modified = meta.as_ref().and_then(|m| m.modified().ok()).unwrap_or(std::time::UNIX_EPOCH);
            let len = meta.map(|m| m.len()).unwrap_or(0);
            (p, modified, len)
        })
        .collect();
    entries.sort_by(|a, b| b.1.cmp(&a.1)); // newest first
    Ok(entries
        .iter()
        .map(|(p, _, len)| CardFile {
            name: p.file_name().and_then(|n| n.to_str()).unwrap_or("?").to_string(),
            detail: format!("{:.1} KB", *len as f64 / 1024.0),
        })
        .collect())
}

#[cfg(target_os = "xous")]
fn list_card_files() -> Result<Vec<CardFile>> {
    let fs = fs::FileSystem::<crate::fs_permissions::FileSystemPermissions>::default();
    let dir = fs
        .open_dir("/", fs::Location::Airlock)
        .map_err(|e| anyhow!("no card / cannot open Airlock: {e:?}"))?;
    let mut entries: Vec<fs::DirEntry> = Vec::new();
    loop {
        match dir.next_entry() {
            Ok(Some(entry)) => {
                if entry.is_file && entry.name.to_lowercase().ends_with(".dcrtx") {
                    entries.push(entry);
                }
            }
            Ok(None) => break,
            Err(e) => return Err(anyhow!("listing Airlock: {e:?}")),
        }
    }
    entries.sort_by(|a, b| b.modified.cmp(&a.modified)); // newest first
    Ok(entries
        .into_iter()
        .map(|e| CardFile { detail: format!("{:.1} KB", e.len as f64 / 1024.0), name: e.name })
        .collect())
}

/// Read one named .dcrtx file off the card. `name` has already been vetted by
/// `load_named_file` (no separators, no traversal).
#[cfg(not(target_os = "xous"))]
pub(crate) fn read_card_file(name: &str) -> Result<Vec<u8>> {
    let path = sim_card_dir().join(name);
    std::fs::read(&path).map_err(|e| anyhow!("read {}: {e}", path.display()))
}

#[cfg(target_os = "xous")]
pub(crate) fn read_card_file(name: &str) -> Result<Vec<u8>> {
    use std::io::Read;
    let mut opened = fs::FileSystem::<crate::fs_permissions::FileSystemPermissions>::default()
        .open_file(name, fs::Location::Airlock, fs::OpenFlags { read: true, write: false, create: false })
        .map_err(|e| anyhow!("opening {name}: {e:?}"))?;
    let mut bytes = Vec::new();
    opened.read_to_end(&mut bytes).map_err(|e| anyhow!("reading {name}: {e}"))?;
    Ok(bytes)
}

/// Write the signed tx back to the card. Returns the display path.
#[cfg(not(target_os = "xous"))]
fn write_signed_to_card(signed: &[u8]) -> Result<String> {
    // Unique filename per signing so successive signs don't overwrite each
    // other; a short hash of the signed bytes is the tag. A hex twin is kept
    // for easy broadcast during interop testing.
    let tag: String = hex::encode(signed).chars().take(12).collect();
    let dir = sim_card_dir();
    let path = dir.join(format!("decred_signed_{tag}.dcrtx"));
    let hex_path = dir.join(format!("decred_signed_{tag}.hex"));
    std::fs::write(&path, signed).map_err(|e| anyhow!("writing {}: {e}", path.display()))?;
    std::fs::write(&hex_path, hex::encode(signed))
        .map_err(|e| anyhow!("writing {}: {e}", hex_path.display()))?;
    log::info!("SIGNED TX written: {} ({} bytes)", path.display(), signed.len());
    Ok(path.display().to_string())
}

#[cfg(target_os = "xous")]
fn write_signed_to_card(signed: &[u8]) -> Result<String> {
    use std::io::Write;
    let mut file = fs::FileSystem::<crate::fs_permissions::FileSystemPermissions>::default()
        .open_file(
            "signed.dcrtx",
            fs::Location::Airlock,
            fs::OpenFlags { read: false, write: true, create: true },
        )
        .map_err(|e| anyhow!("creating signed.dcrtx: {e:?}"))?;
    file.write_all(signed).map_err(|e| anyhow!("writing signed.dcrtx: {e}"))?;
    Ok("signed.dcrtx (SD card)".to_string())
}

/// List .dcrtx files on the card, newest first, and enter the picker.
fn list_sd_files(state: StoredValue<AppState>) -> Result<()> {
    let files = list_card_files()?;
    if files.is_empty() {
        return Err(anyhow!("No transaction files (.dcrtx) found on the card."));
    }
    let rows: Vec<crate::SdFile> =
        files.into_iter().map(|f| crate::SdFile { name: f.name.into(), detail: f.detail.into() }).collect();

    let ui = state.borrow().ui();
    let sign = ui.global::<SignTx>();
    sign.set_sd_files(slint_keyos_platform::slint::ModelRc::new(
        slint_keyos_platform::slint::VecModel::from(rows),
    ));
    sign.set_state(SignState::Picking);
    Ok(())
}

/// Load one named file from the card and go to review.
fn load_named_file(state: StoredValue<AppState>, name: &str) -> Result<()> {
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err(anyhow!("invalid file name"));
    }
    let bytes = read_card_file(name)?;
    ingest(state, Origin::SdCard, &bytes)
}

/// Common path for both transports: decode the package, verify it against the
/// wallet's own (public) keys, derive a review summary for the on-device
/// confirmation screen, and stash it pending approval. Uses the session's
/// cached account xpub, so this prompts for seed access at most once per
/// account per session — the master key itself is only loaded when the user
/// actually approves.
pub fn ingest(state: StoredValue<AppState>, origin: Origin, bytes: &[u8]) -> Result<()> {
    let req: SignRequest = decode_sign_request(bytes).map_err(|e| anyhow!("bad package: {e}"))?;
    // REFUSE dishonest math before anything is even shown for review.
    req.validate().map_err(|e| anyhow!("REFUSED: {e}"))?;

    let summary: ReviewSummary = {
        let mut s = state.borrow_mut();
        let xpub = s.account_xpub(req.account).map_err(|e| anyhow!("seed error: {e}"))?;

        // WRONG-WALLET DETECTION (optional field from the companion): if the
        // request names the account fingerprint it was built against, verify
        // it matches this wallet+account before showing anything. Pure UX —
        // the prev_script check below still protects funds without it.
        if let Some(fp) = req.account_fp {
            if xpub.fingerprint() != fp {
                return Err(anyhow!(
                    "This transaction was built for a DIFFERENT wallet or account. \
                     Open the wallet it belongs to (check the passphrase) and try again."
                ));
            }
        }

        // ANTI-TAMPER, moved up to review time: every input's prev_script must
        // re-derive from OUR account key. A package that would fail signing is
        // rejected here, before the user is even asked to look at it.
        req.check_owned_inputs(&s.secp, &xpub).map_err(|e| anyhow!("REFUSED: {e}"))?;

        // TRUSTLESS REVIEW: re-derive our own addresses and classify each
        // output ourselves instead of trusting the companion's is_change flag.
        req.review_owned(&s.secp, &xpub).map_err(|e| anyhow!("review failed: {e}"))?
    };

    // Persist the raw bytes + origin so approve_and_sign can re-decode and sign.
    {
        let mut s = state.borrow_mut();
        s.set_pending(origin, bytes.to_vec());
    }

    render_review(state, &summary, req.account, req.inputs.len(), req.outputs.len());
    Ok(())
}

/// Alarm threshold for the review screen's high-fee warning: > 0.1 DCR
/// absolute, or > 5% of the amount sent (only when the fee also exceeds
/// 0.01 DCR, so tiny everyday transactions never false-alarm). UI policy, so
/// it lives here in the app rather than in the consensus library.
fn fee_is_worrying(fee: i64, recipients: &[(String, i64)]) -> bool {
    const ABS: i64 = 10_000_000; // 0.1 DCR
    const MIN_FOR_PCT: i64 = 1_000_000; // 0.01 DCR
    if fee > ABS {
        return true;
    }
    let sent: i64 = recipients.iter().map(|(_, a)| *a).sum();
    fee > MIN_FOR_PCT && sent > 0 && fee > sent / 20
}

/// Push the human-readable review (recipients, change, fee) into Slint.
/// Amounts are formatted to DCR strings here because Slint's `int` is 32-bit
/// and atom values (1 DCR = 1e8 atoms) overflow it; the UI shows "1.2345 DCR".
fn render_review(
    state: StoredValue<AppState>,
    summary: &ReviewSummary,
    account: u32,
    n_in: usize,
    n_out: usize,
) {
    let ui = state.borrow().ui();
    let sign = ui.global::<SignTx>();
    let send_total: i64 = summary.recipients.iter().map(|(_, amt)| *amt).sum();
    let change_total: i64 = summary.change.iter().map(|(_, amt)| *amt).sum();
    sign.set_send_total(fmt_dcr(send_total).into());
    sign.set_fee(fmt_dcr(summary.fee).into());
    sign.set_change(fmt_dcr(change_total).into());
    sign.set_recipient_count(summary.recipients.len() as i32);
    sign.set_flagged_count(summary.flagged_mismatches.len() as i32);
    sign.set_fee_warning(fee_is_worrying(summary.fee, &summary.recipients));
    // Reset the acknowledgment each time a new tx is reviewed.
    sign.set_mismatch_acknowledged(false);
    // Join recipient address(es) + amount for on-screen verification.
    let recipient_str: String = summary
        .recipients
        .iter()
        .map(|(addr, amt)| format!("{addr}\n{}", fmt_dcr(*amt)))
        .collect::<Vec<_>>()
        .join("\n\n");
    sign.set_recipient(recipient_str.into());

    // Per-recipient verification cards: 5-char groups over two rows; the
    // page emphasizes the outer groups (what humans actually compare).
    let mut views: Vec<RecipientView> = Vec::new();
    for (addr, amt) in &summary.recipients {
        let groups: Vec<String> =
            addr.as_bytes().chunks(5).map(|c| core::str::from_utf8(c).unwrap_or("").to_string()).collect();
        let split = groups.len().div_ceil(2);
        let (top, bottom) = groups.split_at(split);
        views.push(RecipientView {
            top_first: top.first().cloned().unwrap_or_default().into(),
            top_rest: top.get(1..).unwrap_or(&[]).join(" ").into(),
            bottom_rest: bottom.get(..bottom.len().saturating_sub(1)).unwrap_or(&[]).join(" ").into(),
            bottom_last: bottom.last().cloned().unwrap_or_default().into(),
            amount: fmt_dcr(*amt).into(),
        });
    }
    sign.set_recipients(ModelRc::new(VecModel::from(views)));

    // Which account this request spends from (named if we know it). Uses the
    // REQUEST's account, which is what signing will actually use.
    let account_label = {
        let s = state.borrow();
        s.accounts
            .accounts
            .iter()
            .find(|a| a.index == account)
            .map(|a| a.name.clone())
            .unwrap_or_else(|| format!("Account #{account}"))
    };
    sign.set_signing_account(account_label.into());

    // Estimated fee rate. Exact size is only known after signing, so this is
    // an estimate from typical P2PKH input/output sizes.
    let est_size = 12 + 166 * n_in as i64 + 36 * n_out as i64;
    let rate = if est_size > 0 { summary.fee / est_size } else { 0 };
    sign.set_fee_rate(format!("≈ {rate} atoms/B (est.)").into());
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

/// The actual signing. Re-verifies the package against the review gate (the
/// UI's acknowledgment cannot be bypassed by a code path that skips the
/// screen), fires the secure-element seed prompt ONCE, re-derives every input
/// key, verifies prev_scripts, signs, and serializes the full tx. Then hands
/// the result to whichever transport it came from and clears the pending
/// request so the same package cannot be signed twice by accident.
fn approve_and_sign(state: StoredValue<AppState>) -> Result<()> {
    let (origin, bytes) = {
        let s = state.borrow();
        s.pending().ok_or_else(|| anyhow!("nothing to sign"))?
    };

    // Re-decode the package we stashed at ingest time.
    let req: SignRequest = decode_sign_request(&bytes).map_err(|e| anyhow!("bad package: {e}"))?;

    // DEFENSE IN DEPTH: recompute the review from the cached xpub and re-check
    // the acknowledgment gate the UI enforces. Signing must refuse on its own
    // if the review found mislabelled change or a worrying fee and the user
    // has not explicitly acknowledged it — even if a future UI change (or bug)
    // were to leave the Approve button enabled.
    {
        let mut s = state.borrow_mut();
        let xpub = s.account_xpub(req.account).map_err(|e| anyhow!("seed error: {e}"))?;
        let summary = req.review_owned(&s.secp, &xpub).map_err(|e| anyhow!("review failed: {e}"))?;
        let needs_ack =
            !summary.flagged_mismatches.is_empty() || fee_is_worrying(summary.fee, &summary.recipients);
        drop(s);
        if needs_ack {
            let ui = state.borrow().ui();
            if !ui.global::<SignTx>().get_mismatch_acknowledged() {
                return Err(anyhow!("REFUSED: review warnings were not acknowledged"));
            }
        }
    }

    let signed: Vec<u8> = {
        let s = state.borrow();
        // load_master_key triggers the on-device user confirmation gate and is
        // the single seam that touches the seed. sign_request re-validates,
        // re-derives per-input keys, verifies each prev_script (ScriptMismatch
        // => refuse), signs SigHashAll low-S, and returns the fully serialized
        // network tx bytes. The master key is dropped — and zeroized — on exit
        // from this block.
        let master = load_master_key(&s.security, &s.passphrase).map_err(|e| anyhow!("seed error: {e}"))?;
        sign_request(&s.secp, &master, &req).map_err(|e| anyhow!("sign failed: {e}"))?
    };

    // The request is consumed: a fresh scan/load is required to sign again.
    state.borrow_mut().clear_pending();

    match origin {
        Origin::Qr => emit_qr(state, &signed),
        Origin::SdCard => {
            let shown_path = write_signed_to_card(&signed)?;
            let ui = state.borrow().ui();
            ui.global::<SignTx>().set_saved_path(shown_path.as_str().into());
            ui.global::<SignTx>().set_state(SignState::Done);
            Ok(())
        }
    }
}

/// Render the signed tx as an animated UR QR for the companion to scan and
/// broadcast.
fn emit_qr(state: StoredValue<AppState>, signed: &[u8]) -> Result<()> {
    use foundation_ur::Encoder;
    // Bytes per QR frame. ~90 keeps each frame comfortably scannable; the tx is
    // split across `sequence_count()` frames that the DynamicQrCode animates.
    const MAX_FRAGMENT_LEN: usize = 90;

    // "bytes" is the generic UR type any BC-UR reader (a phone, Cake) can decode.
    // Switch to a Decred-specific "dcr-signed-tx" type once both ends agree on it.
    let mut encoder = Encoder::new();
    encoder.start("bytes", signed, MAX_FRAGMENT_LEN);

    // Emit a FOUNTAIN stream, not just the fixed set. foundation-ur keeps
    // producing XOR-combined fragments past sequence_count(); a fountain
    // decoder reassembles from any sufficient subset without waiting for the
    // animation to cycle back to a missed frame. Small txs get 3x redundancy;
    // large ones are capped at ~25% overhead so the precomputed frame list
    // stays bounded on-device (a decoder needs at least sequence_count()
    // distinct frames, so the count must never drop below that).
    let seq = encoder.sequence_count().max(1) as usize;
    let frame_count = if seq <= 64 { (seq * 3).max(8) } else { seq + (seq / 4).max(8) };
    let mut parts = Vec::with_capacity(frame_count);
    for _ in 0..frame_count {
        parts.push(encoder.next_part().to_string());
    }

    log::info!("emit_qr: {} byte tx -> {} UR fountain frame(s) (seq={})", signed.len(), parts.len(), seq);
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

// ---------------------------------------------------------------------------
// DEBUG (hosted sim only): feed adversarial packages through the same `ingest`
// path the SD/QR transports use, so the GUI review→approve→sign flow can be
// exercised without an Airlock image or camera. Compiled out on hardware.
// ---------------------------------------------------------------------------

/// FUZZ MODE: cycle through every *.dcrtx in the sim card dir on each tap, so
/// a batch of adversarial files can be walked through while watching the
/// device reject each one. Index persists in a counter file next to them.
#[cfg(not(target_os = "xous"))]
pub fn debug_inject_karamble_file(state: StoredValue<AppState>) -> Result<()> {
    let dir = sim_card_dir();
    let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(&dir)
        .map_err(|e| anyhow!("read fuzz dir {}: {e}", dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().map(|x| x == "dcrtx").unwrap_or(false))
        .collect();
    files.sort();
    if files.is_empty() {
        return Err(anyhow!("no .dcrtx files in {}", dir.display()));
    }
    // Persisted rotating index.
    let idx_path = dir.join(".fuzz_idx");
    let idx: usize = std::fs::read_to_string(&idx_path).ok().and_then(|s| s.trim().parse().ok()).unwrap_or(0);
    let pick = idx % files.len();
    let file = &files[pick];
    let _ = std::fs::write(&idx_path, ((pick + 1) % files.len()).to_string());
    log::info!("FUZZ: loading [{}/{}] {}", pick + 1, files.len(), file.display());
    let bytes = std::fs::read(file).map_err(|e| anyhow!("read {}: {e}", file.display()))?;
    ingest(state, Origin::SdCard, &bytes)
}

#[cfg(target_os = "xous")]
pub fn debug_inject_karamble_file(_state: StoredValue<AppState>) -> Result<()> {
    Err(anyhow!("debug fuzz cycling is simulator-only"))
}

/// DEBUG (hosted sim only): build a known unsigned tx in-memory and feed it
/// into the same `ingest` path. The single input's prev_script is derived from
/// THIS device's own index-0 key, so signing's anti-tamper script check passes
/// exactly as on real hardware.
#[cfg(not(target_os = "xous"))]
#[allow(dead_code)]
pub fn debug_inject_test_tx(state: StoredValue<AppState>) -> Result<()> {
    use decred_core::address::p2pkh_script;
    use decred_core::airgap::{encode_sign_request, InputMeta, OutputMeta, FORMAT_VERSION};
    use decred_core::hashing::hash160;
    use decred_core::hd::BRANCH_EXTERNAL;

    let (prev_script, dest_script) = {
        let mut s = state.borrow_mut();
        let xpub = s.account_xpub(0).map_err(|e| anyhow!("seed error: {e}"))?;
        let pk0 = xpub.pubkey_at(&s.secp, BRANCH_EXTERNAL, 0).map_err(|e| anyhow!("addr0: {e}"))?;
        let pk1 = xpub.pubkey_at(&s.secp, BRANCH_EXTERNAL, 1).map_err(|e| anyhow!("addr1: {e}"))?;
        (p2pkh_script(&hash160(&pk0)).to_vec(), p2pkh_script(&hash160(&pk1)).to_vec())
    };

    // REAL prevout: funding tx 37564c16...d954, vout 0, 100000 atoms, to index-0.
    // txid is given in display (big-endian) order; reverse to internal byte order.
    let txid_display = "37564c16ef112d03c1fd44df93c0fd2703b057580797de6489463bcabfe5d954";
    let raw: Vec<u8> = (0..txid_display.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&txid_display[i..i + 2], 16).unwrap())
        .collect();
    let mut prev_hash = [0u8; 32];
    for (i, b) in raw.iter().rev().enumerate() {
        prev_hash[i] = *b;
    }

    let input = InputMeta {
        prev_hash,
        prev_index: 0, // vout 0 (the output paying our index-0 address)
        tree: 0,
        sequence: 0xffff_ffff,
        value_in: 100_000, // 0.001 DCR (exact)
        branch: BRANCH_EXTERNAL,
        index: 0, // device re-derives m/44'/42'/0'/0/0, checks prev_script
        prev_script,
    };
    let output = OutputMeta {
        value: 94_000, // 0.00094 DCR to index-1; fee = 6000 atoms
        version: 0,
        pk_script: dest_script,
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
        account_fp: None,
    };
    let bytes = encode_sign_request(&req).map_err(|e| anyhow!("encode: {e}"))?;
    log::info!("debug_inject_test_tx: built {} byte unsigned package", bytes.len());
    // Origin::SdCard: symmetric file transport. The signed.dcrtx is written
    // out as a file (see approve_and_sign), matching how it was "loaded".
    ingest(state, Origin::SdCard, &bytes)
}
