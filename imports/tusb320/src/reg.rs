// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundationdevices.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use bitfield::bitfield;

#[derive(Debug, Copy, Clone)]
pub enum Registers {
    CurrentModeDetectAdvertise = 0x08,
    StateDirInterruptStatus = 0x09,
    DebounceModeSelectReset = 0x0A,
}

bitfield! {
    pub struct CurrentModeDetectAdvertise(u8);
    impl Debug;
    pub active_cable_detection, _: 0;
    pub accessory_connected, _: 3, 1;
    pub current_mode_detect, _: 5, 4;
    pub current_mode_advertise, set_current_mode_advertise: 7, 6;
}

bitfield! {
    pub struct DebounceModeSelectReset(u8);
    impl Debug;
    pub _reserved, _: 2, 0;
    pub i2c_soft_reset, set_i2c_soft_reset: 3;
    pub mode_select, set_mode_select: 5, 4;
    pub debounce, set_debounce: 7, 6;
}

bitfield! {
    pub struct StateDirInterruptStatus(u8);
    impl Debug;
    pub _reserved, _: 0;
    pub drp_duty_cycle, set_drp_duty_cycle: 2, 1;
    pub _reserved2, _: 3;
    pub interrupt_status, set_interrupt_status: 4;
    pub cable_dir, _: 5;
    pub attached_state, _: 7, 6;
}

#[derive(Debug, Copy, Clone, Default)]
pub enum ModeSelect {
    #[default]
    Port = 0b00,
    Ufp = 0b01,
    Dfp = 0b10,
    Drp = 0b11,
}

impl From<u8> for ModeSelect {
    fn from(value: u8) -> Self {
        match value & 0b11 {
            0b00 => ModeSelect::Port,
            0b01 => ModeSelect::Ufp,
            0b10 => ModeSelect::Dfp,
            0b11 => ModeSelect::Drp,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub enum AttachedState {
    #[default]
    NotAttached = 0b00,
    AttachedSrc = 0b01,
    AttachedSnk = 0b10,
    AttachedAccessory = 0b11,
}

impl From<u8> for AttachedState {
    fn from(value: u8) -> Self {
        match value & 0b11 {
            0b00 => AttachedState::NotAttached,
            0b01 => AttachedState::AttachedSrc,
            0b10 => AttachedState::AttachedSnk,
            0b11 => AttachedState::AttachedAccessory,
            _ => unreachable!(),
        }
    }
}
