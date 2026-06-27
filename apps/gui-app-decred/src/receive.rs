use slint_keyos_platform::slint::ComponentHandle;
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Receive screen: derive a fresh external-branch P2PKH address
// (m/44'/42'/account'/0/index) from the secure element and show it as text +
// QR for a sender (or Cake Wallet's "receive from cold storage" flow) to use.
//
// Deriving a receive address requires reading the seed, so this also passes
// through the user-confirmation gate. We expose the public address only; no
// private material leaves this function.

use anyhow::{anyhow, Result};
use slint_keyos_platform::StoredValue;

use crate::keys::{load_master_key, receive_address};
use crate::state::AppState;
use crate::{Receive, ReceiveState};

pub fn init(state: StoredValue<AppState>) {
    let ui = state.borrow().ui();
    let recv = ui.global::<Receive>();

    recv.on_show_address({
        move |index| {
            match derive_for_display(state, index as u32) {
                Ok(addr) => {
                    let ui = state.borrow().ui();
                    let recv = ui.global::<Receive>();
                    recv.set_address(addr.into());
                    recv.set_index(index);
                    recv.set_state(ReceiveState::Shown);
                }
                Err(e) => {
                    log::error!("receive derive failed: {e:?}");
                    let ui = state.borrow().ui();
                    ui.global::<Receive>().set_error_text(e.to_string().into());
                    ui.global::<Receive>().set_state(ReceiveState::Error);
                }
            }
        }
    });
}

fn derive_for_display(state: StoredValue<AppState>, index: u32) -> Result<String> {
    let s = state.borrow();
    let master = load_master_key(&s.secp, &s.security, "").map_err(|e| anyhow!("{e}"))?;
    let addr = receive_address(&s.secp, &master, s.account, index).map_err(|e| anyhow!("{e}"))?;
    Ok(addr)
}
