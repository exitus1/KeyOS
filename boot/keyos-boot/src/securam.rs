// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundationdevices.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::{
        boot_screen::{show_boot_screen, BootScreenPage},
        system_errors::{set_system_error, SystemErrorCode},
        EXTRA_ENTROPY,
    },
    atsama5d27::{
        pmc::{PeripheralId, Pmc},
        rstc::{ResetCause, Rstc},
        securam::HW_SECURAM_BASE,
        sfc::Sfc,
        sha::Sha,
    },
    securam_manager::{Error, KernelPanicMessage, OsArguments, SecuramManager, NUM_SECURAM_AES_KEYS},
};

static mut PANIC_MESSAGE_COPY: KernelPanicMessage = KernelPanicMessage::new_empty();

#[cfg(feature = "tamper")]
pub(crate) fn check_tamper_detection() {
    if boot_common::tamper::tamper_detected() {
        let error = SystemErrorCode::Tamper;
        set_system_error(error, [""]);
        crate::splash::hide_progress_layer();
        crate::splash::hide_splash_layer();
        show_boot_screen(BootScreenPage::SystemError);
    }
}

macro_rules! hash_fields {
    ($sha:ident, $($field:expr, $len:expr),*) => {{
        let mut copied = [0u8; $($len +)* 0];
        let mut d: &mut [u8] = &mut copied;
        $(
            d[..$len].copy_from_slice(&$field);
            d = &mut d[$len..];
        )*
        let _ = d;
        $sha.hash::<atsama5d27::sha::Sha256>(&copied)
    }}
}
pub(crate) fn init_securam() {
    let mut pmc = Pmc::new();
    pmc.enable_peripheral_clock(PeripheralId::Sha);
    pmc.enable_peripheral_clock(PeripheralId::Securam);

    let sha = Sha::new();
    let mut securam_manager =
        unsafe { SecuramManager::new(HW_SECURAM_BASE as *mut _) }.unwrap_or_else(|_| {
            let mut securam_manager = unsafe { SecuramManager::new_clear(HW_SECURAM_BASE as *mut _) };
            let fuse_entropy = fuse::get_entropy(&Sfc::new()).unwrap_or_default();

            #[cfg(not(feature = "production"))]
            let io_protection_secret = hash_fields!(
                sha,
                fuse_entropy,
                32,
                EXTRA_ENTROPY,
                EXTRA_ENTROPY.len(),
                "IO Protection Secret".as_bytes(),
                "IO Protection Secret".len()
            );
            #[cfg(feature = "production")]
            let io_protection_secret = hash_fields!(
                sha,
                fuse_entropy,
                32,
                "IO Protection Secret".as_bytes(),
                "IO Protection Secret".len()
            );
            securam_manager.set_io_protection_secret(&io_protection_secret).ok();
            securam_manager
                .set_bluetooth_challenge_secret(&hash_fields!(
                    sha,
                    fuse_entropy,
                    32,
                    "Bluetooth Secret".as_bytes(),
                    "Bluetooth Secret".len()
                ))
                .ok();

            securam_manager.set_magic().ok();
            securam_manager
        });
    // Security check secret should always reflect the currently running bootloader.
    securam_manager
        .set_security_check_secret(&hash_fields!(
            sha,
            EXTRA_ENTROPY,
            EXTRA_ENTROPY.len(),
            "Security Check".as_bytes(),
            "Security Check".len()
        ))
        .ok();
    // Clear all AES keys between boots to not leak data accidentally
    for key in 0..NUM_SECURAM_AES_KEYS {
        securam_manager.set_aes_key(key, &[0; 32]).ok();
    }
    // Disk keys should only be present when the user correctly logged in
    securam_manager.set_disk_encryption_keys((&[0; 32], &[0; 32])).ok();
}

pub(crate) fn set_os_arguments(os_arguments: &OsArguments) -> Result<(), Error> {
    unsafe { SecuramManager::new(HW_SECURAM_BASE as *mut _) }?.set_os_arguments(os_arguments)?;
    Ok(())
}

fn read_panic_message() -> Option<KernelPanicMessage> {
    let mut securam_manager = unsafe { SecuramManager::new(HW_SECURAM_BASE as *mut _) }.ok()?;
    let msg = securam_manager.kernel_panic_message().ok()?.clone();
    if !msg.is_empty() {
        securam_manager.set_kernel_panic_message(&KernelPanicMessage::new_empty()).ok();
        return Some(msg);
    }

    None
}

pub(crate) fn check_reset_cause() {
    let rstc = Rstc::new();
    match rstc.reset_cause() {
        ResetCause::General | ResetCause::Wkup | ResetCause::User => (), // proceed as usual
        ResetCause::Wdt => {
            let error = SystemErrorCode::Watchdog;
            set_system_error(error, [""]);
            crate::splash::hide_progress_layer();
            crate::splash::hide_splash_layer();
            show_boot_screen(BootScreenPage::SystemError);
        }
        ResetCause::Software => {
            if let Some(panic_message) = read_panic_message() {
                unsafe { PANIC_MESSAGE_COPY = panic_message }
                #[allow(static_mut_refs)]
                if let Some(panic_str) = unsafe { PANIC_MESSAGE_COPY.as_str() } {
                    let error = SystemErrorCode::Panic;
                    set_system_error(error, panic_str.lines());
                    crate::splash::hide_progress_layer();
                    crate::splash::hide_splash_layer();
                    show_boot_screen(BootScreenPage::SystemError);
                }
            }
        }
        ResetCause::Reserved5 => {}
        ResetCause::Reserved6 => {}
        ResetCause::SlckXtal => {}
        ResetCause::Unknown => {}
    }
}
