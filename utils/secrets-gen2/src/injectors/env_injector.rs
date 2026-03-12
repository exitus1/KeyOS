// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::config::InjectionConfig;
use crate::error::SecretsGenError;
use crate::key_generation::Key;
use crate::naming;

/// .env file injector
pub struct EnvInjector;

impl super::FileInjector for EnvInjector {
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

        // Parse the .env content and preserve structure
        let mut lines = Vec::new();
        let mut keys_seen = HashSet::new();
        
        // First pass: preserve existing lines
        for line in content.lines() {
            if line.trim().is_empty() || line.starts_with('#') {
                // Keep comments and empty lines as is
                lines.push(line.to_string());
            } else if let Some(pos) = line.find('=') {
                let key = line[..pos].trim().to_string();
                
                // Mark this key as seen
                keys_seen.insert(key);
                
                // Keep the line for now, we'll update it in the second pass
                lines.push(line.to_string());
            }
        }
        
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

            // Update existing line or add new line
            if keys_seen.contains(&name) {
                // Update existing line
                for i in 0..lines.len() {
                    if let Some(pos) = lines[i].find('=') {
                        let line_key = lines[i][..pos].trim();
                        if line_key == name {
                            lines[i] = format!("{}={}", name, key_value);
                            break;
                        }
                    }
                }
            } else {
                // Add new line
                lines.push(format!("{}={}", name, key_value));
                keys_seen.insert(name);
            }
        }
        
        // Join all lines with newlines
        let updated_content = lines.join("\n") + "\n";
        
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
