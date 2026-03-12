// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use num_traits::{FromPrimitive, ToPrimitive};
use server::SimpleMemoryMessage;
use xous::MemoryRange;

use crate::{Peripheral, SpiError};

#[derive(Debug, server::Message)]
#[response(Result<(), SpiError>)]
pub struct ClaimPeripheral(pub Peripheral);

#[derive(Debug, server::Message)]
#[response(Result<usize, SpiError>)]
pub struct SpiXfer {
    pub buffer: MemoryRange,
    pub bytes: usize,
    pub peripheral: Peripheral,
}

impl From<SimpleMemoryMessage> for SpiXfer {
    fn from(value: SimpleMemoryMessage) -> Self {
        Self {
            buffer: value.buf,
            bytes: value.arg1,
            peripheral: Peripheral::from_usize(value.arg2).unwrap_or(Peripheral::Nfc),
        }
    }
}

impl From<SpiXfer> for SimpleMemoryMessage {
    fn from(value: SpiXfer) -> Self {
        SimpleMemoryMessage {
            buf: value.buffer,
            arg1: value.bytes,
            arg2: value.peripheral.to_usize().unwrap(),
        }
    }
}

#[derive(Debug, server::Message)]
#[response(Result<usize, SpiError>)]
pub struct St25r95ReadData {
    pub buffer: MemoryRange,
    pub peripheral: Peripheral,
}

impl From<SimpleMemoryMessage> for St25r95ReadData {
    fn from(value: SimpleMemoryMessage) -> Self {
        Self { buffer: value.buf, peripheral: Peripheral::from_usize(value.arg2).unwrap_or(Peripheral::Nfc) }
    }
}

impl From<St25r95ReadData> for SimpleMemoryMessage {
    fn from(value: St25r95ReadData) -> Self {
        SimpleMemoryMessage { buf: value.buffer, arg1: 0, arg2: value.peripheral.to_usize().unwrap() }
    }
}

#[derive(Debug, server::Message)]
#[response(Result<usize, SpiError>)]
pub struct NrfReadData {
    pub buffer: MemoryRange,
    pub timeout_ms: usize,
    pub peripheral: Peripheral,
    pub bytes: usize,
}

impl From<SimpleMemoryMessage> for NrfReadData {
    fn from(value: SimpleMemoryMessage) -> Self {
        Self {
            buffer: value.buf,
            timeout_ms: value.arg1,
            bytes: (value.arg2 >> 8),
            peripheral: Peripheral::from_usize(value.arg2 & 0xFF).unwrap_or(Peripheral::Nfc),
        }
    }
}

impl From<NrfReadData> for SimpleMemoryMessage {
    fn from(value: NrfReadData) -> Self {
        SimpleMemoryMessage {
            buf: value.buffer,
            arg1: value.timeout_ms,
            arg2: value.peripheral.to_usize().unwrap() | (value.bytes << 8),
        }
    }
}
