// SPDX-License-Identifier: GPL-3.0-or-later
//
// Passphrase (hidden) wallets. A BIP39 passphrase combined with the device
// seed yields a COMPLETELY separate wallet: its own accounts, addresses and
// watch-only export. Opt-in only; the empty passphrase is the default wallet
// and nothing about it changes.
//
// Typo protection (pattern borrowed from the Bitcoin app's fingerprint
// preview): after opening, the wallet's CODE — the first 8 characters of its
// first receive address (account 0, m/44'/42'/0'/0/0) — is displayed. The
// same passphrase always shows the same code; a typo shows a different code,
// so "wrong wallet" is visible before any funds move.
//
// The passphrase lives in AppState for the session only and is never written
// anywhere. Hidden wallets use an ephemeral in-memory account list.

use slint_keyos_platform::slint::ComponentHandle;
use slint_keyos_platform::StoredValue;

use crate::account_store::AccountStore;
use crate::keys::{load_master_key, receive_address};
use crate::state::AppState;
use crate::Passphrase;

pub fn init(state: StoredValue<AppState>) {
    let ui = state.borrow().ui();
    let pp = ui.global::<Passphrase>();

    // Show the fingerprint for the typed passphrase without switching wallets
    // (Bitcoin app's try_passphrase pattern). The confirm step applies it.
    pp.on_preview({
        move |text| {
            match derive_code(state, text.as_str()) {
                Ok(code) => {
                    let ui = state.borrow().ui();
                    let g = ui.global::<Passphrase>();
                    g.set_code(code.into());
                    g.set_error("".into());
                }
                Err(e) => set_error(state, &e),
            }
        }
    });

    // Open the wallet for the typed passphrase (empty = default wallet).
    pp.on_apply({
        move |text| {
            apply(state, text.as_str());
        }
    });

    // Leave the hidden wallet, back to the default.
    pp.on_clear({
        move || {
            apply(state, "");
        }
    });
}

/// Fingerprint of the wallet a passphrase would open: the first 8 characters
/// of its first receive address (account 0, m/44'/42'/0'/0/0). Deterministic:
/// same passphrase, same fingerprint, always.
fn derive_code(state: StoredValue<AppState>, passphrase: &str) -> Result<String, String> {
    let s = state.borrow();
    let master =
        load_master_key(&s.secp, &s.security, passphrase).map_err(|e| format!("{e}"))?;
    let addr = receive_address(&s.secp, &master, 0, 0)
        .map_err(|e| format!("derivation failed: {e}"))?;
    Ok(addr.chars().take(8).collect())
}

fn apply(state: StoredValue<AppState>, passphrase: &str) {
    // Derive the wallet code FIRST: if the secure element refuses (user
    // declined the seed prompt), nothing is switched.
    let code = match derive_code(state, passphrase) {
        Ok(c) => c,
        Err(e) => {
            set_error(state, &e);
            return;
        }
    };

    let hidden = !passphrase.is_empty();
    {
        let mut s = state.borrow_mut();
        s.passphrase = passphrase.to_string();
        // Hidden wallets get a fresh in-memory account list (no disk trace);
        // returning to the default wallet reloads the persisted list.
        s.accounts = if hidden { AccountStore::ephemeral() } else { AccountStore::load() };
        s.account = s.accounts.active;
    }

    let ui = state.borrow().ui();
    let pp = ui.global::<Passphrase>();
    log::info!(
        "passphrase wallet {}: code {}",
        if hidden { "OPENED" } else { "cleared -> default" },
        code
    );
    pp.set_active(hidden);
    pp.set_code(code.into());
    pp.set_error("".into());
    crate::create_account::refresh_rows(state);
    // Re-match the companion balance against the new wallet's account
    // fingerprints (fp-keyed, so a hidden wallet's entries match only while
    // that hidden wallet is open and its fingerprints are cached).
    crate::balance::render(state);
}

fn set_error(state: StoredValue<AppState>, msg: &str) {
    let ui = state.borrow().ui();
    ui.global::<Passphrase>().set_error(msg.into());
}
