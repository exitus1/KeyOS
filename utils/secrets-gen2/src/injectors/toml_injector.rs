// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::config::InjectionConfig;
use crate::error::SecretsGenError;
use crate::key_generation::Key;
use crate::naming;

/// TOML file injector
pub struct TomlInjector;

impl super::FileInjector for TomlInjector {
    fn inject(
        &self,
        file_path: &Path,
        injections: &[InjectionConfig],
        keys: &HashMap<String, Box<dyn Key>>,
        dry_run: bool,
    ) -> Result<()> {
        // Read the existing file
        let content = match fs::read_to_string(file_path) {
            Ok(content) => content,
            Err(_) => {
                // If the file doesn't exist, create a new empty one
                String::new()
            }
        };

        // Parse the TOML content
        let mut parsed_toml: toml::Table = toml::from_str(&content)
            .with_context(|| format!("Failed to parse TOML file: {}", file_path.display()))?;

        // Process each injection
        for injection in injections {
            // Get the key
            let key_name = &injection.key;
            let key = keys.get(key_name)
                .ok_or_else(|| SecretsGenError::ConfigError(format!("Key not found: {}", key_name)))?;

            // Get the format
            let format = injection.format.as_deref().unwrap_or("hex");

            // Format the key
            let key_value = key.format(format)
                .with_context(|| format!("Failed to format key: {}", key_name))?;

            // Get the name to use in the file
            let name = naming::get_name_for_key(key_name, injection.name.as_deref(), file_path)
                .with_context(|| format!("Failed to get name for key: {}", key_name))?;

            // Update the TOML
            parsed_toml.insert(name, toml::Value::String(key_value));
        }

        // Convert back to TOML string
        let updated_content = toml::to_string(&parsed_toml)
            .context("Failed to convert TOML to string")?;

        // Write back to the file
        if !dry_run {
            // Ensure parent directory exists
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
            }

            fs::write(file_path, updated_content)
                .with_context(|| format!("Failed to write to file: {}", file_path.display()))?;
        }

        Ok(())
    }
}
