// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use {
    anyhow::{bail, Context},
    foundation_urtypes::value::Value as UrValue,
    slint_keyos_platform::{
        gui_server_api::navigation::qrscanner::{ScanQrOptions, ScanQrResult},
        navigation::open_qr_scanner,
        slint::{ComponentHandle, ToSharedString},
        spawn_local, StoredValue,
    },
    std::{thread, time::Duration},
};

use crate::{
    gui_permissions::GuiPermissions,
    psbt_signing::{verify::verify_psbt, PendingPsbt},
    state::{AccountColor, AppState},
    store::{CreateMultiSigAccount, CreateSingleSigAccount},
    tr, Animate, CreateAccount, CreateAccountState, CreateMultiSigOptions, CreateSingleSigOptions,
    MultiSigView, Navigate, NavigateOptions, RouteOption, RouteState, TrId,
};

pub fn init(state: StoredValue<AppState>) {
    let ui = state.borrow().ui();
    let global = ui.global::<CreateAccount>();

    global.on_import_multisig(move || {
        if let Err(e) = import_multisig(state) {
            let ui = state.borrow().ui();
            ui.global::<CreateAccount>().set_state(CreateAccountState::Error);
            ui.global::<Navigate>().invoke_import_multi_sig(Default::default());
            log::error!("failed to import multisig {e:?}");
        }
    });

    global.on_create_single_sig(move |options| {
        spawn_local(async move {
            create_single_sig(state, options).await;
        })
        .detach();
    });

    global.on_create_multi_sig(move |options| {
        spawn_local(async move {
            create_multisig(state, options).await;
        })
        .detach();
    });

    global.on_validate_new_label({
        move |label| state.borrow().store.validate_label(&label).unwrap_or_default().into()
    });

    global.on_validate_new_index({
        move |index, network| {
            state
                .borrow()
                .store
                .validate_index(index.trim().parse::<u32>().unwrap_or(0), network.into())
                .unwrap_or_default()
                .into()
        }
    });

    global.on_get_next_index(move |network| {
        let index_num = state.borrow().store.get_next_index(network.into());
        format!("{}", index_num).into()
    });

    global.on_reset_prefilled_mode(move || {
        let ui = state.borrow().ui();
        let global = ui.global::<CreateAccount>();
        global.set_prefilled_mode(false);
        state.borrow_mut().pending_singlesig = None;
    });
}

async fn create_multisig(state: StoredValue<AppState>, options: CreateMultiSigOptions) {
    let Some(multisig) = state.borrow().pending_multisig.clone() else {
        log::error!("no pending multisig found");
        return;
    };

    let ui = state.borrow().ui();
    let global = ui.global::<CreateAccount>();

    global.set_state(CreateAccountState::Creating);

    let create = CreateMultiSigAccount {
        label: options.label.into(),
        color: AccountColor::Pine,
        network: options.network.into(),
        multisig,
    };
    let result = AppState::create_multisig_account(state, create).await;

    match result {
        Ok(account_id) => {
            global.set_state(CreateAccountState::Success);
            global.set_new_account_id(account_id.to_shared_string());
            state.borrow_mut().pending_multisig = None;

            if let PendingPsbt::NotSaved { psbt, origin } =
                std::mem::take(&mut state.borrow_mut().pending_psbt)
            {
                global.set_state(CreateAccountState::Idle);
                let bytes = psbt.serialize();
                spawn_local(async move {
                    verify_psbt(state, bytes, origin, true).await;
                })
                .detach();
            }
        }
        Err(e) => {
            log::error!("Failed to create single sig account: {e:?}");
            global.set_state(CreateAccountState::Error);
        }
    }
}

async fn create_single_sig(state: StoredValue<AppState>, options: CreateSingleSigOptions) {
    let ui = state.borrow().ui();
    let global = ui.global::<CreateAccount>();

    global.set_state(CreateAccountState::Creating);

    let create = CreateSingleSigAccount {
        label: options.label.to_string(),
        network: options.network.into(),
        index: options.index.trim().parse::<u32>().unwrap_or(0),
        color: AccountColor::from(options.color),
    };

    global.set_prefilled_mode(false);

    let result = AppState::create_singlesig_account(state, create).await;

    match result {
        Ok(account_id) => {
            global.set_state(CreateAccountState::Success);
            global.set_new_account_id(account_id.to_shared_string());
            state.borrow_mut().pending_singlesig = None;

            if let PendingPsbt::NotSaved { psbt, origin } =
                std::mem::take(&mut state.borrow_mut().pending_psbt)
            {
                global.set_state(CreateAccountState::Idle);
                let bytes = psbt.serialize();
                spawn_local(async move {
                    verify_psbt(state, bytes, origin, true).await;
                })
                .detach();
            }
        }
        Err(e) => {
            log::error!("Failed to create single sig account: {e:?}");
            global.set_state(CreateAccountState::Error);
        }
    }
}

fn import_multisig(state: StoredValue<AppState>) -> anyhow::Result<()> {
    let opts = ScanQrOptions {
        header_title: tr::lookup_id(TrId::ImportConfigTitle).into(),
        message: String::new(),
        header_left_icon: String::new(),
        header_right_icon: String::from("close"),
        button_icon: String::from("file"),
        button_text: tr::lookup_id(TrId::ImportConfigImportFile).into(),
        ..Default::default()
    };

    let scan = match open_qr_scanner::<GuiPermissions>(opts) {
        Ok(Some(s)) => s,
        Ok(None) => {
            log::info!("qr scanner returned no data");
            return Ok(());
        }
        Err(e) => {
            log::info!("failed to open qr scanner: {e:?}");
            return Ok(());
        }
    };

    let string = match scan {
        ScanQrResult::Qr(data) => String::from_utf8(data).context("invalid qr utf8")?,
        ScanQrResult::Ur2(ur_type, data) => {
            let ur_value = UrValue::from_ur(&ur_type, data.as_slice()).context("invalid UR value")?;
            let bytes = match ur_value {
                UrValue::Bytes(bytes) => bytes,
                other => {
                    bail!("non-bytes UrValue {other:?}")
                }
            };
            String::from_utf8(bytes.to_vec()).context("invalid UR2 multisig utf8")?
        }
        ScanQrResult::ButtonClicked => {
            // sleep to avoid the OOM killer closing the bitcoin app
            thread::sleep(Duration::from_millis(500));
            let bytes = crate::callbacks::execute_file_picker(state)?;

            match bytes {
                Some(b) => String::from_utf8(b).context("invalid file utf8")?,
                None => return Ok(()),
            }
        }
        ScanQrResult::RightClicked => return Ok(()),
        action => {
            bail!("unexpected scan result: {:?}", action);
        }
    };

    let ui = state.borrow().ui();

    let multisig_view = state.borrow_mut().parse_multisig(&string).map(MultiSigView::from)?;

    let global = ui.global::<CreateAccount>();
    global.set_state(CreateAccountState::Idle);
    global.set_pending_multisig_account(multisig_view);

    if ui.global::<RouteState>().get_active() != RouteOption::ImportMultiSig {
        ui.global::<Navigate>()
            .invoke_import_multi_sig(NavigateOptions { replace: false, animate: Animate::None });
    }

    Ok(())
}
