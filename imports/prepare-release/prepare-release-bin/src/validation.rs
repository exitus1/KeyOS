// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use {regex::Regex, std::env};

/// Validate EXTRA_ENTROPY environment variable
pub fn validate_extra_entropy() -> Result<(), Error> {
    let extra_entropy = env::var("EXTRA_ENTROPY").map_err(|_| Error::ExtraEntropyMissing)?;

    // Validate format (must be 64 hex characters)
    let hex_regex = Regex::new(r"^[0-9a-fA-F]{64}$").unwrap();
    if !hex_regex.is_match(&extra_entropy) {
        return Err(Error::ExtraEntropyInvalid { value: extra_entropy.clone(), length: extra_entropy.len() });
    }

    Ok(())
}

#[derive(Debug)]
pub enum Error {
    ExtraEntropyMissing,
    ExtraEntropyInvalid { value: String, length: usize },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ExtraEntropyMissing => {
                write!(
                    f,
                    "EXTRA_ENTROPY environment variable is not set\n\
                          EXTRA_ENTROPY must be a 32-byte (64 character) hex string"
                )
            }
            Error::ExtraEntropyInvalid { value, length } => {
                write!(
                    f,
                    "EXTRA_ENTROPY must be exactly 64 hexadecimal characters (32 bytes)\n\
                          Current EXTRA_ENTROPY: '{}'\n\
                          Length: {} characters",
                    value, length
                )
            }
        }
    }
}

impl std::error::Error for Error {}
