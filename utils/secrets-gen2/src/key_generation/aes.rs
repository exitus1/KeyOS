// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::{Context, Result};
use base64::Engine;
use rand::{rngs::OsRng, RngCore};

use crate::error::SecretsGenError;

/// AES key generator
pub struct AesKeyGenerator;

impl super::KeyGenerator for AesKeyGenerator {
    fn generate(&self, params: Option<&toml::Value>) -> Result<Box<dyn super::Key>> {
        // Get the key size in bits
        let bits = if let Some(params) = params {
            if let Some(bits) = params.get("bits") {
                bits.as_integer().context("bits parameter must be an integer")? as usize
            } else {
                256 // Default to 256 bits
            }
        } else {
            256 // Default to 256 bits
        };

        // Validate the key size
        match bits {
            128 | 192 | 256 => {}
            _ => {
                return Err(SecretsGenError::ConfigError(format!(
                    "Invalid AES key size: {}. Must be 128, 192, or 256 bits.",
                    bits
                ))
                .into())
            }
        }

        // Generate the key
        let key = AesKey::generate(bits / 8)?;
        Ok(Box::new(key))
    }
}

/// AES key
#[derive(Debug)]
pub struct AesKey {
    bytes: Vec<u8>,
}

impl AesKey {
    /// Generate a new AES key
    pub fn generate(length: usize) -> Result<Self> {
        let mut bytes = vec![0u8; length];
        OsRng.fill_bytes(&mut bytes);
        Ok(Self { bytes })
    }
}

impl super::Key for AesKey {
    fn format(&self, format: &str) -> Result<String> {
        match format {
            "hex" => Ok(hex::encode(&self.bytes)),
            "base64" => Ok(base64::engine::general_purpose::STANDARD.encode(&self.bytes)),
            _ => Err(SecretsGenError::UnsupportedKeyFormat(format.to_string()).into()),
        }
    }
}
