// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::config::InjectionConfig;
use crate::error::SecretsGenError;
use crate::key_generation::Key;

/// Rust file injector
pub struct RsInjector;

impl super::FileInjector for RsInjector {
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
            Err(e) => {
                return Err(SecretsGenError::IoError(e).into());
            }
        };

        // Process the file
        let lines: Vec<&str> = content.lines().collect();
        let mut updated_content = String::new();

        // Create a map of marker names to key values
        let mut marker_values = HashMap::new();
        for injection in injections {
            // Get the key
            let key_name = &injection.key;
            let key = keys
                .get(key_name)
                .ok_or_else(|| SecretsGenError::ConfigError(format!("Key not found: {}", key_name)))?;

            // Get the format
            let format = injection.format.as_deref().unwrap_or("hex");

            // Format the key
            let key_value =
                key.format(format).with_context(|| format!("Failed to format key: {}", key_name))?;

            // Store the marker name and key value
            if let Some(marker) = &injection.inject {
                marker_values.insert(marker.clone(), key_value);
            }
        }

        // Process the file line by line
        let mut i = 0;
        while i < lines.len() {
            let line = lines[i];

            // Check if this line is a begin marker
            if line.trim().starts_with("// @Inject:Begin ") {
                // Extract the marker name
                let marker_name = line.trim()["// @Inject:Begin ".len()..].trim();

                // Add the begin marker line to the output
                updated_content.push_str(line);
                updated_content.push('\n');

                // Find the corresponding end marker
                let mut end_marker_index = None;
                for j in (i + 1)..lines.len() {
                    if lines[j].trim().starts_with("// @Inject:End ") {
                        let end_marker_name = lines[j].trim()["// @Inject:End ".len()..].trim();
                        if end_marker_name == marker_name {
                            end_marker_index = Some(j);
                            break;
                        }
                    }
                }

                // If we found the end marker
                if let Some(end_idx) = end_marker_index {
                    // Check if we have a value for this marker
                    if let Some(value) = marker_values.get(marker_name) {
                        // Add the key value
                        updated_content.push_str(value);
                        updated_content.push('\n');
                    }

                    // Add the end marker line
                    updated_content.push_str(lines[end_idx]);
                    updated_content.push('\n');

                    // Skip to after the end marker
                    i = end_idx + 1;
                    continue;
                } else {
                    // If we didn't find the end marker, just continue with the next line
                    i += 1;
                    continue;
                }
            }

            // If it's not a marker, just add the line as is
            updated_content.push_str(line);
            updated_content.push('\n');
            i += 1;
        }

        // Write back to the file
        if !dry_run {
            fs::write(file_path, updated_content)
                .with_context(|| format!("Failed to write to file: {}", file_path.display()))?;
        }

        Ok(())
    }
}
