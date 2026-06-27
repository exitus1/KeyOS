use slint_keyos_platform::slint::ComponentHandle;
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Seed *display* (view / back up) screen. This app does NOT create or restore
// seeds — that is the OS Seed Vault's job (see create_account.rs). This screen
// only displays the seed already in the secure element as 24 words or a SeedQR.
// Reading the seed passes the OS user-confirmation gate. Encoders mirror
// gui-app-onboarding/src/seed.rs so output is byte-identical to the OS backup.

use anyhow::{anyhow, Result};
use slint_keyos_platform::slint;
use slint_keyos_platform::StoredValue;

use crate::state::AppState;
use crate::SeedDisplay;

pub fn init(state: StoredValue<AppState>) {
    let ui = state.borrow().ui();
    let sd = ui.global::<SeedDisplay>();

    sd.on_get_seed_words(move || {
        get_seed_words(state).unwrap_or_else(|e| {
            log::error!("get_seed_words failed: {e:?}");
            empty_words()
        })
    });

    sd.on_get_standard_seed_qr(move || {
        get_standard_seed_qr(state).unwrap_or_else(|e| {
            log::error!("standard seed qr failed: {e:?}");
            slint::Image::default()
        })
    });

    sd.on_get_compact_seed_qr(move || {
        get_compact_seed_qr(state).unwrap_or_else(|e| {
            log::error!("compact seed qr failed: {e:?}");
            slint::Image::default()
        })
    });
}

fn get_seed_words(state: StoredValue<AppState>) -> Result<slint::ModelRc<slint::SharedString>> {
    let seed = read_seed(state)?;
    let mnemonic = bip39::Mnemonic::from_entropy(seed.bytes())
        .map_err(|e| anyhow!("mnemonic from entropy: {e}"))?;
    let words: Vec<slint::SharedString> =
        mnemonic.words().map(slint::SharedString::from).collect();
    Ok(slint::ModelRc::new(slint::VecModel::from(words)))
}

fn get_standard_seed_qr(state: StoredValue<AppState>) -> Result<slint::Image> {
    let seed = read_seed(state)?;
    let mnemonic = bip39::Mnemonic::from_entropy(seed.bytes())
        .map_err(|e| anyhow!("mnemonic from entropy: {e}"))?;
    let indices: String = mnemonic.word_indices().map(|idx| format!("{idx:04}")).collect();
    Ok(render_qr(indices.as_bytes()))
}

fn get_compact_seed_qr(state: StoredValue<AppState>) -> Result<slint::Image> {
    let seed = read_seed(state)?;
    let mnemonic = bip39::Mnemonic::from_entropy(seed.bytes())
        .map_err(|e| anyhow!("mnemonic from entropy: {e}"))?;
    let entropy = mnemonic.to_entropy();
    Ok(render_qr(&entropy))
}

fn read_seed(state: StoredValue<AppState>) -> Result<security::Seed> {
    let s = state.borrow();
    s.security
        .seed()
        .map_err(|_| anyhow!("seed access denied"))?
        .ok_or_else(|| anyhow!("no seed on device"))
}

fn render_qr(data: &[u8]) -> slint::Image {
    slint_keyos_platform::qrcode::render(
        data,
        slint::Color::from_rgb_u8(0, 0, 0),
        slint::Color::from_rgb_u8(255, 255, 255),
    )
}

fn empty_words() -> slint::ModelRc<slint::SharedString> {
    slint::ModelRc::new(slint::VecModel::from(Vec::<slint::SharedString>::new()))
}
