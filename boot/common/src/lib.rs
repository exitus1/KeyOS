// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundationdevices.com>
// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]

use {
    atsama5d27::{
        pit::{Pit, PIV_MAX},
        pmc::{PeripheralId, Pmc},
        sfc::Sfc,
        shdwc::WakeupOptions,
        wdt::{Config, Wdt},
    },
    fuse::{get_board_revision, BoardRevision},
    keyos::{BOOT_SPLASH_PHYS_ADDR, MASTER_CLOCK_SPEED, PLAINTEXT_DRAM_END},
    random::delay,
};

pub mod colors;
pub mod display;
pub mod fonts;
pub mod gui;
pub mod i2c;
pub mod pins;
pub mod random;
pub mod tamper;
pub mod theme;
pub mod touch;

// Screen size
pub const WIDTH: usize = 480;
pub const HEIGHT: usize = 800;

// Memory Layout
//
// DMA descriptors: 128B each
// Progress Bar Overlay: ~200KB
// Frame Buffer 0: ~3.1MB
// Frame Buffer 1: ~3.1MB
// Splash Buffer: ~3.1MB
// Splash DMA: 1 page

pub const FB_SIZE_BYTES: usize = WIDTH * HEIGHT * 4;
pub const FB_BASE_ADDR_1: usize = BOOT_SPLASH_PHYS_ADDR - FB_SIZE_BYTES;
pub const FB_BASE_ADDR_0: usize = FB_BASE_ADDR_1 - FB_SIZE_BYTES;
pub static mut FB_BASE_ADDR_ACTIVE: usize = 0;
pub static mut FB_BASE_ADDR_OFFSCREEN: usize = 0;

const _: () = assert!(
    BOOT_SPLASH_PHYS_ADDR + FB_SIZE_BYTES + 128 <= PLAINTEXT_DRAM_END
        && keyos::BOOT_SPLASH_PAGES * keyos::PAGE_SIZE >= FB_SIZE_BYTES + 128,
    "Verify that BOOT_SPLASH_ADDRESS has enough room for a framebuffer and its DMA desc"
);

pub const PB_HEIGHT: u32 = 3;
pub const PB_OVERLAY_HEIGHT: usize = 100 + PB_HEIGHT as usize;
pub const PB_OVERLAY_SIZE_BYTES: usize = WIDTH * PB_OVERLAY_HEIGHT * 4; // 32 bpp argb
pub const FB_PB_OVERLAY_ADDR: usize = FB_BASE_ADDR_0 - PB_OVERLAY_SIZE_BYTES;

pub const FB_BASE_DMA_ADDR: usize = FB_PB_OVERLAY_ADDR - 128;
pub const PB_OVERLAY_DMA_ADDR: usize = FB_BASE_DMA_ADDR - 128;
pub const SPLASH_DMA_ADDR: usize = BOOT_SPLASH_PHYS_ADDR + FB_SIZE_BYTES;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    // use core::fmt::Write;
    // let mut console = atsama5d27::uart::Uart::<atsama5d27::uart::Uart1>::new();
    // core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
    // writeln!(console, "{}", _info).ok();
    reboot()
}

extern "C" {
    pub fn load_os_image_file(image_name: *const u8, header_only: bool) -> u32;
}

#[no_mangle]
pub extern "C" fn ffi_get_os_image_max_size() -> u32 {
    128 * 1024 * 1024 // 128 MB
}

#[no_mangle]
pub extern "C" fn ffi_random_boot_delay() { random::delay(); }

pub fn reboot() -> ! {
    atsama5d27::rstc::Rstc::new().do_reset();
    unreachable!()
}

pub fn shutdown() -> ! {
    let shdwc = atsama5d27::shdwc::Shdwc::new();

    // Initialize PMC for SFC
    let mut pmc = Pmc::new();
    pmc.enable_peripheral_clock(PeripheralId::Sfc);
    let sfc = Sfc::new();

    if get_board_revision(&sfc) == BoardRevision::RevD1 {
        // Switch to the crystal oscillator before shutting down
        // to ensure that both SHDWC and RXLP are running from the same clock
        let mut sckc = atsama5d27::sckc::Sckc::default();
        sckc.select_clock(atsama5d27::sckc::SclkType::Crystal);
        delay();

        // Workaround for a spurious wake-up from WKUP0 by using RXLP instead (SFT-5196)
        let mut rxlp = atsama5d27::rxlp::Rxlp::new();
        rxlp.init(1, atsama5d27::rxlp::Parity::No);
        rxlp.set_comparison(0x01, 0xff);
        rxlp.read();
        unsafe {
            core::arch::asm!("dsb");
        }

        shdwc.do_shutdown(WakeupOptions::RXLP);
    } else {
        // Use WKUP0 for Rev-D6 boards
        shdwc.do_shutdown(WakeupOptions::WKUP0);
    }
}

#[cfg(not(feature = "production"))]
pub fn enter_sam_ba_mode() {
    let bureg0 = 0xF804_5400 as *mut u32;
    let bsc_cr = 0xF804_8054 as *mut u32;
    unsafe {
        // disable SPI, QSPI and SDMMC boot
        bureg0.write_volatile(0xFFF);
        // use BUREG_0 + BUREG_VALID + WPKEY
        bsc_cr.write_volatile(0x66830004);
    }

    // Disable watchdog before rebooting into sam-ba
    wdt_disable();

    reboot();
}

pub fn get_pit() -> Pit {
    let mut pit = Pit::new();
    pit.set_interval(PIV_MAX);
    pit.set_enabled(true);
    pit.set_clock_speed(MASTER_CLOCK_SPEED);
    pit
}

pub fn wdt_init() {
    let mut wdt = Wdt::new();
    let debug_halt = cfg!(not(feature = "production"));
    wdt.enable(&Config::default().with_debug_halt(debug_halt).with_interrupt(false));
    wdt.restart();
}

pub fn wdt_reset() { Wdt::new().restart(); }

pub fn wdt_disable() { Wdt::new().disable(); }
