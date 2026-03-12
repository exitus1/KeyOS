// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundationdevices.com>
// SPDX-License-Identifier: MIT OR Apache-2.0

#![no_std]
#![no_main]

use {
    atsama5d27::{
        aic::{Aic, InterruptEntry, SourceKind},
        pit::{Pit, PIV_MAX},
        pmc::{PeripheralId, Pmc},
        secumod::{
            FilterValue,
            PioPair,
            PioPairProtection,
            Protections,
            Secumod,
            StaticProtectionSettings,
        },
        securam::Securam,
        uart::{Uart, Uart1},
    },
    core::{
        arch::global_asm,
        fmt::Write,
        panic::PanicInfo,
        ptr::addr_of_mut,
        sync::atomic::{compiler_fence, Ordering::SeqCst},
    },
};

global_asm!(include_str!("../start.S"));

type UartType = Uart<Uart1>;
const UART_PERIPH_ID: PeripheralId = PeripheralId::Uart1;

// MCK: 164MHz
// Clock frequency is divided by 2 because of the default `h32mxdiv` PMC setting
const MASTER_CLOCK_SPEED: u32 = 164000000 / 2;

#[no_mangle]
fn _entry() -> ! {
    extern "C" {
        // These symbols come from `link.ld`
        static mut _sbss: u32;
        static mut _ebss: u32;
    }

    // Initialize RAM
    unsafe {
        r0::zero_bss(addr_of_mut!(_sbss), addr_of_mut!(_ebss));
    }

    atsama5d27::l1cache::disable_dcache();

    let mut pmc = Pmc::new();
    pmc.enable_peripheral_clock(PeripheralId::Pit);
    pmc.enable_peripheral_clock(PeripheralId::Aic);
    pmc.enable_peripheral_clock(PeripheralId::Pioa);
    pmc.enable_peripheral_clock(PeripheralId::Piob);
    pmc.enable_peripheral_clock(PeripheralId::Pioc);
    pmc.enable_peripheral_clock(PeripheralId::Piod);
    pmc.enable_peripheral_clock(PeripheralId::Secumod);
    pmc.enable_peripheral_clock(PeripheralId::Securam);

    let mut aic = Aic::new();
    aic.init();
    aic.set_spurious_handler_fn_ptr(aic_spurious_handler as unsafe extern "C" fn() as usize);

    let uart_irq_ptr = uart_irq_handler as unsafe extern "C" fn() as usize;
    aic.set_interrupt_handler(InterruptEntry {
        peripheral_id: UART_PERIPH_ID,
        vector_fn_ptr: uart_irq_ptr,
        kind: SourceKind::LevelSensitive,
        priority: 0,
    });

    // Enable interrupts
    unsafe {
        core::arch::asm!("cpsie if");
    }

    // Timer for delays
    let mut pit = Pit::new();
    pit.set_interval(PIV_MAX);
    pit.set_enabled(true);
    pit.set_clock_speed(MASTER_CLOCK_SPEED);
    pit.busy_wait_ms(MASTER_CLOCK_SPEED, 500);

    let mut uart = UartType::new();
    uart.set_rx_interrupt(true);
    uart.set_rx(true);
    writeln!(uart, "Running").ok();

    let mut secumod = Secumod::new();
    secumod.set_normal_mode();
    // TODO: (OPS-418) dynamic tamper protection isn't supported by the normally-closed switch
    // secumod.configure_protection(
    //     PioPair::new_4_5(),
    //     PioPairProtection::Dynamic(DynamicProtectionSettings::new(FilterType::Majority3))
    // );
    // secumod.configure_protection(
    //     PioPair::new_6_7(),
    //     PioPairProtection::Dynamic(DynamicProtectionSettings::new(FilterType::Majority3))
    // );
    secumod.configure_protection(
        PioPair::new_4_5(),
        PioPairProtection::Static(StaticProtectionSettings::new(
            FilterValue::Max2,
            FilterValue::Max2,
            true,
        )),
    );
    secumod.configure_protection(
        PioPair::new_6_7(),
        PioPairProtection::Static(StaticProtectionSettings::new(
            FilterValue::Max2,
            FilterValue::Max2,
            true,
        )),
    );
    secumod.with_normal_mode_protections_mut(|protections: &mut Protections| {
        *protections = Protections::DET5 | Protections::DET7;
    });

    writeln!(
        uart,
        "Triggered protections: {:?}",
        secumod.protections_status()
    )
    .ok();
    writeln!(uart, "SECUMOD Sys Status: {:?}", secumod.system_status()).ok();

    while !secumod.is_ram_ready() {
        writeln!(uart, "Waiting for SECURAM to be ready").ok();
        pit.busy_wait_ms(MASTER_CLOCK_SPEED, 500);
    }

    let mut securam = Securam::new();
    writeln!(uart, "Lower 4K SECURAM:").ok();
    print_hex(securam.lower());

    loop {
        let protections = secumod.protections_status();
        writeln!(uart, "Triggered protections: {:?}", protections).ok();
        writeln!(uart, "SECUMOD Sys Status: {:?}", secumod.system_status()).ok();
        secumod.clear_protections(Protections::DET7 | Protections::DET5 | Protections::JTAG);
        secumod.erase_done_clear();

        pit.busy_wait_ms(MASTER_CLOCK_SPEED, 1000);
    }
}

fn print_hex(slice: &[u8]) {
    let mut uart = UartType::new();

    const CHUNK_SIZE: usize = 64;
    let mut temp_buf: [u8; CHUNK_SIZE * 2] = [0; CHUNK_SIZE * 2];

    for chunk in slice.chunks_exact(CHUNK_SIZE) {
        temp_buf.fill(0);
        hex::encode_to_slice(chunk, &mut temp_buf).unwrap();
        let hash_str = core::str::from_utf8(&temp_buf).unwrap();
        writeln!(uart, "{}", hash_str).ok();
    }

    let rem = slice.chunks_exact(CHUNK_SIZE).remainder();
    if !rem.is_empty() {
        temp_buf.fill(b'0');
        hex::encode_to_slice(rem, &mut temp_buf[..rem.len() * 2]).unwrap();
        let hash_str = core::str::from_utf8(&temp_buf).unwrap();
        writeln!(uart, "{}", hash_str).ok();
    }

    writeln!(uart).ok();
}

#[no_mangle]
unsafe extern "C" fn aic_spurious_handler() {
    core::arch::asm!("bkpt");
}

#[no_mangle]
unsafe extern "C" fn uart_irq_handler() {
    let mut uart = UartType::new();
    let char = uart.getc() as char;
    writeln!(uart, "Received character: {}", char).ok();
}

#[inline(never)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    let mut console = Uart::<Uart1>::new();

    compiler_fence(SeqCst);
    writeln!(console, "{}", _info).ok();

    loop {
        unsafe {
            core::arch::asm!("bkpt");
        }
    }
}
