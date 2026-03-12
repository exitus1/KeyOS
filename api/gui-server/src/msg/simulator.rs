// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use num_traits::{FromPrimitive, ToPrimitive};
use server::{AsScalar, FromScalar, SimpleMemoryMessage};
use xous::MemoryRange;

use crate::touch::{Touch, TouchKind};

#[derive(Debug, server::Message)]
#[response(())]
pub struct GetDeviceFrame(pub MemoryRange);

impl From<SimpleMemoryMessage> for GetDeviceFrame {
    fn from(value: SimpleMemoryMessage) -> Self { Self(value.buf) }
}

impl From<GetDeviceFrame> for SimpleMemoryMessage {
    fn from(val: GetDeviceFrame) -> Self { SimpleMemoryMessage { buf: val.0, arg1: 0, arg2: 0 } }
}

#[derive(Debug, server::Message)]
#[response(())]
pub struct GetScreenFrame(pub MemoryRange);

impl From<SimpleMemoryMessage> for GetScreenFrame {
    fn from(value: SimpleMemoryMessage) -> Self { Self(value.buf) }
}

impl From<GetScreenFrame> for SimpleMemoryMessage {
    fn from(val: GetScreenFrame) -> Self { SimpleMemoryMessage { buf: val.0, arg1: 0, arg2: 0 } }
}

#[derive(Debug, server::Message)]
pub struct SetScaleFactor(pub usize);

#[derive(Debug, server::Message)]
pub struct SimulateTouch(pub Touch);

impl FromScalar<4> for Touch {
    fn from_scalar([kind, id, x, y]: [u32; 4]) -> Self {
        Touch { kind: TouchKind::from_u32(kind).unwrap(), id: id as usize, x: x as usize, y: y as usize }
    }
}

impl AsScalar<4> for Touch {
    fn as_scalar(&self) -> [u32; 4] {
        [self.kind.to_u32().unwrap(), self.id as u32, self.x as u32, self.y as u32]
    }
}

#[derive(Debug, server::Message)]
pub struct SimulatePowerButton(pub bool);
