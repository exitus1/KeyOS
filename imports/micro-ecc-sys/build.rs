use std::cfg;
use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {

    println!("cargo:rerun-if-changed=build.rs");

    let target = env::var("TARGET")?;
    let mut builder = cc::Build::new();

    let mut builder = builder
        .flag("-std=c11")
        .file("micro-ecc/uECC.c")
        .define("uECC_SUPPORTS_secp160r1", "0")
        .define("uECC_SUPPORTS_secp192r1", "0")
        .define("uECC_SUPPORTS_secp224r1", "0")
        .define("uECC_SUPPORTS_secp256k1", "1")
        .define("uECC_SUPPORTS_secp256r1", "0");

    // turn on optimizations in release builds
    if cfg!(not(debug_assertions)) {
        builder = builder
            .define("uECC_OPTIMIZATION_LEVEL", "4")
            .flag("-fomit-frame-pointer");
    }

    // for timing the effect of umaal purposes
    if cfg!(feature = "no-umaal") {
        builder = builder.define("uECC_ARM_USE_UMAAL", "0")
    }

    if cfg!(feature = "square") {
        builder = builder.define("uECC_SQUARE_FUNC", "1")
    }

    builder.compile("micro-ecc-sys");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let bindings = bindgen::Builder::default()
        .header("micro-ecc/uECC.h")
        .clang_arg(format!("--target={}", target))
        .use_core()
        .ctypes_prefix("cty")

        .generate()
        .expect("Unable to generate bindings");

    let out_file = out_dir.join("bindings.rs");

    bindings
        .write_to_file(out_file)
        .expect("Couldn't write bindings!");

    Ok(())
}
