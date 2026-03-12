// SPDX-FileCopyrightText: 2020 Sean Cross <sean@xobs.io>
// SPDX-License-Identifier: Apache-2.0

// NOTE: Adapted from cortex-m/build.rs
use std::env;

use vergen_git2::{Emitter, Git2Builder};

fn main() {
    let target = env::var("TARGET").unwrap();
    let is_arm = target.starts_with("armv7a");

    // For ARM, link in the startup library.
    if is_arm {
        let kernel_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let linker_file_path = format!("{kernel_dir}/src/arch/arm/link.x");
        println!("cargo:rustc-link-arg=-T{linker_file_path}");
        println!("cargo:rerun-if-changed={linker_file_path}");
        println!("cargo:rustc-link-arg=-Map=kernel.map");
    }

    println!("cargo:rerun-if-changed=build.rs");

    // Generate the version information
    let git2 = Git2Builder::default().describe(true, true, None).sha(false).build().unwrap();

    Emitter::new().add_instructions(&git2).unwrap().emit().unwrap();
}
