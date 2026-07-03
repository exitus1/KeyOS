use slint_keyos_platform::slint::ComponentHandle;
// SPDX-License-Identifier: GPL-3.0-or-later
//
// "Create account" here is lightweight: Decred wallet *creation* (generating /
// restoring the seed) is owned by the OS Seed Vault app, not this one. What
// this screen does is let the user pick an account index (m/44'/42'/N') and
// export that account's extended PUBLIC key (dpub) so a watch-only Cake Wallet
// can be set up to build transactions and track the balance.
//
// Only the account dpub leaves the device — never a private key. The dpub is
// derived from the secure-element seed, so this passes the confirmation gate.

use anyhow::{anyhow, Result};
use slint_keyos_platform::StoredValue;

use decred_core::account_export::{
    encode_account_export, AccountExport, ExportedAccount, ACCOUNT_EXPORT_FORMAT_VERSION,
};

use crate::keys::load_master_key;
use crate::state::AppState;
use crate::{Account, AccountRow, AccountState};
use slint_keyos_platform::slint::{ModelRc, VecModel};

/// Push the named-account list into the Slint model, marking the active one.
pub(crate) fn refresh_rows(state: StoredValue<AppState>) {
    let (rows, active_name, active) = {
        let s = state.borrow();
        let active = s.accounts.active;
        let rows: Vec<AccountRow> = s
            .accounts
            .accounts
            .iter()
            .map(|a| {
                // Small deterministic accent palette, cycled by index.
                const PALETTE: [(u8, u8, u8); 5] = [
                    (0x29, 0x70, 0xff), // decred blue
                    (0x2d, 0xd8, 0xa3), // decred teal
                    (0x8b, 0x5c, 0xf6), // violet
                    (0xf5, 0x9e, 0x0b), // amber
                    (0xec, 0x48, 0x99), // pink
                ];
                let (r, g, b) = PALETTE[(a.index as usize) % PALETTE.len()];
                let initial: String = a
                    .name
                    .chars()
                    .next()
                    .map(|c| c.to_uppercase().collect::<String>())
                    .unwrap_or_default();
                AccountRow {
                    idx: a.index as i32,
                    name: a.name.as_str().into(),
                    path: format!("m/44'/42'/{}'", a.index).into(),
                    active: a.index == active,
                    initial: initial.into(),
                    accent: slint_keyos_platform::slint::Color::from_rgb_u8(r, g, b),
                }
            })
            .collect();
        (rows, s.accounts.active_name(), active)
    };
    let ui = state.borrow().ui();
    let acct = ui.global::<Account>();
    acct.set_rows(ModelRc::new(VecModel::from(rows)));
    acct.set_active_name(active_name.into());
    acct.set_active_index(active as i32);
}

pub fn init(state: StoredValue<AppState>) {
    // First launch: give the user a default account so the picker is never
    // empty, and sync the live derivation index with the persisted choice.
    {
        let mut s = state.borrow_mut();
        if s.accounts.is_empty() {
            let _ = s.accounts.create("Main");
        }
        s.account = s.accounts.active;
    }
    refresh_rows(state);

    let ui = state.borrow().ui();
    let acct = ui.global::<Account>();

    // User named a new account in the picker.
    acct.on_create_account({
        move |name| {
            let result = {
                let mut s = state.borrow_mut();
                let r = s.accounts.create(name.as_str());
                if let Ok(i) = r {
                    s.account = i;
                }
                r
            };
            {
                let ui = state.borrow().ui();
                let acct = ui.global::<Account>();
                match result {
                    Ok(_) => acct.set_name_error("".into()),
                    Err(e) => acct.set_name_error(e.into()),
                }
            }
            refresh_rows(state);
        }
    });

    // User picks an account index and taps "Export watch-only key".
    acct.on_export_account_xpub({
        move |index| {
            match export_dpub(state, index as u32) {
                Ok((dpub, fp)) => {
                    // Cache the fingerprint while it is in hand: it is the
                    // account's true cross-airgap identity, and lets companion
                    // balances match without another secure-element prompt.
                    if state.borrow_mut().accounts.set_fp(index as u32, fp) {
                        crate::balance::render(state);
                    }
                    // Name lookup + verification head/tail for the page. The
                    // stashed export carries the STORE name (empty when the
                    // browse index is unnamed), not the display fallback.
                    let store_name = {
                        let s = state.borrow();
                        s.accounts
                            .accounts
                            .iter()
                            .find(|a| a.index == index as u32)
                            .map(|a| a.name.clone())
                    };
                    state.borrow_mut().set_last_dpub_export(ExportedAccount {
                        account: index as u32,
                        dpub: dpub.clone(),
                        name: store_name.clone().unwrap_or_default(),
                    });
                    let name = store_name
                        .unwrap_or_else(|| format!("Account #{index} (unnamed)"));
                    let head: String = dpub.chars().take(12).collect();
                    let tail: String = dpub
                        .chars()
                        .rev()
                        .take(12)
                        .collect::<Vec<_>>()
                        .into_iter()
                        .rev()
                        .collect();
                    let ui = state.borrow().ui();
                    let acct = ui.global::<Account>();
                    acct.set_sd_note("".into());
                    acct.set_export_name(name.into());
                    acct.set_export_path(format!("m/44'/42'/{index}'").into());
                    acct.set_dpub_head(head.into());
                    acct.set_dpub_tail(tail.into());
                    acct.set_account_dpub(dpub.into());
                    acct.set_state(AccountState::Exported);
                }
                Err(e) => {
                    log::error!("dpub export failed: {e:?}");
                    let ui = state.borrow().ui();
                    ui.global::<Account>().set_error_text(e.to_string().into());
                    ui.global::<Account>().set_state(AccountState::Error);
                }
            }
        }
    });
    // Switch the active account index; everything downstream (receive, sign,
    // export, balance) derives from state.account. If the index has a named
    // account it is persisted as the new default; the export page's raw
    // stepper may visit unnamed indices, which switch live but don't persist.
    acct.on_set_active_account({
        move |index| {
            let idx = index.max(0) as u32;
            {
                let mut s = state.borrow_mut();
                s.accounts.set_active(idx);
                s.account = idx;
            }
            refresh_rows(state);
            let ui = state.borrow().ui();
            ui.global::<Account>().set_active_index(idx as i32);
            // The home card shows the ACTIVE account's companion balance.
            crate::balance::render(state);
            log::info!("active account -> {}", idx);
        }
    });

    // Write the export page's current account to the SD card.
    acct.on_write_account_export({
        move || {
            let note = match state.borrow().last_dpub_export() {
                Some(entry) => match write_accounts_file(&[entry]) {
                    Ok(()) => "accounts.dcr written to SD card".to_string(),
                    Err(e) => {
                        log::error!("account export write failed: {e:?}");
                        format!("Write failed: {e}")
                    }
                },
                None => "Nothing exported yet".to_string(),
            };
            let ui = state.borrow().ui();
            ui.global::<Account>().set_sd_note(note.into());
        }
    });

    // Write every named account to the SD card in one file (one seed prompt).
    acct.on_export_all_accounts({
        move || {
            let note = match export_all_accounts(state) {
                Ok(n) => format!(
                    "{n} account{} written to accounts.dcr",
                    if n == 1 { "" } else { "s" }
                ),
                Err(e) => {
                    log::error!("export all accounts failed: {e:?}");
                    format!("Export failed: {e}")
                }
            };
            let ui = state.borrow().ui();
            ui.global::<Account>().set_sd_note(note.into());
            // Fingerprints were backfilled along the way; re-match balances.
            crate::balance::render(state);
        }
    });
}

/// Derive every named account's dpub (one seed prompt) and write them all as
/// a single accounts.dcr, sorted by index. Caches each fingerprint while the
/// keys are in hand.
fn export_all_accounts(state: StoredValue<AppState>) -> Result<usize> {
    let (entries, fps) = {
        let s = state.borrow();
        let master =
            load_master_key(&s.secp, &s.security, &s.passphrase).map_err(|e| anyhow!("{e}"))?;
        let mut list: Vec<_> = s.accounts.accounts.iter().collect();
        list.sort_by_key(|a| a.index);
        let mut entries = Vec::with_capacity(list.len());
        let mut fps = Vec::with_capacity(list.len());
        for a in list {
            let key = master.account_key(&s.secp, a.index).map_err(|e| anyhow!("{e}"))?;
            entries.push(ExportedAccount {
                account: a.index,
                dpub: key.to_dpub(&s.secp),
                name: a.name.clone(),
            });
            fps.push((a.index, key.fingerprint(&s.secp)));
        }
        (entries, fps)
    };
    {
        let mut s = state.borrow_mut();
        for (idx, fp) in fps {
            s.accounts.set_fp(idx, fp);
        }
    }
    write_accounts_file(&entries)?;
    Ok(entries.len())
}

/// Encode the entries as an AccountExport and write accounts.dcr to the card.
fn write_accounts_file(entries: &[ExportedAccount]) -> Result<()> {
    let exp = AccountExport {
        format_version: ACCOUNT_EXPORT_FORMAT_VERSION,
        accounts: entries.to_vec(),
    };
    let bytes = encode_account_export(&exp).map_err(|e| anyhow!("{e}"))?;
    let path = std::path::Path::new(crate::sign_tx::card_dir_pub()).join("accounts.dcr");
    std::fs::write(&path, &bytes).map_err(|e| anyhow!("write {}: {e}", path.display()))?;
    log::info!("wrote {} account(s) to {}", entries.len(), path.display());
    Ok(())
}

/// Derive the account key at m/44'/42'/index' and return its neutered dpub
/// plus its fingerprint (cached for companion balance matching).
fn export_dpub(state: StoredValue<AppState>, index: u32) -> Result<(String, [u8; 4])> {
    let s = state.borrow();
    let master = load_master_key(&s.secp, &s.security, &s.passphrase).map_err(|e| anyhow!("{e}"))?;
    let account = master.account_key(&s.secp, index).map_err(|e| anyhow!("{e}"))?;
    // Export the NEUTERED account key (dpub) — public only, safe to hand to a
    // watch-only Cake Wallet. Never exports private material.
    Ok((account.to_dpub(&s.secp), account.fingerprint(&s.secp)))
}
