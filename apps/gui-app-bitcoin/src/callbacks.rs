// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::{
        account_id::AccountId, gui_permissions::GuiPermissions, psbt_signing::PsbtOrigin, state::AppState,
        tr, AddressType, Callbacks, KeychainKind, Navigate, PsbtOriginView, SignPsbt, SignPsbtState, TrId,
    },
    anyhow::Context,
    foundation_urtypes::value::Value as UrValue,
    slint_keyos_platform::{
        gui_server_api::{
            navigation::{
                filepicker::{self, SelectFileOptions},
                qrscanner::{ScanQrOptions, ScanQrResult},
            },
            GuiApiLight,
        },
        navigation::{open_qr_scanner, select_file},
        slint::{ComponentHandle, ModelRc, SharedString},
        spawn_local, StoredValue,
    },
    std::{cell::RefCell, io::Read, rc::Rc},
};

pub fn init_callbacks(state: StoredValue<AppState>) {
    let ui = state.borrow().ui();
    let callbacks = ui.global::<Callbacks>();

    callbacks.on_account_addresses(move |id, keychain_kind, address_type| {
        let account_id = match id.as_str().parse::<AccountId>() {
            Ok(id) => id,
            Err(_) => return Default::default(),
        };
        ModelRc::new(AddressModel {
            account_id,
            keychain_kind,
            address_type,
            state,
            cache: Default::default(),
        })
    });

    callbacks.on_select_file({
        move || {
            let state = state.clone();
            if let Err(e) = execute_file_picker_psbt(state) {
                log::error!("file picker failed: {e:?}");
                let ui = state.borrow().ui();
                let sign_psbt = ui.global::<SignPsbt>();
                sign_psbt.set_origin(PsbtOriginView::File);
                sign_psbt.set_state(SignPsbtState::Error);
                ui.global::<Navigate>().invoke_sign_psbt(Default::default());
            }
        }
    });

    // TODO: for now, this will only do PSBT signing, but we
    // could handle multisig configs here in the future
    callbacks.on_scan_clicked({
        move || {
            let state = state.clone();
            if let Err(e) = execute_scan(state, false) {
                log::error!("scan failed: {e:?}");
            }
        }
    });

    callbacks.on_account_details({
        move |id| state.borrow().get_account_view_str(&id).map(|(_id, acct)| acct).unwrap_or_default()
    });

    callbacks.on_update_account_name(move |id, name| {
        let id = match id.as_str().parse::<AccountId>() {
            Ok(id) => id,
            Err(_) => return,
        };
        AppState::update_account_config(state, id, |config| {
            config.name = name.to_string();
        });
    });

    callbacks.on_set_archive_mode_inner(move |mode| {
        AppState::set_archive_mode(state, mode);
    });

    callbacks.on_update_account_archived(move |id, archived| {
        let id = match id.as_str().parse::<AccountId>() {
            Ok(id) => id,
            Err(_) => return,
        };
        AppState::update_account_config(state, id, |config| {
            config.archived = archived;
        });
    });

    callbacks.on_delete_account(move |id| {
        let id = match id.as_str().parse::<AccountId>() {
            Ok(id) => id,
            Err(_) => return,
        };
        AppState::delete_account(state, id);
    });
}

pub fn execute_scan(state: StoredValue<AppState>, return_to_launcher_on_cancel: bool) -> anyhow::Result<()> {
    let opts = ScanQrOptions {
        header_title: tr::lookup_id(TrId::ScanTitle).into(),
        message: String::new(),
        header_left_icon: String::new(),
        header_right_icon: String::from("close"),
        ..ScanQrOptions::default()
    };

    let scan = match open_qr_scanner::<GuiPermissions>(opts) {
        Ok(Some(s)) => s,
        Ok(None) => {
            log::info!("Nothing returned from qr scanner");
            return Ok(());
        }
        Err(e) => {
            log::info!("Error while scanning QR: {:?}", e);
            return Ok(());
        }
    };

    match scan {
        ScanQrResult::Ur2(ur_type, data) => {
            let value = UrValue::from_ur(&ur_type, data.as_slice()).context("parse UrValue")?;
            match value {
                UrValue::Psbt(bytes) | UrValue::Bytes(bytes) => {
                    let fut = crate::psbt_signing::verify::verify_psbt(
                        state,
                        bytes.to_vec(),
                        PsbtOrigin::Qr { ur_type },
                        false,
                    );
                    spawn_local(fut).detach();
                }
                _ => {}
            }
        }
        ScanQrResult::RightClicked | ScanQrResult::LeftClicked => {
            if return_to_launcher_on_cancel {
                if let Err(e) = GuiApiLight::<GuiPermissions>::default().switch_to_launcher() {
                    log::error!("Failed to switch to launcher: {e:?}");
                }
            }
        }
        action @ _ => {
            log::error!("universal scan failed: {:?}", action);
            let ui = state.borrow().ui();
            let sign_psbt = ui.global::<SignPsbt>();
            sign_psbt.set_origin(PsbtOriginView::Qr);
            sign_psbt.set_state(SignPsbtState::Error);
            ui.global::<Navigate>().invoke_sign_psbt(Default::default());
        }
    }

    Ok(())
}

pub fn execute_file_picker(state: StoredValue<AppState>) -> anyhow::Result<Option<Vec<u8>>> {
    let options = SelectFileOptions::default().with_dirs_allowed(true);
    let files = match select_file::<GuiPermissions>(options) {
        Ok(Some(f)) => f,
        Ok(None) => {
            log::info!("Nothing returned from file picker");
            return Ok(None);
        }
        Err(e) => {
            log::info!("Error while picking file: {:?}", e);
            return Ok(None);
        }
    };

    let (path, location) = match files.files().len() {
        0 => {
            log::error!("No files selected");
            return Ok(None);
        }
        1 => files.files()[0].clone(),
        _ => {
            log::info!("Multiple files selected, using first only");
            files.files()[0].clone()
        }
    };

    let location = match location {
        filepicker::Location::Internal => fs::Location::User,
        filepicker::Location::Airlock => fs::Location::Airlock,
        filepicker::Location::External => fs::Location::Usb,
    };

    let mut opened = state
        .borrow()
        .store
        .fs
        .open_file(&path, location, fs::OpenFlags { read: true, write: false, create: false })
        .with_context(|| format!("Failed to open selected file {}", path))?;

    let mut bytes = Vec::new();
    let _ = opened.read_to_end(&mut bytes)?;

    Ok(Some(bytes))
}

pub fn execute_file_picker_psbt(state: StoredValue<AppState>) -> anyhow::Result<()> {
    let bytes = execute_file_picker(state)?;

    if let Some(b) = bytes {
        let fut = crate::psbt_signing::verify::verify_psbt(state, b, PsbtOrigin::File, false);
        spawn_local(fut).detach();
    }

    Ok(())
}

struct AddressModel {
    account_id: AccountId,
    keychain_kind: KeychainKind,
    address_type: AddressType,
    state: StoredValue<AppState>,
    cache: Rc<RefCell<AddressCache>>,
}

#[derive(Default)]
struct AddressCache {
    addresses: Vec<String>,
}

const MAX_ADDRESS_COUNT: usize = 1000;

// TODO: once we have support of lazy loading, we can improve this implementation
// right now we limit the number of addresses fetched to 100
impl slint_keyos_platform::slint::Model for AddressModel {
    type Data = SharedString;

    fn row_count(&self) -> usize { MAX_ADDRESS_COUNT }

    fn row_data(&self, row: usize) -> Option<Self::Data> {
        const WINDOW_SIZE: usize = 50;

        if row >= MAX_ADDRESS_COUNT {
            return None;
        }

        let mut cache = self.cache.borrow_mut();

        if row < cache.addresses.len() {
            return Some(SharedString::from(&cache.addresses[row]));
        }

        // fetch next 50 addresses
        let addresses = self
            .state
            .borrow_mut()
            .get_account_addresses(
                self.account_id.clone(),
                self.keychain_kind.into(),
                self.address_type.into(),
                Some(cache.addresses.len() as u32),
                WINDOW_SIZE,
            )
            .ok()?;

        cache.addresses.extend(addresses);

        Some(SharedString::from(&cache.addresses[row]))
    }

    fn model_tracker(&self) -> &dyn slint_keyos_platform::slint::ModelTracker { &() }
}
