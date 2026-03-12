// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundationdevices.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use atsama5d27::{
    pmc::{PeripheralId, Pmc},
    sckc::{Sckc, SclkType},
    secumod::{
        DynamicProtectionSettings, FilterType, PioPair, PioPairProtection, Protections, Secumod, SystemStatus,
    },
};

const USED_PROTECTIONS: Protections = Protections::from_bits_truncate(
    Protections::DBLFM.bits()
        | Protections::SHLDM.bits()
        | Protections::TPML.bits()
        | Protections::TPMH.bits()
        | Protections::VDDBUL.bits()
        | Protections::VDDBUH.bits()
        | Protections::JTAG.bits()
        | Protections::DET5.bits()
        | Protections::DET7.bits(),
);

pub fn init_tamper_detection() {
    let mut pmc = Pmc::new();
    pmc.enable_peripheral_clock(PeripheralId::Secumod);
    pmc.enable_peripheral_clock(PeripheralId::Securam);

    let secumod = Secumod::new();
    secumod.configure_protection(
        PioPair::new_4_5(),
        PioPairProtection::Dynamic(DynamicProtectionSettings::new(FilterType::Majority3)),
    );
    secumod.configure_protection(
        PioPair::new_6_7(),
        PioPairProtection::Dynamic(DynamicProtectionSettings::new(FilterType::Majority3)),
    );

    let mut protections = USED_PROTECTIONS;
    // First boot since battery insert, crystal is not yet oscillating properly
    if Sckc::default().selected_clock() != SclkType::Crystal {
        protections = protections.difference(Protections::DBLFM);
    }

    secumod.with_protection_registers(|regs| {
        regs.set_normal_mode_protections(protections);
        regs.set_backup_mode_protections(protections);
    });
    while !secumod.is_ram_ready() {}
    secumod.set_normal_mode();
}

pub fn tamper_detected() -> bool {
    let secumod = Secumod::new();
    secumod.system_status().contains(SystemStatus::ERASE_DONE)
        || secumod.protections_status().intersects(USED_PROTECTIONS)
}

pub fn clear_tamper_detection() {
    let secumod = Secumod::new();
    secumod.clear_protections(USED_PROTECTIONS);
    secumod.erase_done_clear();
}
