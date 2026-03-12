// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};

#[derive(Debug, Copy, Clone, PartialEq, Eq, ToPrimitive, FromPrimitive)]
pub enum TouchKind {
    Press = 0,
    Release,
    Drag,
}

#[derive(Debug, Copy, Clone)]
pub struct Touch {
    pub kind: TouchKind,
    pub id: usize,
    pub x: usize,
    pub y: usize,
}

impl Touch {
    #[must_use]
    pub fn translate_pos(&self, origin_x: usize, origin_y: usize) -> Touch {
        Touch { x: self.x.saturating_sub(origin_x), y: self.y.saturating_sub(origin_y), ..*self }
    }

    #[must_use]
    pub fn with_offset(&self, offset_x: i32, offset_y: i32) -> Touch {
        Touch {
            x: (self.x as i32).saturating_add(offset_x) as usize,
            y: (self.y as i32).saturating_add(offset_y) as usize,
            ..*self
        }
    }

    pub fn is_within_area(&self, x: usize, y: usize, w: usize, h: usize) -> bool {
        (self.x >= x && self.x < x + w) && (self.y >= y && self.y < y + h)
    }

    pub fn is_press(&self) -> bool { matches!(self.kind, TouchKind::Press) }

    pub fn is_drag(&self) -> bool { matches!(self.kind, TouchKind::Drag) }

    pub fn is_release(&self) -> bool { matches!(self.kind, TouchKind::Release) }

    /// Used by `gui-server` to convert `Touch` message into `InputMessage` format for the
    /// use in GUI apps.
    pub fn as_input_message(&self, msg_id: usize) -> xous::Message {
        xous::Message::new_scalar(msg_id, self.kind.to_usize().expect("to u32"), self.id, self.x, self.y)
    }

    /// To be used by GUI apps to parse an `InputMessage` from the `gui-server`'s
    /// `receive_input()`.
    pub fn try_from_input_message(msg: &xous::Message) -> Option<Self> {
        let scalar = msg.scalar_message()?;
        let kind = TouchKind::from_usize(scalar.arg1)?;
        let id = scalar.arg2;
        let x = scalar.arg3;
        let y = scalar.arg4;
        Some(Touch { kind, id, x, y })
    }

    pub fn diff(&self, other: &Touch) -> (isize, isize) {
        (self.x as isize - other.x as isize, self.y as isize - other.y as isize)
    }

    /// Calculate the distance between two touches.
    pub fn distance_to(&self, other: &Touch) -> f32 {
        let (dx, dy) = self.diff(other);
        (dx as f32 * dx as f32 + dy as f32 * dy as f32).sqrt()
    }
}
