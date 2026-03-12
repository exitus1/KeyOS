// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

#[cfg(keyos)]
fn main() {
    log_server::init_wait(env!("CARGO_CRATE_NAME")).unwrap();
    log::set_max_level(log::LevelFilter::Debug);
    log::info!("Starting");

    // Ensure test ELF is stripped by running `strip` on it.
    // example: arm-none-eabi-strip --strip-unneeded gui-app-example-logo
    let proc_name = "sys-benchmark";
    let elf_file = include_bytes!("../../../target/armv7a-unknown-xous-elf/release/sys-benchmark.strip");

    let len = elf_file.len();
    let mut memory =
        xous::map_memory(None, None, len.next_multiple_of(0x1000), xous::MemoryFlags::W).unwrap();

    memory.as_slice_mut()[..len].copy_from_slice(elf_file);
    let args = xous::ProcessArgs::new(proc_name, memory);

    let process = xous::create_process(args).unwrap().0;

    log::info!("Spawn successful, terminating now");
}

#[cfg(not(keyos))]
pub fn main() {}
