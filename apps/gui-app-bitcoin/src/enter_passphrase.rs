// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::{
        state::AccountColor, state::AppState, store::CreateSingleSigAccount, EnterPassphrase,
        EnterPassphraseState, Navigate,
    },
    slint_keyos_platform::{slint::ComponentHandle, spawn_local, StoredValue},
};

pub fn init(state: StoredValue<AppState>) {
    let ui = state.borrow().ui();
    let global = ui.global::<EnterPassphrase>();

    global.on_try(move |passphrase| {
        try_passphrase(state, passphrase.into());
    });

    global.on_confirm(move |passphrase| {
        spawn_local(async move { confirm_passphrase(state, passphrase.into()).await }).detach();
    });

    global.on_clear_passphrase(move || {
        spawn_local(async move {
            AppState::apply_passphrase(state, String::new()).await;
        })
        .detach();
    });

    global.on_reapply(move |passphrase| {
        spawn_local(async move {
            AppState::apply_passphrase(state, passphrase.into()).await;
        })
        .detach();
    });

    // Handle local view switching (Default/Passphrase toggle) without notifying Envoy
    global.on_switch_view(move |index| {
        // Get the passphrase to use for this view
        let passphrase: String = {
            if index == 0 {
                String::new() // Default view: use empty passphrase
            } else {
                // Passphrase view: get stored passphrase from UI global
                let app_state = state.borrow();
                let ui = app_state.ui();
                ui.global::<EnterPassphrase>().get_passphrase().into()
            }
        }; // Borrow is released here
        AppState::switch_view_locally(state, passphrase);
    });

    global.on_create_initial_account(move |label| {
        spawn_local(async move { create_initial_account(state, label.into()).await }).detach();
    });
}

fn try_passphrase(state: StoredValue<AppState>, passphrase: String) {
    let app_state = state.borrow_mut();
    let ui = app_state.ui();
    let global = ui.global::<EnterPassphrase>();
    let fingerprint = app_state.store.try_passphrase(passphrase).unwrap_or_default();
    global.set_fingerprint(fingerprint.to_string().to_uppercase().into());
    global.set_no_accounts(app_state.store.num_single_accounts(Some(fingerprint)) == 0);
}

async fn confirm_passphrase(state: StoredValue<AppState>, passphrase: String) {
    AppState::apply_passphrase(state, passphrase).await;

    let app_state = state.borrow_mut();
    let ui = app_state.ui();
    let global = ui.global::<EnterPassphrase>();
    global.set_state(EnterPassphraseState::Clear);
    ui.global::<Navigate>().invoke_backward();
}

async fn create_initial_account(state: StoredValue<AppState>, label: String) {
    AppState::create_singlesig_account(
        state,
        CreateSingleSigAccount {
            label,
            color: AccountColor::LightCopper,
            network: ngwallet::bdk_wallet::bitcoin::Network::Bitcoin,
            index: 0,
        },
    )
    .await
    .inspect_err(|e| log::error!("failed to create single sig account {e:?}"))
    .ok();
}
