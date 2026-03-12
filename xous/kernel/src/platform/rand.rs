// SPDX-FileCopyrightText: 2022 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: Apache-2.0

pub fn get_u32() -> u32 {
    // hosted rand code is coupled with arch code.
    #[cfg(any(windows, unix))]
    let rand = crate::arch::rand::get_u32();

    #[cfg(keyos)]
    let rand = crate::platform::atsama5d2::rand::get_u32();

    rand
}
