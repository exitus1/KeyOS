// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

mod env_injector;
mod raw_injector;
mod rs_injector;
mod toml_injector;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::config::{FileConfig, InjectionConfig, InjectorConfig};
use crate::error::SecretsGenError;
use crate::key_generation::Key;

/// Trait for file injectors
pub trait FileInjector {
    /// Inject keys into a file
    fn inject(
        &self,
        file_path: &Path,
        injections: &[InjectionConfig],
        keys: &HashMap<String, Box<dyn Key>>,
        dry_run: bool,
    ) -> Result<()>;
}

/// Inject keys into files based on configuration
pub fn inject_keys(
    injector_config: &InjectorConfig,
    keys: &HashMap<String, Box<dyn Key>>,
    output_dir: &Path,
) -> Result<()> {
    for file_config in &injector_config.files {
        inject_keys_into_file(file_config, keys, output_dir)
            .with_context(|| format!("Failed to inject keys into file: {}", file_config.file))?;
    }

    Ok(())
}

/// Inject keys into a single file
fn inject_keys_into_file(
    file_config: &FileConfig,
    keys: &HashMap<String, Box<dyn Key>>,
    output_dir: &Path,
) -> Result<()> {
    // Expand tilde in file path
    let file_path = expand_tilde(&PathBuf::from(&file_config.file));

    // Determine file type
    let file_type = if let Some(file_type) = &file_config.file_type {
        file_type.clone()
    } else {
        // Determine file type from extension
        if let Some(extension) = file_path.extension() {
            extension.to_string_lossy().to_string()
        } else {
            return Err(SecretsGenError::UnsupportedFileType("No file extension".to_string()).into());
        }
    };

    // Get the appropriate injector
    let injector: Box<dyn FileInjector> = match file_type.as_str() {
        "toml" => Box::new(toml_injector::TomlInjector),
        "env" => Box::new(env_injector::EnvInjector),
        "rs" => Box::new(rs_injector::RsInjector),
        "raw" => Box::new(raw_injector::RawInjector::new(output_dir)),
        _ => return Err(SecretsGenError::UnsupportedFileType(file_type).into()),
    };

    // Inject keys
    injector
        .inject(&file_path, &file_config.injections, keys, false)
        .with_context(|| format!("Failed to inject keys into file: {}", file_path.display()))?;

    Ok(())
}

/// Expand the tilde in a path to the user's home directory
fn expand_tilde(path: &PathBuf) -> PathBuf {
    if let Some(path_str) = path.to_str() {
        if path_str.starts_with("~/") {
            if let Some(home_dir) = dirs::home_dir() {
                return home_dir.join(&path_str[2..]);
            }
        }
    }
    path.clone()
}
