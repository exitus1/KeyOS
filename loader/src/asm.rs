// SPDX-FileCopyrightText: 2022 Sean Cross <sean@xobs.io>
// SPDX-FileCopyrightText: 2023 Foundation Devices, Inc <hello@foundation.xyz>
// SPDX-License-Identifier: Apache-2.0

extern "C" {
    pub fn start_kernel(
        stack: usize,      // r0
        ttbr: usize,       // r1
        entrypoint: usize, // r2
        args: usize,       // r3
    ) -> !;
}

core::arch::global_asm!(include_str!("asm.S"), options(raw),);
