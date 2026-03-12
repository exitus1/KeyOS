// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::error::SecretsGenError;

/// Main configuration structure
#[derive(Debug, Deserialize)]
pub struct Config {
    /// Keys to generate
    pub keys: HashMap<String, KeyConfig>,
    /// Injectors for files
    pub injectors: InjectorConfig,
}

/// Configuration for a key
#[derive(Debug, Deserialize)]
pub struct KeyConfig {
    /// Type of key to generate
    #[serde(rename = "type")]
    pub key_type: String,
    /// Optional parameters for key generation
    pub params: Option<toml::Value>,
}

/// Configuration for injectors
#[derive(Debug, Deserialize)]
pub struct InjectorConfig {
    /// Files to inject keys into
    pub files: Vec<FileConfig>,
}

/// Configuration for a file
#[derive(Debug, Deserialize)]
pub struct FileConfig {
    /// Path to the file
    pub file: String,
    /// Type of file (determined by extension if not specified)
    #[serde(rename = "type")]
    pub file_type: Option<String>,
    /// Injections to perform
    pub injections: Vec<InjectionConfig>,
}

/// Configuration for an injection
#[derive(Debug, Deserialize)]
pub struct InjectionConfig {
    /// Key to inject
    pub key: String,
    /// Optional name to use in the file
    pub name: Option<String>,
    /// Optional format for the key value
    pub format: Option<String>,
    /// Optional marker name for Rust files (used with @Inject:Begin/End markers)
    pub inject: Option<String>,
}

/// Parse a configuration file
pub fn parse_config(path: &Path) -> Result<Config> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read configuration file: {}", path.display()))?;

    let config: Config = toml::from_str(&content)
        .with_context(|| format!("Failed to parse configuration file: {}", path.display()))?;

    validate_config(&config)?;

    Ok(config)
}

/// Validate the configuration
fn validate_config(config: &Config) -> Result<()> {
    // Check that all keys in injections exist in the keys section
    for file in &config.injectors.files {
        for injection in &file.injections {
            if !config.keys.contains_key(&injection.key) {
                return Err(SecretsGenError::ConfigError(format!(
                    "Key '{}' referenced in injection for file '{}' does not exist in the keys section",
                    injection.key, file.file
                ))
                .into());
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use tempfile::NamedTempFile;

    use super::*;

    #[test]
    fn test_parse_valid_config() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            r#"
[keys]
TestKey = {{ type = "P256" }}

[injectors]
[[injectors.files]]
file = "test.toml"
injections = [
  {{ key = "TestKey", format = "private-hex" }}
]
"#
        )
        .unwrap();

        let config = parse_config(temp_file.path()).unwrap();

        assert_eq!(config.keys.len(), 1);
        assert!(config.keys.contains_key("TestKey"));
        assert_eq!(config.keys["TestKey"].key_type, "P256");

        assert_eq!(config.injectors.files.len(), 1);
        assert_eq!(config.injectors.files[0].file, "test.toml");
        assert_eq!(config.injectors.files[0].injections.len(), 1);
        assert_eq!(config.injectors.files[0].injections[0].key, "TestKey");
        assert_eq!(config.injectors.files[0].injections[0].format, Some("private-hex".to_string()));
    }

    #[test]
    fn test_parse_invalid_config() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            r#"
[keys]
TestKey = {{ type = "P256" }}

[injectors]
[[injectors.files]]
file = "test.toml"
injections = [
  {{ key = "NonExistentKey", format = "private-hex" }}
]
"#
        )
        .unwrap();

        let result = parse_config(temp_file.path());

        assert!(result.is_err());
    }
}
