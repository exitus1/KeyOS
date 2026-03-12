// SPDX-FileCopyrightText: 2024-2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use recovery_worker::{
    AppBinVerificationState, ArchiveState, Progress, ProgressKind, ReadArchiveError,
    DOWNGRADE_NOT_ALLOWED_MSG,
};
use slint_keyos_platform::{slint::ComponentHandle, spawn_local, subscribe_scalar};

use crate::{
    recovery_worker_permissions::RecoveryWorkerPermissions, AppWindow, RecoveryGlobal, RecoveryStep,
    RecoveryValidationStep, RecoveryWorkerApi,
};

pub fn init(ui: AppWindow) {
    let mut progress = subscribe_scalar::<RecoveryWorkerPermissions, _>(recovery_worker::SubscribeProgress);
    spawn_local(async move {
        let worker_api = RecoveryWorkerApi::default();
        while let Some(progress) = progress.next().await {
            handle_progress(&worker_api, progress, ui.clone_strong());
        }
    })
    .detach();
}

fn handle_progress(worker_api: &RecoveryWorkerApi, info: Progress, ui: AppWindow) {
    match info.kind {
        ProgressKind::ArchiveRead => {
            if info.is_completed {
                if !info.is_error {
                    if let ArchiveState::Ok {
                        num_apps,
                        num_assets,
                        is_existing,
                        has_recovery_or_bootloader,
                        ..
                    } = worker_api.archive_state()
                    {
                        ui.global::<RecoveryGlobal>().set_new_keyos_valid(false);
                        ui.global::<RecoveryGlobal>().set_num_apps(num_apps as i32);
                        ui.global::<RecoveryGlobal>().set_num_assets(num_assets as i32);
                        ui.global::<RecoveryGlobal>()
                            .set_new_keyos_validation_step(RecoveryValidationStep::Copying);
                        ui.global::<RecoveryGlobal>()
                            .set_is_recovering_critical_components(has_recovery_or_bootloader);

                        // TODO: handle existing archive
                        if !is_existing {
                            ui.global::<RecoveryGlobal>().set_new_keyos_validation_error(false);
                            if let Err(e) = worker_api.copy_archive() {
                                log::error!("Error copying archive: {e:?}");
                                ui.global::<RecoveryGlobal>().set_new_keyos_validation_error(true);
                                ui.global::<RecoveryGlobal>().set_new_keyos_validation_error_str(
                                    format!("Could not copy archive ({e:?})").into(),
                                );
                            }
                        } else {
                            log::info!("Archive already copied, skipping copy step.");
                        }
                    }
                } else {
                    set_archive_error(worker_api, &ui);
                }
            }
        }

        ProgressKind::ArchiveCopy => {
            ui.global::<RecoveryGlobal>().set_new_keyos_validation_step(RecoveryValidationStep::Copying);
            ui.global::<RecoveryGlobal>().set_recovery_progress(info.progress);

            if info.is_completed && !info.is_error {
                ui.global::<RecoveryGlobal>().set_is_tar_copied(true);
            } else if info.is_error {
                set_archive_error(worker_api, &ui);
            }
        }

        ProgressKind::AppBinVerify => {
            if !info.is_error {
                if !info.is_completed {
                    ui.global::<RecoveryGlobal>()
                        .set_new_keyos_validation_step(RecoveryValidationStep::Verifying);
                    ui.global::<RecoveryGlobal>().set_new_keyos_validation_in_progress(true);
                    ui.global::<RecoveryGlobal>().set_recovery_progress(info.progress);
                } else {
                    let verification_state = worker_api.app_bin_verification_state();

                    match verification_state {
                        AppBinVerificationState::None => {
                            ui.global::<RecoveryGlobal>().set_new_keyos_valid(false);
                            ui.global::<RecoveryGlobal>().set_new_keyos_validation_in_progress(false);
                        }
                        AppBinVerificationState::Invalid(e) => {
                            ui.global::<RecoveryGlobal>().set_new_keyos_valid(false);
                            ui.global::<RecoveryGlobal>().set_new_keyos_validation_in_progress(false);
                            ui.global::<RecoveryGlobal>().set_new_keyos_validation_error(true);
                            ui.global::<RecoveryGlobal>().set_new_keyos_validation_error_str(e.into());
                        }
                        AppBinVerificationState::Valid { hash, version, build_date, .. } => {
                            ui.global::<RecoveryGlobal>().set_new_keyos_valid(true);
                            ui.global::<RecoveryGlobal>().set_new_keyos_hash(hash.into());
                            ui.global::<RecoveryGlobal>().set_new_keyos_version(version.into());
                            ui.global::<RecoveryGlobal>().set_new_keyos_build_date(build_date.into());
                            ui.global::<RecoveryGlobal>().set_new_keyos_validation_error(false);
                            ui.global::<RecoveryGlobal>().set_new_keyos_validation_in_progress(false);
                        }
                        AppBinVerificationState::Copied => {
                            ui.global::<RecoveryGlobal>().set_new_keyos_valid(true);
                            ui.global::<RecoveryGlobal>().set_new_keyos_validation_in_progress(false);
                        }
                    }
                }
            } else {
                set_archive_error(worker_api, &ui);
            }
        }

        ProgressKind::Extracting => {
            if info.is_error {
                ui.global::<RecoveryGlobal>().set_recovery_error(true);
                ui.global::<RecoveryGlobal>()
                    .set_recovery_error_str(worker_api.last_error().unwrap_or_default().into());
                ui.global::<RecoveryGlobal>().set_recovery_complete(true);
            } else {
                ui.global::<RecoveryGlobal>().set_recovery_progress(info.progress);
            }

            ui.global::<RecoveryGlobal>().set_curr_recovery_step(RecoveryStep::Extracting);
        }

        ProgressKind::RebootCountdown => {
            ui.global::<RecoveryGlobal>().set_curr_recovery_step(RecoveryStep::Rebooting);
            ui.global::<RecoveryGlobal>().set_recovery_error(false);
            ui.global::<RecoveryGlobal>().set_recovery_complete(true);
        }

        _ => log::error!("Unknown ProgressKind received: {:?}", info.kind),
    }
}

fn set_archive_error(worker_api: &RecoveryWorkerApi, ui: &AppWindow) {
    ui.global::<RecoveryGlobal>().set_new_keyos_valid(false);
    ui.global::<RecoveryGlobal>().set_new_keyos_validation_in_progress(false);
    ui.global::<RecoveryGlobal>().set_new_keyos_validation_error(true);

    if let ArchiveState::Error(error) = worker_api.archive_state() {
        log::error!("Error reading archive: {:?}", error);

        ui.global::<RecoveryGlobal>().set_new_keyos_validation_error_str(archive_error_message(error).into());
    }
}

fn archive_error_message(error: ReadArchiveError) -> String {
    match error {
        ReadArchiveError::DowngradeNotAllowed => DOWNGRADE_NOT_ALLOWED_MSG.into(),
        ReadArchiveError::UnsupportedFormat => "Unsupported file format.".into(),
        ReadArchiveError::MissingRequiredFiles => "Unsupported file format (missing required files).".into(),
        ReadArchiveError::InternalError(e) => format!("Could not read firmware file ({e})"),
    }
}

#[cfg(test)]
mod tests {
    use recovery_worker::{ReadArchiveError, DOWNGRADE_NOT_ALLOWED_MSG};

    use super::archive_error_message;

    #[test]
    fn downgrade_message_is_top_level_policy_error() {
        assert_eq!(archive_error_message(ReadArchiveError::DowngradeNotAllowed), DOWNGRADE_NOT_ALLOWED_MSG);
    }

    #[test]
    fn internal_error_keeps_read_file_context() {
        assert_eq!(
            archive_error_message(ReadArchiveError::InternalError("x".to_string())),
            "Could not read firmware file (x)"
        );
    }

    #[test]
    fn unsupported_format_errors_are_unchanged() {
        assert_eq!(archive_error_message(ReadArchiveError::UnsupportedFormat), "Unsupported file format.");
        assert_eq!(
            archive_error_message(ReadArchiveError::MissingRequiredFiles),
            "Unsupported file format (missing required files)."
        );
    }
}
