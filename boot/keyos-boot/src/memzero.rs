// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundationdevices.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use atsama5d27::{
    dma::{DmaChannel, DmaDataWidth, Xdmac},
    pmc::{PeripheralId, Pmc},
};

pub fn memzero_async(range: core::ops::Range<usize>) {
    let mut pmc = Pmc::new();
    pmc.enable_peripheral_clock(PeripheralId::Xdmac0);

    let channel = Xdmac::xdmac0().channel(DmaChannel::Channel1);
    channel.configure_memset_transfer(DmaDataWidth::D32);

    channel.execute_transfer(0, range.start as u32, range.len() / core::mem::size_of::<u32>());
}

pub fn wait_for_memzero() {
    let channel = Xdmac::xdmac0().channel(DmaChannel::Channel1);
    while !channel.is_transfer_complete() {}
}
