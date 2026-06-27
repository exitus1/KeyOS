// SPDX-License-Identifier: GPL-3.0-or-later
//
// Decred Wallet entry point. Mirrors gui-app-bitcoin/src/main.rs structure:
// pull in the security API, declare the app via the `app!` macro, build a
// StoredValue<AppState>, and wire each feature module's init().
//
// Scope (deliberately small): receive addresses, view accounts, and SIGN an
// unsigned-tx package that Cake Wallet built. No SPV, no broadcast, no staking,
// no mixing. Transport for signing is QR (animated UR) or SD card.

#![feature(must_not_suspend)]
#![deny(must_not_suspend)]

use slint_keyos_platform::{app, gui_server_api::InputMessage, StoredValue};

mod create_account;
mod keys;
mod receive;
mod sign_tx;
mod state;

use state::AppState;

// Brings `crate::Security` + the GetSeed/GetDeviceId message-allowed wiring
// into scope, exactly as the Bitcoin app does.
security::use_api!();

app!("Decred Wallet");
fn app_main(cx: AppContext, ui: AppWindow) {
    log_server::init_wait(env!("CARGO_CRATE_NAME")).unwrap();
    log::set_max_level(log::LevelFilter::Info);

    cx.config.enable_swipe_back.set(false);

    let state = StoredValue::new(AppState::new(ui.as_weak()));

    // Feature wiring. Each module installs its Slint callbacks against `state`.
    receive::init(state);
    sign_tx::init(state);
    create_account::init(state);

    // Handle deep-link navigation (e.g. "open Decred and start a QR scan").
    cx.set_input_handler({
        move |input| {
            if input.msg == InputMessage::NavigationFocused {
                // The Decred app currently exposes a single deep-linked entry:
                // jump straight into the sign-tx scanner. Kept intentionally
                // simpler than the Bitcoin app's multi-action router.
                if let Err(e) = sign_tx::begin_scan(state) {
                    log::error!("failed to begin scan: {e:?}");
                }
            }
        }
    });

    ui.run().expect("UI running");
}
