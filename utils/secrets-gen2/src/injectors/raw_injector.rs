// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::config::InjectionConfig;
use crate::error::SecretsGenError;
use crate::key_generation::Key;

/// Raw file injector
pub struct RawInjector;

impl RawInjector {
    /// Create a new raw file injector
    pub fn new(_output_dir: &Path) -> Self { Self }
}

impl super::FileInjector for RawInjector {
    fn inject(
        &self,
        file_path: &Path,
        injections: &[InjectionConfig],
        keys: &HashMap<String, Box<dyn Key>>,
        dry_run: bool,
    ) -> Result<()> {
        // For raw files, we only support one injection per file
        if injections.len() != 1 {
            return Err(SecretsGenError::FileInjectionError(
                "Raw files only support one injection per file".to_string(),
            )
            .into());
        }

        let injection = &injections[0];

        // Get the key
        let key_name = &injection.key;
        let key = keys
            .get(key_name)
            .ok_or_else(|| SecretsGenError::ConfigError(format!("Key not found: {}", key_name)))?;

        // Get the format
        let format = injection.format.as_deref().unwrap_or("hex");

        // Format the key
        let key_value = key.format(format).with_context(|| format!("Failed to format key: {}", key_name))?;

        // Write to the file
        if !dry_run {
            // Ensure parent directory exists
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
            }

            fs::write(file_path, &key_value)
                .with_context(|| format!("Failed to write to file: {}", file_path.display()))?;
        }

        Ok(())
    }
}
