// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::{
        error::ToValidationString,
        seed::{Seed, SeedType},
        state::AppState,
        CallbackResult, Callbacks, SeedView, SeedViewType,
    },
    ordered_table::CardSortMode,
    slint_keyos_platform::{
        slint::{ComponentHandle, Image, ModelRc, SharedString, VecModel},
        StoredValue,
    },
};

pub fn init_callbacks(state: StoredValue<AppState>) {
    let ui = state.borrow().ui();
    let callbacks = ui.global::<Callbacks>();

    callbacks.on_validate_new_label({
        move |label: SharedString| {
            let app_state = state.borrow();
            if let Err(e) = app_state.validate_new_label(label.to_string()) {
                return e.to_validation_string().into();
            }

            SharedString::new()
        }
    });

    callbacks.on_validate_new_index({
        move |seed_index: SharedString, view_type: SeedViewType| {
            let app_state = state.borrow();

            let seed_type = match SeedType::from_view_type(view_type, Some(seed_index), None, None) {
                Ok(st) => st,
                Err(e) => return e.to_validation_string().into(),
            };

            if let Err(e) = app_state.validate_new_index(seed_type) {
                return e.to_validation_string().into();
            }

            SharedString::new()
        }
    });

    callbacks.on_save({
        move |seed_view: SeedView| {
            let mut app_state = state.borrow_mut();

            let seed = match Seed::from_view(seed_view) {
                Ok(s) => s,
                Err(e) => return CallbackResult::from(e),
            };

            if let Err(e) = app_state.save(seed) {
                return CallbackResult::from(e);
            }

            app_state.update_accounts();
            app_state.nav_accounts();
            CallbackResult::success()
        }
    });

    callbacks.on_set_archive_mode({
        move |archive_mode| {
            let mut app_state = state.borrow_mut();
            app_state.archive_mode = archive_mode;
            app_state.update_accounts();
        }
    });

    callbacks.on_set_sort_mode({
        move |sort_mode| {
            let mut app_state = state.borrow_mut();
            app_state.set_sort_mode(CardSortMode::from(sort_mode as usize));
            app_state.update_accounts();
        }
    });

    callbacks.on_move_position({
        move |index, up| {
            let mut app_state = state.borrow_mut();
            // Ignores errors, nothing happens
            let _ = app_state.move_position(index, up);
            app_state.update_accounts();
        }
    });

    callbacks.on_search({
        move |text| {
            let mut app_state = state.borrow_mut();
            app_state.search_text = text.to_string().to_lowercase();
            app_state.update_accounts();
        }
    });

    callbacks.on_get_next_index_string({
        move |seed_type: SeedViewType| {
            let app_state = state.borrow();
            format!("{}", app_state.get_next_index(seed_type)).into()
        }
    });

    callbacks.on_validate_edit_label({
        move |index, new_label| {
            let mut app_state = state.borrow_mut();
            if let Err(e) = app_state.validate_edit_label(index, new_label.into()) {
                return e.to_validation_string().into();
            }

            SharedString::new()
        }
    });

    callbacks.on_edit_indexed({
        move |index, new_label, new_color| {
            let mut app_state = state.borrow_mut();

            if let Err(e) = app_state.edit_indexed(index, new_label.into(), new_color as u8) {
                return CallbackResult::from(e);
            }

            app_state.update_accounts();
            CallbackResult::success()
        }
    });

    callbacks.on_edit_password({
        move |index, new_label, new_account, new_password, new_color| {
            let mut app_state = state.borrow_mut();

            if let Err(e) = app_state.edit_password(
                index,
                new_label.into(),
                new_account.into(),
                new_password.into(),
                new_color as u8,
            ) {
                return CallbackResult::from(e);
            }

            app_state.update_accounts();
            CallbackResult::success()
        }
    });

    callbacks.on_set_archived({
        move |index, archived| {
            let mut app_state = state.borrow_mut();

            if let Err(e) = app_state.set_archived(index, archived) {
                log::warn!("{}", e);
                return;
            }

            app_state.update_accounts();
        }
    });

    callbacks.on_delete({
        move |index| {
            let mut app_state = state.borrow_mut();

            if let Err(e) = app_state.delete(index) {
                log::warn!("{}", e);
                return;
            }

            app_state.update_accounts();
        }
    });

    callbacks.on_get_words({
        move |index| {
            let app_state = state.borrow();

            let words = app_state
                .get_words(index)
                .unwrap_or_else(|e| {
                    log::warn!("Could not get seed words: {:?}", e);
                    String::new()
                })
                .split(' ')
                .map(SharedString::from)
                .collect::<Vec<SharedString>>();

            ModelRc::new(VecModel::from(words))
        }
    });

    callbacks.on_get_standard_seed_qr({
        move |index| {
            let app_state = state.borrow();

            app_state.get_standard_seed_qr(index).unwrap_or_else(|e| {
                log::warn!("Could not get seed qr: {:?}", e);
                Image::default()
            })
        }
    });

    callbacks.on_get_compact_seed_qr({
        move |index| {
            let app_state = state.borrow();

            app_state.get_compact_seed_qr(index).unwrap_or_else(|e| {
                log::warn!("Could not get compact seed qr: {:?}", e);
                Image::default()
            })
        }
    });

    callbacks.on_get_npub_qr({
        move |index| {
            let app_state = state.borrow();

            app_state.get_npub_qr(index).unwrap_or_else(|e| {
                log::warn!("Could not get npub qr: {:?}", e);
                Image::default()
            })
        }
    });

    callbacks.on_get_nsec_qr({
        move |index| {
            let app_state = state.borrow();

            app_state.get_nsec_qr(index).unwrap_or_else(|e| {
                log::warn!("Could not get nsec qr: {:?}", e);
                Image::default()
            })
        }
    });

    callbacks.on_get_npub({
        move |index| {
            let app_state = state.borrow();

            app_state.get_npub(index).map(SharedString::from).unwrap_or_else(|e| {
                log::warn!("Could not get npub: {:?}", e);
                SharedString::new()
            })
        }
    });

    callbacks.on_get_nsec({
        move |index| {
            let app_state = state.borrow();

            app_state.get_nsec(index).map(SharedString::from).unwrap_or_else(|e| {
                log::warn!("Could not get nsec: {:?}", e);
                SharedString::new()
            })
        }
    });
}
