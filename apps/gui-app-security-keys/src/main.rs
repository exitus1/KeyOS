// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use haptics::HapticPattern;
#[cfg(not(test))]
use slint_keyos_platform::file_backed::JsonBacked;
use {
    fido::error::FidoError,
    fuzzy_filter::FuzzyFilter,
    gui_app_security_keys::{Key, KeyDuplicateReason, KeyEditField, KeyValidationError, DATABASE_FILE},
    ordered_table::{CardSortMode, FilePersistence, OrderedTable, OrderedTableError, SortableCard},
    slint_keyos_platform::{
        app,
        gui_server_api::{
            navigation::securitykeys::{OperationType, SecurityKeysNavRequest, UserPresenceResult},
            InputMessage,
        },
        sleep,
        slint::{Model, ModelRc, SharedString, VecModel},
        spawn_local, StoredValue,
    },
    std::{rc::Rc, time::Duration},
};

use crate::fs_permissions::FileSystemPermissions;

fido::use_api!();
haptics::use_api!();
nfc::use_api!();
#[cfg(keyos)]
usb::use_device_api!();

#[derive(Debug, thiserror::Error)]
pub enum KeyError {
    #[error("OrderedTableError: {0:?}")]
    OrderedTableError(OrderedTableError<Key>),
    #[error("ValidationError: {0:?}")]
    ValidationError(KeyValidationError),
    #[error("DuplicateError: {0:?}")]
    DuplicateError(KeyDuplicateReason),
    #[error("Could not use negative index")]
    IndexError,
    #[error("Could not move key to {0:?}, only {1:?} non-archived")]
    MovePositionError(usize, usize),
    #[error("Code {0:?} is already archived")]
    RedundantArchivalError(usize),
    #[error("Fido error: {0:?}")]
    FidoError(FidoError),
}

impl From<OrderedTableError<Key>> for KeyError {
    fn from(value: OrderedTableError<Key>) -> Self { KeyError::OrderedTableError(value) }
}

impl From<FidoError> for KeyError {
    fn from(value: FidoError) -> Self { KeyError::FidoError(value) }
}

impl From<KeyValidationError> for KeyError {
    fn from(value: KeyValidationError) -> Self { KeyError::ValidationError(value) }
}

impl From<KeyDuplicateReason> for KeyError {
    fn from(value: KeyDuplicateReason) -> Self { KeyError::DuplicateError(value) }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct KeySettings {
    sort_mode: CardSortMode,
}

impl Default for KeySettings {
    fn default() -> Self { Self { sort_mode: CardSortMode::Label } }
}

struct AppState {
    key_table: OrderedTable<Key, FilePersistence<FileSystemPermissions>>,
    search_text: String,
    archive_mode: bool,
    model: Rc<VecModel<KeyView>>,
    #[cfg(not(test))]
    settings: JsonBacked<KeySettings, FileSystemPermissions>,
    #[cfg(test)]
    sort_mode: CardSortMode,
    fido_api: FidoApi,
    haptics_api: HapticsApi,
    nfc_api: NfcApi,
    #[cfg(keyos)]
    usb_device: UsbDeviceEmulation,
    dropdown_model: Rc<VecModel<DropdownModel>>,
}

impl KeyView {
    fn new(value: &Key, live: bool) -> Self {
        Self {
            label: SharedString::from(value.get_label()),
            color: value.color as i32,
            live,
            icon: SharedString::from(&value.icon),
            index: -1,
        }
    }

    fn with_index(mut self, index: i32) -> Self {
        self.index = index;
        self
    }
}

impl AppState {
    fn get_key_entries(&self) -> ModelRc<KeyView> {
        self.model.clear();

        let filter = if self.search_text.is_empty() {
            None
        } else {
            Some(FuzzyFilter::new(self.search_text.as_ref()))
        };

        let entries = self
            .key_table
            .view_sorted(|a, b| Key::compare_by(a, b, self.get_sort_mode()))
            .filter(|(_i, entry)| {
                if entry.archived != self.archive_mode {
                    return false;
                }

                match &filter {
                    Some(filter) if !filter.matches(entry.get_label().to_lowercase().as_ref()) => false,
                    _ => true,
                }
            })
            .map(|(i, entry)| {
                let is_live = self.fido_api.is_live(entry.get_index()).unwrap_or_else(|e| {
                    log::warn!("Could not get live status for {}: {}", entry.get_label(), e);
                    false
                });

                KeyView::new(entry, is_live).with_index(i as i32)
            })
            .collect::<Vec<KeyView>>();

        self.model.extend(entries);
        ModelRc::from(self.model.clone())
    }

    fn get_sort_mode(&self) -> CardSortMode {
        #[cfg(not(test))]
        return self.settings.sort_mode.clone();
        #[cfg(test)]
        return self.sort_mode;
    }

    #[cfg(not(test))]
    fn set_sort_mode(&mut self, mode: CardSortMode) { self.settings.guard().sort_mode = mode; }

    #[cfg(test)]
    fn set_sort_mode(&mut self, mode: CardSortMode) { self.sort_mode = mode; }

    fn get_dropdown_model(&self) -> ModelRc<DropdownModel> {
        self.dropdown_model.clear();

        let entries = self
            .key_table
            .view_sorted(|a, b| Key::compare_by(a, b, self.get_sort_mode()))
            .filter(|(_i, entry)| {
                if entry.archived != self.archive_mode {
                    return false;
                }

                true
            })
            .map(|(i, entry)| DropdownModel {
                label: entry.get_label().into(),
                value: i.to_string().into(),
                icon: SharedString::from("key"),
            })
            .collect::<Vec<DropdownModel>>();

        self.dropdown_model.extend(entries);
        ModelRc::from(self.dropdown_model.clone())
    }
}

trait ToValidationString {
    fn to_validation_string(&self) -> SharedString;
}

impl ToValidationString for KeyError {
    fn to_validation_string(&self) -> SharedString {
        match self {
            KeyError::OrderedTableError(e) => e.to_validation_string(),
            KeyError::ValidationError(e) => e.to_validation_string(),
            KeyError::DuplicateError(e) => e.to_validation_string(),
            ref other => other.to_string().into(),
        }
    }
}

impl ToValidationString for OrderedTableError<Key> {
    fn to_validation_string(&self) -> SharedString {
        match self {
            OrderedTableError::PushInvalidError(validation_error) => validation_error.to_validation_string(),
            OrderedTableError::PushDuplicateError((duplicate_reason, _i)) => {
                duplicate_reason.to_validation_string()
            }
            OrderedTableError::EditInvalidOperationError(validation_error) => {
                validation_error.to_validation_string()
            }
            OrderedTableError::EditInvalidResultError(validation_error) => {
                validation_error.to_validation_string()
            }
            OrderedTableError::EditDuplicateError((duplicate_reason, _i)) => {
                duplicate_reason.to_validation_string()
            }
            ref other => {
                log::warn!("{}", self);
                other.to_string().into()
            }
        }
    }
}

impl ToValidationString for KeyValidationError {
    fn to_validation_string(&self) -> SharedString {
        match self {
            KeyValidationError::InvalidLabelError => SharedString::from(tr::lookup_id(TrId::AddLabelMissing)),
        }
    }
}

impl ToValidationString for KeyDuplicateReason {
    fn to_validation_string(&self) -> SharedString {
        match self {
            KeyDuplicateReason::Label(_other) => {
                SharedString::from(tr::lookup_id(TrId::AddLabelAlreadyInUse))
            }
            ref other => {
                log::warn!("{}", self);
                other.to_string().into()
            }
        }
    }
}

impl From<KeyError> for CallbackResult {
    fn from(error: KeyError) -> Self {
        log::warn!("{}", error);
        match error {
            KeyError::OrderedTableError(e) => Self::from(e),
            KeyError::ValidationError(e) => Self::from(e),
            KeyError::DuplicateError(reason) => Self::from(reason),
            // Other KeyErrors should never be seen because they
            // only result from unexpected behavior like system errors
            ref other => Self::failure(ResultLevel::Error, String::from("Error"), other.to_string()),
        }
    }
}

impl From<OrderedTableError<Key>> for CallbackResult {
    fn from(error: OrderedTableError<Key>) -> Self {
        log::warn!("{}", error);
        match error {
            OrderedTableError::PushInvalidError(validation_error) => CallbackResult::from(validation_error),
            OrderedTableError::PushDuplicateError((duplicate_reason, _i)) => {
                CallbackResult::from(duplicate_reason)
            }
            OrderedTableError::EditInvalidOperationError(validation_error) => {
                CallbackResult::from(validation_error)
            }
            OrderedTableError::EditInvalidResultError(validation_error) => {
                CallbackResult::from(validation_error)
            }
            OrderedTableError::EditDuplicateError((duplicate_reason, _i)) => {
                CallbackResult::from(duplicate_reason)
            }
            ref other => Self::failure(ResultLevel::Error, String::from("Error"), other.to_string()),
        }
    }
}

impl From<KeyValidationError> for CallbackResult {
    fn from(error: KeyValidationError) -> Self {
        log::warn!("{}", error);
        match error {
            // Other ValidationErrors should never be seen because save buttons
            // are disabled in case of these validation errors.
            ref other => Self::failure(
                ResultLevel::Error,
                String::from("Error"),
                other.to_validation_string().to_string(),
            ),
        }
    }
}

impl From<KeyDuplicateReason> for CallbackResult {
    fn from(reason: KeyDuplicateReason) -> Self {
        log::warn!("{}", reason);
        match reason {
            // Other DuplicateReasons should never be seen because save buttons
            // are disabled in case of these validation errors.
            ref other => Self::failure(
                ResultLevel::Error,
                String::from("Error"),
                other.to_validation_string().to_string(),
            ),
        }
    }
}

impl CallbackResult {
    fn success() -> Self {
        Self {
            success: true,
            level: ResultLevel::Info,
            title: SharedString::new(),
            text: SharedString::new(),
        }
    }

    fn failure(level: ResultLevel, title: String, text: String) -> Self {
        Self { success: false, level, title: SharedString::from(title), text: SharedString::from(text) }
    }
}

app!("Keys");

fn app_main(cx: AppContext, ui: AppWindow) {
    log_server::init_wait(env!("CARGO_CRATE_NAME")).unwrap();
    log::set_max_level(log::LevelFilter::Info);

    cx.config.enable_swipe_back.set(false);

    // All errors encountered here are unrecoverable.
    let app_state = AppState {
        key_table: OrderedTable::new()
            .with_persistence(FilePersistence::new(String::from(DATABASE_FILE), fs::Location::AppData))
            .expect("failed to create security key database"),
        search_text: String::new(),
        archive_mode: false,
        model: Rc::new(VecModel::default()),
        #[cfg(not(test))]
        settings: JsonBacked::new("settings.json", fs::Location::AppData).0,
        #[cfg(test)]
        sort_mode: CardSortMode::Label,
        fido_api: FidoApi::default(),
        haptics_api: HapticsApi::default(),
        nfc_api: NfcApi::default(),
        #[cfg(keyos)]
        usb_device: UsbDeviceEmulation::default(),
        dropdown_model: Rc::new(VecModel::default()),
    };

    if app_state.key_table.len() == 0 {
        ui.global::<Navigate>().invoke_add(NavigateOptions { replace: true, animate: Animate::None });
    }

    let ui_state = ui.global::<SecurityKeyCallbacks>();
    ui_state.set_entries(app_state.get_key_entries());
    ui_state.set_dropdown_model(app_state.get_dropdown_model());
    ui_state.set_sort_mode(app_state.get_sort_mode() as i32);

    let app_state = StoredValue::new(app_state);

    ui.global::<SecurityKeyCallbacks>().on_search({
        let ui = ui.clone_strong();
        move |text| {
            let mut app_state = app_state.borrow_mut();
            let ui_state = ui.global::<SecurityKeyCallbacks>();
            app_state.search_text = text.to_string().to_lowercase();
            ui_state.set_entries(app_state.get_key_entries());
            ui_state.set_dropdown_model(app_state.get_dropdown_model());
        }
    });

    ui.global::<SecurityKeyCallbacks>().on_save({
        let ui = ui.clone_strong();
        move |label, icon, _live, color| {
            let mut app_state = app_state.borrow_mut();
            let ui_state = ui.global::<SecurityKeyCallbacks>();

            // for MVP, all keys are live by default
            if let Err(e) = adapt_save(label, icon, true, color, &mut app_state) {
                return CallbackResult::from(e);
            };

            ui_state.set_entries(app_state.get_key_entries());
            ui_state.set_dropdown_model(app_state.get_dropdown_model());

            CallbackResult::success()
        }
    });

    ui.global::<SecurityKeyCallbacks>().on_validate_new_label({
        move |label| {
            let mut app_state = app_state.borrow_mut();

            if let Err(e) = adapt_validate_new_label(label, None, None, None, &mut app_state) {
                return e.to_validation_string();
            };

            SharedString::new()
        }
    });

    ui.global::<SecurityKeyCallbacks>().on_validate_edit_label({
        move |index, label| {
            let app_state = app_state.borrow_mut();

            let Ok(index) = usize::try_from(index) else {
                return KeyError::IndexError.to_validation_string();
            };

            if let Err(e) = app_state
                .key_table
                .validate_edit(index, move |a| a.edit(KeyEditField::Label(label.clone().into())))
            {
                return e.to_validation_string();
            };

            SharedString::new()
        }
    });

    ui.global::<SecurityKeyCallbacks>().on_edit({
        let ui = ui.clone_strong();
        move |index, label, icon, live, color| {
            let mut app_state = app_state.borrow_mut();
            let ui_state = ui.global::<SecurityKeyCallbacks>();

            let Ok(index) = usize::try_from(index) else {
                return CallbackResult::from(KeyError::IndexError);
            };

            if let Err(e) = adapt_edit(index, label, icon, live, color, &mut app_state) {
                return CallbackResult::from(e);
            }

            ui_state.set_entries(app_state.get_key_entries());
            ui_state.set_dropdown_model(app_state.get_dropdown_model());
            CallbackResult::success()
        }
    });

    ui.global::<SecurityKeyCallbacks>().on_set_archived({
        let ui = ui.clone_strong();
        move |index, archived| {
            let mut app_state = app_state.borrow_mut();
            let ui_state = ui.global::<SecurityKeyCallbacks>();

            if let Err(e) = adapt_set_archived(index, archived, &mut app_state) {
                log::warn!("{}", e);
                return;
            }

            ui_state.set_entries(app_state.get_key_entries());
            ui_state.set_dropdown_model(app_state.get_dropdown_model());
        }
    });

    ui.global::<SecurityKeyCallbacks>().on_set_archive_mode({
        let ui = ui.clone_strong();
        move |archive_mode| {
            let mut app_state = app_state.borrow_mut();
            let ui_state = ui.global::<SecurityKeyCallbacks>();
            app_state.archive_mode = archive_mode;
            ui_state.set_entries(app_state.get_key_entries());
            ui_state.set_dropdown_model(app_state.get_dropdown_model());
        }
    });

    ui.global::<SecurityKeyCallbacks>().on_move_position({
        let ui = ui.clone_strong();
        move |index, up| {
            let mut app_state = app_state.borrow_mut();
            let ui_state = ui.global::<SecurityKeyCallbacks>();

            if let Err(e) = adapt_move_position(index, up, &mut app_state) {
                log::warn!("{}", e);
                return;
            }

            ui_state.set_entries(app_state.get_key_entries());
            ui_state.set_dropdown_model(app_state.get_dropdown_model());
        }
    });

    ui.global::<SecurityKeyCallbacks>().on_set_sort_mode({
        let ui = ui.clone_strong();
        move |sort_mode| {
            let mut app_state = app_state.borrow_mut();
            let ui_state = ui.global::<SecurityKeyCallbacks>();
            ui_state.set_selected_index(0);
            app_state.set_sort_mode(CardSortMode::from(sort_mode as usize));
            ui_state.set_entries(app_state.get_key_entries());
            ui_state.set_dropdown_model(app_state.get_dropdown_model());
        }
    });

    cx.set_input_handler({
        let ui = ui.clone_strong();
        let gui_api = cx.gui.clone();
        let router = cx.router.clone();
        move |input| {
            if input.msg == InputMessage::NavigationFocused {
                let Ok(Some(nav_bytes)) = gui_api.navigate_pending() else {
                    log::error!("Navigation focused but no pending nav request");
                    return;
                };

                let (table_index, auth_state) = match SecurityKeysNavRequest::from_slice(&nav_bytes) {
                    Some(SecurityKeysNavRequest::UserPresence(options)) => {
                        let app_state_local = app_state.borrow();

                        let table_index = match options.security_key_index {
                            // Key index specified: find the corresponding table entry
                            Some(key_index) => {
                                match app_state_local
                                    .key_table
                                    .iter()
                                    .enumerate()
                                    .filter(|(_i, e)| e.get_index() == key_index)
                                    .next()
                                {
                                    Some((t_i, _key)) => t_i,
                                    None => {
                                        log::warn!("No key with index: {}", key_index);
                                        gui_api
                                            .navigate_finish(UserPresenceResult::new_cancelled().serialize())
                                            .unwrap_or_else(|e| {
                                                log::warn!("could not finish navigation: {}", e);
                                            });
                                        return;
                                    }
                                }
                            }
                            // No key index: default to first available non-archived key for
                            // registration only
                            None => {
                                if options.authentication {
                                    log::warn!("Fido server should tell keys app which key to use");
                                    gui_api
                                        .navigate_finish(UserPresenceResult::new_cancelled().serialize())
                                        .unwrap_or_else(|e| {
                                            log::warn!("could not finish navigation: {}", e);
                                        });
                                    return;
                                }

                                match app_state_local
                                    .key_table
                                    .iter()
                                    .enumerate()
                                    .filter(|(_i, e)| !e.archived)
                                    .next()
                                {
                                    Some((t_i, _key)) => {
                                        log::info!("No key pre-selected, defaulting to first non-archived key at table index {}", t_i);
                                        t_i
                                    }
                                    None => {
                                        log::warn!("No non-archived keys available");
                                        ui.global::<SecurityKeyCallbacks>().set_show_no_keys_modal(true);
                                        gui_api
                                            .navigate_finish(UserPresenceResult::new_cancelled().serialize())
                                            .unwrap_or_else(|e| {
                                                log::warn!("could not finish navigation: {}", e);
                                            });
                                        0
                                    }
                                }
                            }
                        };

                        let auth_state = if options.authentication {
                            AuthenticatingState::AuthenticationConfirm
                        } else {
                            AuthenticatingState::RegistrationConfirm
                        };

                        // Ensure outcome mode is false for user presence checks
                        ui.global::<SecurityKeyCallbacks>().set_is_outcome_mode(false);

                        app_state_local.haptics_api.vibrate(HapticPattern::Alert750ms);

                        log::info!("Got user presence request: {:?}", options);

                        (table_index, auth_state)
                    }
                    Some(SecurityKeysNavRequest::OperationOutcome(options)) => {
                        // Immediately unblock the caller (fire-and-forget)
                        gui_api.navigate_finish(Default::default()).unwrap_or_else(|e| {
                            log::warn!("could not finish navigation: {}", e);
                        });

                        let app_state_local = app_state.borrow();

                        let table_index = match app_state_local
                            .key_table
                            .iter()
                            .enumerate()
                            .filter(|(_i, e)| e.get_index() == options.security_key_index)
                            .next()
                        {
                            Some((t_i, _key)) => t_i,
                            None => {
                                log::warn!("No key with index: {}", options.security_key_index);
                                return; // Already called navigate_finish, just return
                            }
                        };

                        let auth_state = match options.operation {
                            OperationType::Registration => AuthenticatingState::RegistrationSuccess,
                            OperationType::Authentication => {
                                AuthenticatingState::AuthenticationSuccess
                            }
                        };

                        log::info!("Got operation outcome: {:?}", options);

                        // Delay haptic feedback and UI outcome display to allow
                        // the FIDO response to finish transmitting first.
                        {
                            let app_state = app_state.clone();
                            let ui = ui.clone_strong();
                            spawn_local(async move {
                                sleep(Duration::from_millis(500)).await;
                                app_state.borrow().haptics_api.double_click();

                                ui.global::<SecurityKeyCallbacks>().set_is_outcome_mode(true);

                                let ui_state = ui.global::<SecurityKeyCallbacks>();
                                ui_state.set_selected_index(table_index as i32);
                                ui_state.set_auth_state(auth_state);

                                ui.global::<Navigate>().invoke_authenticate(
                                    NavigateOptions { replace: false, animate: Animate::None },
                                );
                            })
                            .detach();
                        }

                        return;
                    }
                    Some(SecurityKeysNavRequest::NoKeysWarning) => {
                        // Immediately unblock the caller (fire-and-forget)
                        gui_api.navigate_finish(Default::default()).unwrap_or_else(|e| {
                            log::warn!("could not finish navigation: {}", e);
                        });

                        // Show the "no keys" modal (rendered on authenticate page)
                        ui.global::<SecurityKeyCallbacks>().set_show_no_keys_modal(true);

                        log::info!("Received no keys warning from FIDO server");

                        // Navigate to authenticate page where the modal is rendered
                        // Use index 0 and RegistrationConfirm state (modal will cover it)
                        (0, AuthenticatingState::RegistrationConfirm)
                    }
                    None => {
                        log::error!("Failed to deserialize SecurityKeysNavRequest");
                        gui_api
                            .navigate_finish(UserPresenceResult::new_cancelled().serialize())
                            .unwrap_or_else(|e| {
                                log::warn!("could not finish navigation: {}", e);
                            });
                        return;
                    }
                };

                let ui_state = ui.global::<SecurityKeyCallbacks>();
                ui_state.set_selected_index(table_index as i32);
                ui_state.set_auth_state(auth_state);

                ui.global::<Navigate>().invoke_authenticate(
                    NavigateOptions { replace: false, animate: Animate::None },
                );

            } else if input.msg == InputMessage::Hidden {
                // If on the authenticate page, navigate backward, and unselect the key, to prevent
                // accidental registrations and authentications while the app is hidden.

                let is_authenticate_page = router.borrow().with_history(|history| {
                    history.get_current_path().map(|path| path == "/authenticate").unwrap_or(false)
                });

                let app_state = app_state.borrow_mut();

                if is_authenticate_page {
                    ui.global::<Navigate>().invoke_backward();
                }

                app_state.fido_api.select_security_key(None);
                log::info!("deselected current key");

                let ui_state = ui.global::<SecurityKeyCallbacks>();
                ui_state.set_is_outcome_mode(false);

                // Gui server will cancel any pending navigations when home or power are pressed.
            }
        }
    });

    ui.global::<SecurityKeyCallbacks>().on_user_presence_check({
        let gui_api = cx.gui.clone();
        let ui = ui.clone_strong();
        move |confirmed| {
            let presence_result = if confirmed {
                // Get the currently selected key index from the UI state
                let ui_state = ui.global::<SecurityKeyCallbacks>();
                let table_index = ui_state.get_selected_index();

                // Get the actual key index from the key table
                let app_state_local = app_state.borrow();
                let selected_key_index = if table_index >= 0 {
                    app_state_local.key_table.get(table_index as usize).map(|key| key.get_index()).ok()
                } else {
                    None
                };

                UserPresenceResult::new_checked(selected_key_index)
            } else {
                UserPresenceResult::new_cancelled()
            };

            gui_api.navigate_finish(presence_result.serialize()).unwrap_or_else(|e| {
                log::warn!("could not finish navigation: {}", e);
            });
        }
    });

    ui.global::<SecurityKeyCallbacks>().on_dismiss_outcome({
        let ui = ui.clone_strong();
        move || {
            // Reset state (navigate_finish was already called in OperationOutcome handler)
            let ui_state = ui.global::<SecurityKeyCallbacks>();
            ui_state.set_is_outcome_mode(false);
            ui_state.set_auth_state(AuthenticatingState::Wait);
        }
    });

    ui.global::<SecurityKeyCallbacks>().on_get_view_index({
        move |entries, source_index| match entries.iter().position(|entry| entry.index == source_index) {
            Some(i) => i as i32,
            None => {
                log::warn!("Could not find index of key that should exist");
                0
            }
        }
    });

    ui.global::<SecurityKeyCallbacks>().on_get_dropdown_index({
        move |entries, source_index| match entries
            .iter()
            .position(|entry| entry.value.as_str().parse::<i32>().unwrap_or(-1) == source_index)
        {
            Some(i) => i as i32,
            None => {
                log::warn!("Could not find index of key that should exist");
                0
            }
        }
    });

    ui.global::<SecurityKeyCallbacks>().on_select_key({
        let ui = ui.clone_strong();
        move |index| {
            let mut app_state = app_state.borrow_mut();
            let ui_state = ui.global::<SecurityKeyCallbacks>();
            ui_state.set_selected_index(index);

            if let Err(e) = adapt_select_key(index, &mut app_state) {
                log::warn!("{}", e);
            }
        }
    });

    ui.global::<SecurityKeyCallbacks>().on_deselect_key({
        let ui = ui.clone_strong();
        move || {
            let app_state = app_state.borrow_mut();
            let ui_state = ui.global::<SecurityKeyCallbacks>();
            ui_state.set_selected_index(0);

            app_state.fido_api.select_security_key(None);
            log::info!("deselected current key");
        }
    });

    #[cfg(keyos)]
    ui.global::<SecurityKeyCallbacks>().on_is_usb_on(move || {
        let app_state = app_state.borrow();
        app_state.usb_device.is_cable_connected().unwrap_or(false)
            && app_state.usb_device.is_device_mode().unwrap_or(false)
            && app_state.usb_device.is_enabled().unwrap_or(false)
    });

    #[cfg(not(keyos))]
    ui.global::<SecurityKeyCallbacks>().on_is_usb_on(|| true);

    ui.global::<SecurityKeyCallbacks>()
        .on_is_nfc_on(move || app_state.borrow().nfc_api.is_enabled().unwrap_or(false));

    ui.run().expect("UI running");
}

fn adapt_save(
    label: SharedString,
    icon: SharedString,
    live: bool,
    color: i32,
    app_state: &mut AppState,
) -> Result<(), KeyError> {
    let key = adapt_validate_new_label(
        label,
        Some(icon),
        Some(color),
        Some(get_timestamp_in_seconds()),
        app_state,
    )?;
    let key_index = key.get_index();

    // Use async (fire-and-forget) calls to avoid blocking UI
    app_state.fido_api.create_security_key();
    app_state.fido_api.set_live(key_index, live);

    app_state.key_table.separate_categories(|k| k.get_category());
    app_state.key_table.push_categorized(|k| k.get_category(), key)?;

    Ok(())
}

fn adapt_validate_new_label(
    label: SharedString,
    icon: Option<SharedString>,
    color: Option<i32>,
    date: Option<u64>,
    app_state: &mut AppState,
) -> Result<Key, KeyError> {
    let label: String = label.into();
    let key_index = app_state.fido_api.next_security_key_index()?;
    let key = Key::new(
        key_index,
        label,
        color.unwrap_or(0) as u8,
        date.unwrap_or(0),
        icon.unwrap_or(SharedString::new()).into(),
    )?;
    let duplicate_opt = app_state.key_table.find(&key)?;

    if let Some((reason, _i)) = duplicate_opt {
        return Err(KeyError::from(reason));
    };

    Ok(key)
}

fn adapt_set_archived(index: i32, archived: bool, app_state: &mut AppState) -> Result<(), KeyError> {
    let Ok(index) = usize::try_from(index) else {
        return Err(KeyError::IndexError);
    };

    if app_state.key_table.get(index)?.archived == archived {
        return Err(KeyError::RedundantArchivalError(index));
    }

    if archived {
        let key_index = app_state.key_table.get(index)?.get_index();
        app_state.fido_api.set_live(key_index, false);
    }

    app_state.key_table.edit(index, |a| {
        a.archived = archived;
        Ok(())
    })?;

    app_state.key_table.separate_categories(|k| k.get_category());
    Ok(())
}

fn adapt_move_position(index: i32, up: bool, app_state: &mut AppState) -> Result<(), KeyError> {
    let Ok(destination) = usize::try_from(index + if up { -1 } else { 1 }) else {
        return Err(KeyError::IndexError);
    };

    let Ok(index) = usize::try_from(index) else {
        return Err(KeyError::IndexError);
    };

    // OrderedTable returns errors safely for underflows
    app_state.key_table.move_position_categorized(|k| k.get_category(), index, destination)?;
    Ok(())
}

fn adapt_edit(
    index: usize,
    label: SharedString,
    icon: SharedString,
    live: bool,
    color: i32,
    app_state: &mut AppState,
) -> Result<(), KeyError> {
    app_state.key_table.edit(index, move |a| {
        a.edit(KeyEditField::Label(label.clone().into()))?;
        a.color = color as u8;
        a.icon = icon.clone().into();
        Ok(())
    })?;

    let key_index = app_state.key_table.get(index)?.get_index();
    app_state.fido_api.set_live(key_index, live);
    Ok(())
}

fn adapt_select_key(index: i32, app_state: &mut AppState) -> Result<(), KeyError> {
    let Ok(index) = usize::try_from(index) else {
        return Err(KeyError::IndexError);
    };

    let key_index = app_state.key_table.get(index)?.get_index();
    app_state.fido_api.select_security_key(Some(key_index));
    log::info!("Selected key: {}", key_index);
    Ok(())
}

fn get_timestamp_in_seconds() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_else(|e| {
            log::error!("Could not get time: {:?}", e);
            Duration::ZERO
        })
        .as_secs()
}
