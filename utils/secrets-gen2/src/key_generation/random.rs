// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::{Context, Result};
use base64::Engine;
use rand::{rngs::OsRng, RngCore};

use crate::error::SecretsGenError;

/// Random key generator
pub struct RandomKeyGenerator;

impl super::KeyGenerator for RandomKeyGenerator {
    fn generate(&self, params: Option<&toml::Value>) -> Result<Box<dyn super::Key>> {
        // Get the length in bytes
        let length = if let Some(params) = params {
            if let Some(length) = params.get("length") {
                length.as_integer().context("length parameter must be an integer")? as usize
            } else {
                32 // Default to 32 bytes
            }
        } else {
            32 // Default to 32 bytes
        };

        // Get the count
        let count = if let Some(params) = params {
            if let Some(count) = params.get("count") {
                count.as_integer().context("count parameter must be an integer")? as usize
            } else {
                1 // Default to 1
            }
        } else {
            1 // Default to 1
        };

        // Generate the random bytes
        let random = RandomBytes::generate(length, count)?;
        Ok(Box::new(random))
    }
}

/// Random bytes
#[derive(Debug)]
pub struct RandomBytes {
    bytes: Vec<Vec<u8>>,
}

impl RandomBytes {
    /// Generate random bytes
    pub fn generate(length: usize, count: usize) -> Result<Self> {
        let mut bytes = Vec::with_capacity(count);
        for _ in 0..count {
            let mut value = vec![0u8; length];
            OsRng.fill_bytes(&mut value);
            bytes.push(value);
        }
        Ok(Self { bytes })
    }
}

impl super::Key for RandomBytes {
    fn format(&self, format: &str) -> Result<String> {
        match format {
            "hex" => {
                if self.bytes.len() == 1 {
                    Ok(hex::encode(&self.bytes[0]))
                } else {
                    Err(SecretsGenError::ConfigError(
                        "Cannot format multiple random values as hex. Use csv format instead.".to_string(),
                    )
                    .into())
                }
            }
            "base64" => {
                if self.bytes.len() == 1 {
                    Ok(base64::engine::general_purpose::STANDARD.encode(&self.bytes[0]))
                } else {
                    Err(SecretsGenError::ConfigError(
                        "Cannot format multiple random values as base64. Use csv format instead.".to_string(),
                    )
                    .into())
                }
            }
            "csv" => {
                let hex_values: Vec<String> = self.bytes.iter().map(hex::encode).collect();
                Ok(hex_values.join(","))
            }
            "hex-array" => {
                if self.bytes.len() == 1 {
                    let hex_bytes: Vec<String> =
                        self.bytes[0].iter().map(|b| format!("0x{:02x}", b)).collect();
                    Ok(format!("[{}]", hex_bytes.join(", ")))
                } else {
                    Err(SecretsGenError::ConfigError(
                        "Cannot format multiple random values as hex-array.".to_string(),
                    )
                    .into())
                }
            }
            "string-array" => {
                // Format multiple random values as a Rust string array
                let hex_strings: Vec<String> =
                    self.bytes.iter().map(|bytes| format!("\"{}\"", hex::encode(bytes))).collect();
                Ok(format!("[{}]", hex_strings.join(", ")))
            }
            "hex-value-array" => {
                // Format multiple random values as a Rust array of hex values with 0x prefix
                let hex_values: Vec<String> =
                    self.bytes.iter().map(|bytes| format!("0x{}", hex::encode(bytes))).collect();
                Ok(format!("[{}]", hex_values.join(", ")))
            }
            _ => Err(SecretsGenError::UnsupportedKeyFormat(format.to_string()).into()),
        }
    }
}
