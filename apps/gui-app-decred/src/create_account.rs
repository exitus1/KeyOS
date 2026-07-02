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
            .map(|a| AccountRow {
                idx: a.index as i32,
                name: a.name.as_str().into(),
                path: format!("m/44'/42'/{}'", a.index).into(),
                active: a.index == active,
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
                Ok(dpub) => {
                    let ui = state.borrow().ui();
                    let acct = ui.global::<Account>();
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
            log::info!("active account -> {}", idx);
        }
    });
}

/// Derive the account key at m/44'/42'/index' and return its neutered dpub.
fn export_dpub(state: StoredValue<AppState>, index: u32) -> Result<String> {
    let s = state.borrow();
    let master = load_master_key(&s.secp, &s.security, &s.passphrase).map_err(|e| anyhow!("{e}"))?;
    let account = master.account_key(&s.secp, index).map_err(|e| anyhow!("{e}"))?;
    // Export the NEUTERED account key (dpub) — public only, safe to hand to a
    // watch-only Cake Wallet. Never exports private material.
    Ok(account.to_dpub(&s.secp))
}
