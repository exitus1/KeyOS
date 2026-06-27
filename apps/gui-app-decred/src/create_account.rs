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
use crate::{Account, AccountState};

pub fn init(state: StoredValue<AppState>) {
    let ui = state.borrow().ui();
    let acct = ui.global::<Account>();

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
}

/// Derive the account key at m/44'/42'/index' and return its neutered dpub.
fn export_dpub(state: StoredValue<AppState>, index: u32) -> Result<String> {
    let s = state.borrow();
    let master = load_master_key(&s.secp, &s.security, "").map_err(|e| anyhow!("{e}"))?;
    let account = master.account_key(&s.secp, index).map_err(|e| anyhow!("{e}"))?;
    // Export the NEUTERED account key (dpub) — public only, safe to hand to a
    // watch-only Cake Wallet. Never exports private material.
    Ok(account.to_dpub(&s.secp))
}
