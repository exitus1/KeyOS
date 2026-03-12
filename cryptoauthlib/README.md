<!--
SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
SPDX-License-Identifier: GPL-3.0-or-later
-->

Rust bindings for the [Microchip CryptoAuthentication Library](https://github.com/MicrochipTech/cryptoauthlib) version 3.7.0.

To generate the Rust bindings (for example when updating the C library version), run

```sh
bindgen cryptoauthlib/lib/cryptoauthlib.h -o src/inner/bindings.rs --rust-edition 2021 --rust-target 1.90 --use-array-pointers-in-arguments --allowlist-item "(ATEC|ATCA|CALIB_|atca|calib_|hal_).*" -- -I ./cryptoauthlib/lib/ -D LIBRARY_BUILD_EN -D ATCA_USE_ATCAB_FUNCTIONS -D ATCA_NO_HEAP -D CALIB_SELFTEST_EN -D CALIB_SHA_EN -D CALIB_COUNTER_EN -D CALIB_READ_EN -D CALIB_WRITE_EN -D CALIB_LOCK_EN -D CALIB_MAC_EN -D CALIB_CHECKMAC_EN -D CALIB_NONCE_EN -D CALIB_GENDIG_EN -D CALIB_READ_ENC_EN -D CALIB_GENKEY_EN -D CALIB_SIGN_EN -D CALIB_VERIFY_STORED_EN -D CALIB_PRIVWRITE_EN -ffunction-sections -fdata-sections -fshort-enums --target=arm-none-eabi
```
