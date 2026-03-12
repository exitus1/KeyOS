// SPDX-FileCopyrightText: 2024-2025 Foundation Devices, Inc. <hello@foundationdevices.com>
// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]
#![cfg(target_arch = "arm")]

use atsama5d27::sfc::{Sfc, SfcStatus};

// Refer to Final Security Model document for more information on the fuse allocation.
const COLORWAY_FUSE_REGISTER: usize = 15;
const COLORWAY_FUSE_OFFSET: usize = 0;

const BOARD_REV_MASK: u32 = 0b111;
const BOARD_REV_OFFSET: usize = 1;

const ENTROPY_FUSE_REGISTER: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Colorway {
    /// The device was provisioned with the Dark colorway.
    Dark,

    /// The device was provisioned with the Light colorway.
    Light,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardRevision {
    /// Board revision D1.
    RevD1,

    /// Board revision D6.
    RevD6,
}

/// Returns `true` if the fuse controller has detected an error.
/// Continuing the boot is not recommended in this case, as the device may be under
/// attack.
fn has_error(sfc: &Sfc) -> bool {
    let status = sfc.status();
    status.contains(SfcStatus::LCHECK | SfcStatus::ACE)
}

/// Returns the colorway of the device.
pub fn get_colorway(sfc: &Sfc) -> Option<Colorway> {
    if has_error(sfc) {
        return None;
    }

    let colorway_fuse_reg = sfc.read(COLORWAY_FUSE_REGISTER).unwrap_or_default();
    let colorway_fuse_bit = (colorway_fuse_reg >> COLORWAY_FUSE_OFFSET) & 1;

    Some(if colorway_fuse_bit == 0 { Colorway::Dark } else { Colorway::Light })
}

/// Returns the board revision of the device.
pub fn get_board_revision(sfc: &Sfc) -> BoardRevision {
    if has_error(sfc) {
        return BoardRevision::RevD6;
    }

    let fuse_reg = sfc.read(COLORWAY_FUSE_REGISTER).unwrap_or_default();
    let rev_bits = (fuse_reg >> BOARD_REV_OFFSET) & BOARD_REV_MASK;

    if rev_bits == 0 {
        BoardRevision::RevD1
    } else {
        BoardRevision::RevD6
    }
}

/// Returns the 32-byte random value stored in the fuse bits of the device.
pub fn get_entropy(sfc: &Sfc) -> Option<[u8; 32]> {
    if has_error(sfc) {
        return None;
    }

    let mut fuse_regs = [0u32; 8];
    for (i, fuse_reg) in fuse_regs.iter_mut().enumerate() {
        *fuse_reg = sfc.read(ENTROPY_FUSE_REGISTER + i).unwrap_or_default();
    }

    let mut entropy = [0u8; 32];
    for (i, fuse_reg) in fuse_regs.iter().enumerate() {
        for (j, fuse_byte) in fuse_reg.to_le_bytes().iter().enumerate() {
            entropy[i * 4 + j] = *fuse_byte;
        }
    }

    Some(entropy)
}
