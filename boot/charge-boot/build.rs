// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundationdevices.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    env,
    time::{SystemTime, UNIX_EPOCH},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // See https://reproducible-builds.org/docs/source-date-epoch/
    if env::var("SOURCE_DATE_EPOCH").is_err() {
        let epoch = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        println!("cargo:rustc-env=SOURCE_DATE_EPOCH={epoch}");
    }

    Ok(())
}
