// SPDX-FileCopyrightText: 2022 Sean Cross <sean@xobs.io>
// SPDX-FileCopyrightText: 2023 Foundation Devices, Inc <hello@foundation.xyz>
// SPDX-License-Identifier: Apache-2.0

#![no_main]
#![no_std]

#[macro_use]
mod args;
mod asm;
mod boot;
mod bootconfig;
mod consts;
mod debug;
mod load;
mod panic;

use core::{mem, ptr, slice};

use args::KernelArguments;
use boot::{map_boot_splash_screen, map_physical_ram};
use bootconfig::BootConfig;
use consts::*;

use crate::{
    asm::start_kernel,
    boot::{map_arguments, map_icm_regions, map_peripherals, map_runtime_page_tracker},
    load::ProgramDescription,
};

pub type XousPid = u8;
pub const PAGE_SIZE: usize = 4096;

const VDBG: bool = false; // verbose debug
const VVDBG: bool = false; // very verbose debug

#[repr(C)]
pub struct MemoryRegionExtra {
    start: u32,
    length: u32,
    name: u32,
    padding: u32,
}

/// Entrypoint
/// This makes the program self-sufficient by setting up memory page assignment
/// and copying the arguments to RAM.
/// Assume the bootloader has already set up the stack to point to the end of RAM.
///
/// # Safety
///
/// This function is safe to call exactly once.
#[export_name = "rust_entry"]
pub unsafe extern "C" fn rust_entry(arg_buffer: *const usize) -> ! {
    let mut pmc = atsama5d27::pmc::Pmc::new();
    pmc.enable_peripheral_clock(atsama5d27::pmc::PeripheralId::Trng);
    let trng = atsama5d27::trng::Trng::new().enable();
    keyos::stack_canary::set_stack_guard(trng.read_u32());

    init_aesb();

    let mut cfg = BootConfig { args: KernelArguments::new(arg_buffer), ..Default::default() };
    read_initial_config(&mut cfg);

    // The first region is defined as being "main RAM", which will be used
    // to keep track of allocations.
    println!("Allocating page tracker");
    allocate_page_tracker(&mut cfg);

    println!("Copying kernel");
    let xkrn_tag = cfg
        .args
        .iter()
        .find(|tag| tag.name == u32::from_le_bytes(*b"XKrn"))
        .expect("Could not find XKrn argument");
    let xkrn_desc = unsafe { &*(xkrn_tag.data.as_ptr() as *const ProgramDescription) };
    xkrn_desc.copy(&mut cfg);
    xkrn_desc.map(&mut cfg);
    println!("Done copying the kernel.");

    // Map boot-generated kernel structures into the kernel
    map_runtime_page_tracker(&mut cfg);
    map_arguments(&mut cfg);
    map_peripherals(&mut cfg);
    map_icm_regions(&mut cfg, xkrn_desc.text_offset as _, xkrn_desc.text_size as _);
    map_physical_ram(&mut cfg);
    map_boot_splash_screen(&mut cfg);

    if VVDBG {
        println!("PID1 pagetables:");
        debug::print_pagetable(cfg.pid1.ttbr0);
        println!();
        println!();
    }
    println!("Runtime Page Tracker: {} bytes", cfg.runtime_page_tracker.len());

    // Create a transparent mapping for a single page of the loader code.
    // The loader code will set up and enable MMU and then jump to the kernel entrypoint.
    // We want to map the same virtual address to the same physical address, so it won't fail
    // as soon as the MMU is enabled
    cfg.map_page(cfg.pid1.ttbr0 as _, LOADER_CODE_ADDRESS, LOADER_CODE_ADDRESS, FLG_R | FLG_X | FLG_VALID);

    println!(
        "Jumping to kernel @ {:08x} with map @ {:08x} and stack @ {:08x}",
        cfg.pid1.entrypoint, cfg.pid1.ttbr0, cfg.pid1.sp,
    );

    unsafe { start_kernel(cfg.pid1.sp, cfg.pid1.ttbr0, cfg.pid1.entrypoint, KERNEL_ARGUMENT_OFFSET) }
}

pub fn read_initial_config(cfg: &mut BootConfig) {
    let mut kernel_seen = false;
    for tag in cfg.args.iter() {
        if tag.name == u32::from_le_bytes(*b"MREx") {
            cfg.regions = unsafe {
                slice::from_raw_parts(
                    tag.data.as_ptr() as *const MemoryRegionExtra,
                    tag.size as usize / mem::size_of::<MemoryRegionExtra>(),
                )
            };
        } else if tag.name == u32::from_le_bytes(*b"XKrn") {
            assert!(!kernel_seen, "kernel appears twice");
            assert!(tag.size as usize == mem::size_of::<ProgramDescription>(), "invalid XKrn size");
            kernel_seen = true;
        }
    }

    assert!(kernel_seen, "no kernel definition");
}

/// Allocate and initialize memory regions.
/// Returns a pointer to the start of the memory region.
pub fn allocate_page_tracker(cfg: &mut BootConfig) {
    // Number of individual pages in the system
    let mut rpt_pages = RAM_PAGES;

    for region in cfg.regions.iter() {
        println!(
            "Discovered memory region {} ({:08x} - {:08x}) -- {} bytes",
            core::str::from_utf8(&region.name.to_le_bytes()).unwrap(),
            region.start,
            region.start + region.length,
            region.length
        );
        let region_length_rounded = (region.length as usize + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
        rpt_pages += region_length_rounded / PAGE_SIZE;
    }

    // Round the tracker to a multiple of the page size, so as to keep memory
    // operations fast.
    rpt_pages = (rpt_pages + mem::size_of::<usize>() - 1) & !(mem::size_of::<usize>() - 1);

    cfg.rpt_size_bytes += rpt_pages * mem::size_of::<XousPid>();
    cfg.rpt_size_bytes = cfg.rpt_size_bytes.next_multiple_of(PAGE_SIZE);

    // Clear all memory pages such that they're not owned by anyone
    let runtime_page_tracker = to_encrypted_phys_addr(cfg.get_top() as _) as *mut usize;
    assert!(to_plaintext_phys_addr(runtime_page_tracker as usize) < ENCRYPTED_DRAM_END);
    unsafe {
        bzero(runtime_page_tracker, runtime_page_tracker.add(rpt_pages / mem::size_of::<usize>()));
    }

    cfg.runtime_page_tracker =
        unsafe { slice::from_raw_parts_mut(runtime_page_tracker as *mut XousPid, rpt_pages) };

    println!("Marking pages as in-use");
    for i in 0..(cfg.rpt_size_bytes / PAGE_SIZE) {
        cfg.runtime_page_tracker[RAM_PAGES - i - 1] = 1;
    }
}

/// Initializes `AESB` encrypted memory bridge.
fn init_aesb() {
    let mut pmc = atsama5d27::pmc::Pmc::new();
    pmc.enable_peripheral_clock(atsama5d27::pmc::PeripheralId::Aesb);
    pmc.enable_peripheral_clock(atsama5d27::pmc::PeripheralId::Trng);

    let trng = atsama5d27::trng::Trng::new().enable();

    let mut nonce = [0u32; 4];
    nonce.fill_with(|| trng.read_u32());

    let aesb = atsama5d27::aesb::Aesb::new();
    aesb.init(atsama5d27::aesb::AesMode::Counter { nonce }, 0);
}

pub unsafe fn memcpy<T>(dest: *mut T, src: *const T, count: usize)
where
    T: Copy,
{
    if VDBG {
        println!(
            "COPY (align {}): {:08x} - {:08x} {} {:08x} - {:08x}",
            mem::size_of::<T>(),
            src as usize,
            src as usize + count,
            count,
            dest as usize,
            dest as usize + count
        );
    }
    core::ptr::copy_nonoverlapping(src, dest, count / mem::size_of::<T>());
}

pub unsafe fn bzero<T>(mut sbss: *mut T, ebss: *mut T)
where
    T: Copy,
{
    if VDBG {
        println!("ZERO: {:08x} - {:08x}", sbss as usize, ebss as usize);
    }
    while sbss < ebss {
        // NOTE(volatile) to prevent this from being transformed into `memclr`
        ptr::write_volatile(sbss, mem::zeroed());
        sbss = sbss.offset(1);
    }
}
