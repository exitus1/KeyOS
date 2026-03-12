// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{cell::RefCell, rc::Rc};

use anyhow::Result;
use keycard_scan::backup::{backup_keycards, BackupKind};
use slint_keyos_platform::{slint::ComponentHandle, spawn_local, StoredValue, TaskHandle};

use crate::{
    haptics_permissions, keycard_permissions, quantum_link_permissions, state::AppState, AppWindow,
    KeycardBackupError, KeycardBackupGlobal,
};

#[derive(Default, Debug)]
enum BackupState {
    #[default]
    Running,
    NeedsConfirmation {
        confirmation: oneshot::Sender<bool>,
    },
}

pub struct KeycardBackupFlow {
    _task: TaskHandle<()>,
    state: Rc<RefCell<BackupState>>,
    _kind: BackupKind,
}

impl KeycardBackupFlow {
    pub fn start(app_state: StoredValue<AppState>, kind: BackupKind) {
        let ui = app_state.borrow().ui();
        let state = Rc::new(RefCell::new(BackupState::Running));

        let task = spawn_local({
            let state = state.clone();
            async move {
                if let Err(e) = run_backup(kind, &ui, state.clone()).await {
                    log::error!("Backup flow failed: {e:?}");
                    // todo: add fatal error screen
                }
            }
        });

        app_state.borrow_mut().keycard_backup = Some(Self { _task: task, state, _kind: kind });
    }

    // user has confirmed that they want to overwrite keycard
    pub fn handle_error_click(state: StoredValue<AppState>, confirm: bool) {
        let mut keycard_backup = state.borrow_mut().map(|s| &mut s.keycard_backup);
        let Some(flow) = keycard_backup.as_mut() else { return };

        let flow_state = std::mem::take(&mut *flow.state.borrow_mut());
        match flow_state {
            BackupState::NeedsConfirmation { confirmation } => {
                log::info!("sending confirmation to overwrite keycard");
                let _ = confirmation.send(confirm);
            }
            flow => {
                log::warn!("invaild state {flow:?}")
            }
        }
    }
}

async fn run_backup(kind: BackupKind, ui: &AppWindow, state: Rc<RefCell<BackupState>>) -> Result<()> {
    log::info!("starting manual keycard backup flow");

    let global = ui.global::<KeycardBackupGlobal>();
    let mut adapter =
        BackupStateAdapter { global: &global, state, kind, saved_shard_index: 0, saving_to_keycard: false };

    backup_keycards::<
        _,
        keycard_permissions::KeycardPermissions,
        quantum_link_permissions::QuantumLinkPermissions,
        haptics_permissions::HapticsPermissions,
    >(&mut adapter, kind)
    .await
    .map_err(|e| anyhow::anyhow!("backup_keycards failed: {e:?}"))?;

    Ok(())
}

keycard_scan::impl_backup_state_adapter!();
keycard_scan::impl_keycard_backup_error_from!();
keycard_scan::backup_impl_to_step_model!();
