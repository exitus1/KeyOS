// SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: MIT OR Apache-2.0

pub const FLG_VALID: usize = 1 << 0;
pub const FLG_X: usize = 1 << 1;
pub const FLG_W: usize = 1 << 2;
pub const FLG_R: usize = 1 << 3;
pub const FLG_U: usize = 1 << 4;
pub const FLG_GUARD: usize = 1 << 5;
pub const FLG_DEV: usize = 1 << 6;
pub const FLG_NO_CACHE: usize = 1 << 7;

pub use keyos::*;
