// SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::env;

fn main() {
    env::set_var("CC", "arm-none-eabi-gcc");
    env::set_var("CXX", "arm-none-eabi-g++");
    env::set_var("AR", "arm-none-eabi-ar");
    env::set_var("RANLIB", "arm-none-eabi-ranlib");

    let dst = cmake::Config::new("cryptoauthlib")
        .no_build_target(true) // Prevent installing in host system directories
        .define("ATCA_BUILD_SHARED_LIBS", "0") // Build CAL as a static library
        .define("ATCA_MAX_HAL_CACHE", "1") // Leave space for registering our HAL
        .define("ATCA_USE_ATCAB_FUNCTIONS", "ON")
        .define("CALIB_SELFTEST_EN", "ON")
        .define("CALIB_SHA_EN", "ON")
        .define("CALIB_COUNTER_EN", "ON")
        .define("CALIB_READ_EN", "ON")
        .define("CALIB_WRITE_EN", "ON")
        .define("CALIB_LOCK_EN", "ON")
        .define("CALIB_MAC_EN", "ON")
        .define("CALIB_CHECKMAC_EN", "ON")
        .define("CALIB_NONCE_EN", "ON")
        .define("CALIB_GENDIG_EN", "ON")
        .define("CALIB_READ_ENC_EN", "ON")
        .define("CALIB_GENKEY_EN", "ON")
        .define("CALIB_SIGN_EN", "ON")
        .define("CALIB_VERIFY_STORED_EN", "ON")
        .define("ATCAH_GENDIG", "ON")
        .define("ATCAH_NONCE", "ON")
        .define("ATCAH_PRIVWRITE_AUTH_MAC", "ON")
        .define("CALIB_PRIVWRITE_EN", "ON")
        .define("CMAKE_C_COMPILER_WORKS", "1") // Don't check compiler validity
        // Don't support unneeded devices
        .define("ATCA_ATSHA204A_SUPPORT", "OFF")
        .define("ATCA_ATSHA206A_SUPPORT", "OFF")
        .define("ATCA_ATECC108A_SUPPORT", "OFF")
        .define("ATCA_ATECC508A_SUPPORT", "OFF")
        .define("ATCA_ATECC608_SUPPORT", "ON")
        .define("ATCA_ECC204_SUPPORT", "OFF")
        .define("ATCA_TA010_SUPPORT", "OFF")
        .define("ATCA_SHA104_SUPPORT", "OFF")
        .define("ATCA_SHA105_SUPPORT", "OFF")
        //.define("ATCA_PRINTF", "ON")
        .define("ATCA_NO_HEAP", "ON")
        .target("armv7a-none-eabi")
        .build();
    // Below caller LD_FLAGS are defined. First -L then -l
    println!("cargo:rustc-link-search=native={}/build/lib", dst.display());
    println!("cargo:rustc-link-lib=static=cryptoauth");
}
