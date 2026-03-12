// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: Apache-2.0

//! Integrity Check Monitor

use atsama5d27::icm::{Algorithm, Icm, Interrupts, Region, RegionId};
use keyos::{ICM_KERNEL_ADDR, ICM_KERNEL_DESC_AREA_ADDR, ICM_KERNEL_HASH_AREA_ADDR, PAGE_SIZE};
use xous::arch::irq::IrqNumber;
use xous::MemoryRange;

pub fn init() {
    let mapping = crate::arch::mem::MemoryMapping::current();

    let icm = Icm::with_alt_base_addr(ICM_KERNEL_ADDR as u32);
    icm.reset();
    icm.set_algorithm(Algorithm::Sha256);
    icm.set_secondary_list_branching_disable(true);
    icm.set_double_buffering(true);
    icm.set_automatic_monitoring_mode(true);
    icm.set_eom_disabled(true);
    icm.set_bus_burden(15);

    let (ktext_offset, ktext_size) = unsafe {
        let ptr = ICM_KERNEL_DESC_AREA_ADDR as *const u32;
        (mapping.virt_to_phys(ptr.read_volatile() as _).unwrap() as u32, ptr.add(1).read_volatile())
    };

    let desc_area_phys = mapping.virt_to_phys(ICM_KERNEL_DESC_AREA_ADDR as _).unwrap() as u32;
    icm.set_descriptor_area_address(desc_area_phys).unwrap();

    let hash_area_phys = mapping.virt_to_phys(ICM_KERNEL_HASH_AREA_ADDR as _).unwrap() as u32;
    icm.set_hash_area_address(hash_area_phys).unwrap();

    xous::claim_interrupt(IrqNumber::Icm, icm_irq_fn, core::ptr::null_mut())
        .expect("Couldn't claim ICM interrupt");

    // Interrupt on access violation, bus error and digest mismatch
    icm.enable_interrupts(Region::R0, Interrupts::URAD | Interrupts::RBE | Interrupts::RDM);

    // Monitor the kernel code+rodata region
    icm.start_monitoring_contiguous_region(
        RegionId::R0,
        ICM_KERNEL_DESC_AREA_ADDR as u32,
        ktext_offset,
        ktext_size,
        || {
            mapping
                .flush_cache(
                    unsafe { MemoryRange::new(ICM_KERNEL_DESC_AREA_ADDR, PAGE_SIZE).unwrap() },
                    xous::CacheOperation::Clean,
                )
                .unwrap()
        },
    )
    .unwrap();

    icm.set_enable(true);
}

fn icm_irq_fn(_irq_number: usize, _arg: *mut usize) {
    let icm = Icm::with_alt_base_addr(ICM_KERNEL_ADDR as u32);
    let status = icm.interrupt_status(Region::R0);

    if status.contains(Interrupts::RDM) | status.contains(Interrupts::URAD) {
        panic!("Kernel Integrity Alert");
    }
}
