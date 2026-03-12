// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SecretsGenError {
    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Unsupported key type: {0}")]
    UnsupportedKeyType(String),

    #[error("Unsupported key format: {0}")]
    UnsupportedKeyFormat(String),

    #[error("File injection error: {0}")]
    FileInjectionError(String),

    #[error("Unsupported file type: {0}")]
    UnsupportedFileType(String),

    #[error("Invalid naming convention: {0}")]
    InvalidNamingConvention(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("TOML error: {0}")]
    TomlError(#[from] toml::de::Error),

    #[error("Regex error: {0}")]
    RegexError(#[from] regex::Error),
}
