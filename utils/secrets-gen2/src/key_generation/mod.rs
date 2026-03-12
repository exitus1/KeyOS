// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

mod aes;
mod p256;
mod random;
mod secp256k1;

use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};

use crate::config::KeyConfig;
use crate::error::SecretsGenError;

/// Trait for key generation
pub trait KeyGenerator {
    /// Generate a key
    fn generate(&self, params: Option<&toml::Value>) -> Result<Box<dyn Key>>;
}

/// Trait for keys
pub trait Key: std::fmt::Debug {
    /// Format the key
    fn format(&self, format: &str) -> Result<String>;
}

/// Generate keys based on configuration
pub fn generate_keys(
    key_configs: &HashMap<String, KeyConfig>,
    _output_dir: &Path,
) -> Result<HashMap<String, Box<dyn Key>>> {
    let mut keys = HashMap::new();

    for (key_name, key_config) in key_configs {
        let key =
            generate_key(key_config).with_context(|| format!("Failed to generate key: {}", key_name))?;

        keys.insert(key_name.clone(), key);
    }

    Ok(keys)
}

/// Generate a key based on configuration
fn generate_key(key_config: &KeyConfig) -> Result<Box<dyn Key>> {
    let generator: Box<dyn KeyGenerator> = match key_config.key_type.as_str() {
        "P256" => Box::new(p256::P256KeyGenerator),
        "secp256k1" => Box::new(secp256k1::Secp256k1KeyGenerator),
        "AES" => Box::new(aes::AesKeyGenerator),
        "random" => Box::new(random::RandomKeyGenerator),
        _ => return Err(SecretsGenError::UnsupportedKeyType(key_config.key_type.clone()).into()),
    };

    generator.generate(key_config.params.as_ref())
}
