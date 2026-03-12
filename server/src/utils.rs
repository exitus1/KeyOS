// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::WrongMessageTypeError;

#[inline]
pub(crate) fn extract_move_message<'m>(
    raw: &'m mut xous::MessageEnvelope,
) -> core::result::Result<xous_ipc::Buffer<'m>, rkyv::rancor::Error> {
    match &mut raw.body {
        xous::Message::Move(mem) => Ok(unsafe { xous_ipc::Buffer::from_memory_message(mem) }),
        _ => rkyv::rancor::fail!(WrongMessageTypeError),
    }
}

#[inline]
pub(crate) fn extract_borrow_mut_message<'m>(
    raw: &'m mut xous::MessageEnvelope,
) -> core::result::Result<xous_ipc::Buffer<'m>, rkyv::rancor::Error> {
    match &mut raw.body {
        xous::Message::MutableBorrow(mem) => Ok(unsafe { xous_ipc::Buffer::from_memory_message_mut(mem) }),
        _ => rkyv::rancor::fail!(WrongMessageTypeError),
    }
}

#[inline]
pub(crate) fn extract_scalar_message(
    raw: &mut xous::MessageEnvelope,
) -> core::result::Result<[u32; 4], rkyv::rancor::Error> {
    match &mut raw.body {
        xous::Message::Scalar(scalar) => {
            let [_, arg1, arg2, arg3, arg4] = scalar.to_usize().map(|a| a as u32);
            Ok([arg1, arg2, arg3, arg4])
        }
        _ => rkyv::rancor::fail!(WrongMessageTypeError),
    }
}

#[inline]
pub(crate) fn scalar_to_message(s: &impl crate::AsScalar<4>, msg_id: usize) -> xous::ScalarMessage {
    let [arg1, arg2, arg3, arg4] = s.as_scalar().map(|a| a as usize);
    xous::ScalarMessage { id: msg_id, arg1, arg2, arg3, arg4 }
}

#[inline]
pub(crate) fn scalar_from_message<M: crate::FromScalar<4>>(msg: &xous::ScalarMessage) -> M {
    let [_, arg1, arg2, arg3, arg4] = msg.to_usize().map(|a| a as u32);
    M::from_scalar([arg1, arg2, arg3, arg4])
}
