// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundationdevices.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::{assets::Asset, selected_boot_image_kind, BootImageKind},
    arrayvec::ArrayVec,
    boot_common::gui::MAX_MESSAGE_LINES,
    core::{mem::MaybeUninit, sync::atomic::AtomicBool},
};

#[derive(Copy, Clone, PartialEq)]
pub enum SystemErrorCode {
    Panic,
    FirmwareVerification,
    #[cfg_attr(not(feature = "tamper"), allow(dead_code))]
    Tamper,
    Watchdog,
}

pub enum CtaAction {
    StartKeyOS,
    StartRecoveryOS,
}

pub struct SystemError {
    code: SystemErrorCode,
    // The message is a list of strings to allow for multi-line error messages.
    // The caller is responsible for splitting the text into lines.
    // The maximum width is 29 characters.
    message: ArrayVec<&'static str, MAX_MESSAGE_LINES>,
}

impl SystemErrorCode {
    fn get_error_info(
        &self,
    ) -> (
        &'static [&'static str], // base_message
        &'static str,            // title
        Option<&'static Asset>,  // qr_code
        Option<&'static str>,    // cta_label
        Option<CtaAction>,       // cta_action
    ) {
        match self {
            SystemErrorCode::Panic => (
                &["KeyOS restarted due to an", "unexpected system error.", ""],
                "Error",
                Some(&crate::assets::ASSET_SYS_ERROR_QR),
                Some("Start KeyOS"),
                Some(CtaAction::StartKeyOS),
            ),
            SystemErrorCode::FirmwareVerification => match selected_boot_image_kind() {
                BootImageKind::Main | BootImageKind::UpdatedMain => (
                    &[
                        "KeyOS is unable to verify the",
                        "signature of the installed",
                        "firmware.",
                        "",
                        "Use Recovery mode to install",
                        "Foundation-signed firmware.",
                    ],
                    "Firmware Error",
                    Some(&crate::assets::ASSET_FW_ERROR_QR),
                    Some("Begin Firmware Recovery"),
                    Some(CtaAction::StartRecoveryOS),
                ),
                BootImageKind::Recovery => (
                    &[
                        "Unable to verify the signature",
                        "of the Recovery firmware.",
                        "",
                        "Contact support using the",
                        "button below.",
                    ],
                    "Recovery Firmware Error",
                    Some(&crate::assets::ASSET_REC_ERROR_QR),
                    None,
                    None,
                ),
            },
            SystemErrorCode::Tamper => (
                &[
                    "KeyOS has detected a tamper",
                    "event.",
                    "",
                    "For security, the Master Key",
                    "has been erased from the",
                    "device. To proceed, restore",
                    "a backup or perform a",
                    "factory reset.",
                ],
                "Tamper Detected",
                Some(&crate::assets::ASSET_TAMPER_ERROR_QR),
                Some("Continue"),
                Some(CtaAction::StartKeyOS),
            ),
            SystemErrorCode::Watchdog => (
                &[
                    "KeyOS restarted due to",
                    "a system error.",
                    "",
                    "It is safe to continue",
                    "using Passport.",
                    "",
                    "Your data remains secure.",
                ],
                "Error",
                None,
                Some("Start KeyOS"),
                Some(CtaAction::StartKeyOS),
            ),
        }
    }
}

impl SystemError {
    pub fn new(code: SystemErrorCode, info: impl IntoIterator<Item = &'static str>) -> Self {
        let (base_message, _, _, _, _) = code.get_error_info();
        let base_message_empty = base_message.is_empty();
        let mut message = ArrayVec::new();
        for base_message_line in base_message {
            message.try_push(*base_message_line).ok();
        }
        let mut info = info.into_iter().peekable();
        if info.peek().is_some() {
            if !base_message_empty {
                message.try_push("").ok();
            }
            for info_line in info {
                message.try_push(info_line).ok();
            }
        }
        Self { code, message }
    }
}

// XXX: This needs to be force zero-inited because otherwise it goes into a read-write
// data segment, which means that in-memory the bootloader will differ from the file,
// which in turn breaks hash calculation.
// Note that simply using Vec::new() is not zero-init, because one of its internal
// pointers will have the value "4" (aligned non-zero dangling pointer), and Option::None
// is also not guaranteed to be 0 (in fact it was 0x80000000 in this case when I checked).
static mut SYSTEM_ERROR: MaybeUninit<SystemError> = MaybeUninit::zeroed();
static SYSTEM_ERROR_SET: AtomicBool = AtomicBool::new(false);

pub fn set_system_error(code: SystemErrorCode, info: impl IntoIterator<Item = &'static str>) {
    let system_error = unsafe { &mut *core::ptr::addr_of_mut!(SYSTEM_ERROR) };
    *system_error = MaybeUninit::new(SystemError::new(code, info));
    SYSTEM_ERROR_SET.store(true, core::sync::atomic::Ordering::SeqCst);
}

pub fn get_system_error() -> (
    Option<SystemErrorCode>,
    &'static str,
    ArrayVec<&'static str, MAX_MESSAGE_LINES>,
    Option<&'static Asset>,
    Option<&'static str>,
    Option<CtaAction>,
) {
    if SYSTEM_ERROR_SET.load(core::sync::atomic::Ordering::SeqCst) {
        let system_error = unsafe { (*core::ptr::addr_of_mut!(SYSTEM_ERROR)).assume_init_ref() };
        let (_, title, qr, label, action) = system_error.code.get_error_info();
        (Some(system_error.code), title, system_error.message.clone(), qr, label, action)
    } else {
        (None, "", ArrayVec::new(), None, None, None)
    }
}
