// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use server::{AsScalar, FromScalar};

use crate::KeyboardKind;

#[derive(Debug, server::Message)]
pub struct UpdateKeyboard {
    pub kind: KeyboardKind,
    pub request_caps: bool,
}

impl FromScalar<2> for UpdateKeyboard {
    fn from_scalar([kind, request_caps]: [u32; 2]) -> Self {
        Self { kind: KeyboardKind::from_scalar([kind]), request_caps: bool::from_scalar([request_caps]) }
    }
}

impl AsScalar<2> for UpdateKeyboard {
    fn as_scalar(&self) -> [u32; 2] {
        let [kind] = self.kind.as_scalar();
        let [request_caps] = self.request_caps.as_scalar();
        [kind, request_caps]
    }
}

#[derive(Debug, server::Message)]
pub struct HideKeyboard;

#[derive(Debug, server::Message)]
pub struct KeyPressed(pub Option<crate::Key>);

#[derive(Debug, server::Message)]
pub struct KeyReleased(pub Option<crate::Key>);
