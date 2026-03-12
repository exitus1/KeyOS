// SPDX-FileCopyrightText: 2020 Sean Cross <sean@xobs.io>
// SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: Apache-2.0

#![cfg(any(not(feature = "production"), feature = "log-serial"))]

use core::fmt::{self, Write};

use crate::{
    debug::commands::{debug_command, print_help},
    io::{SerialRead, SerialWrite},
};
/// Instance of the shell output.
static mut OUTPUT: Option<Output> = None;

/// Shell output.
pub struct Output {
    serial: &'static mut dyn SerialWrite,
}

impl Output {
    fn new(_serial: &'static mut dyn SerialWrite) -> Output { Output { serial: _serial } }
}

impl Write for Output {
    fn write_str(&mut self, _s: &str) -> fmt::Result {
        for c in _s.bytes() {
            self.serial.putc(c);
        }
        Ok(())
    }
}

/// Initialize the kernel shell.
///
/// This should be called in platform initialization code.
pub fn init(serial: &'static mut dyn SerialWrite) {
    let mut output = Output::new(serial);

    let banner_lines = include_str!("../../banner.txt").lines();
    for line in banner_lines {
        writeln!(output, "{}", line).ok();
    }
    writeln!(output, "KeyOS kernel {} ({})", env!("VERGEN_GIT_DESCRIBE"), env!("VERGEN_GIT_SHA")).ok();
    writeln!(output).ok();

    writeln!(output, "=== Kernel Debug Shell Available ====").ok();
    print_help(&mut output);
    writeln!(output, "=====================================").ok();
    unsafe { OUTPUT = Some(output) }
}

pub fn with_output(f: impl FnOnce(&mut Output)) {
    if let Some(stream) = unsafe { (&mut *core::ptr::addr_of_mut!(OUTPUT)).as_mut() } {
        f(stream);
    }
}

/// Process possible characters received through a serial interface.
///
/// This should be called when a serial interface has new data, for example,
/// on an interrupt.
pub fn process_characters<R: SerialRead>(serial: &mut R) {
    while let Some(b) = serial.getc() {
        with_output(|stream| {
            writeln!(stream, "> {}", b as char).ok();
            debug_command(b, stream);
        });
    }
}
