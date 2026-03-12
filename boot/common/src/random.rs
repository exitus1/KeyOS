// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundationdevices.com>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Random boot delay derived from TRNG

use {
    crate::{get_pit, MASTER_CLOCK_SPEED},
    atsama5d27::{
        pmc::{PeripheralId, Pmc},
        trng::{Enabled, StatefulTrng, Trng},
    },
};

#[inline(never)]
pub fn delay() {
    const DELAY_MAX_MS: u32 = 20;

    let mut pmc = Pmc::new();
    pmc.enable_peripheral_clock(PeripheralId::Trng);
    pmc.enable_peripheral_clock(PeripheralId::Pit);

    let mut pit = get_pit();

    let trng = Trng::new().enable();
    do_dummy_reads(&trng);

    let delay_ms = trng.read_u32() % DELAY_MAX_MS;
    pit.busy_wait_ms(MASTER_CLOCK_SPEED, delay_ms);
}

/// Perform dummy reads from the TRNG to ensure it is ready for use.
fn do_dummy_reads(trng: &StatefulTrng<Enabled>) {
    const TRNG_DUMMY_READS: usize = 8;
    for _ in 0..TRNG_DUMMY_READS {
        trng.read_u32();
    }
}
