// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

mod config;
mod error;
mod key_generation;
mod injectors;
mod naming;

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;

/// A flexible, configuration-driven utility for generating cryptographic keys and injecting them into various file types
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the configuration file
    #[arg(short, long, default_value = "secrets-config.toml")]
    config: PathBuf,

    /// Generate keys and sample config files without updating actual config files
    #[arg(long)]
    dry_run: bool,

    /// Directory to save generated keys
    #[arg(short, long)]
    output_dir: Option<PathBuf>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Parse configuration file
    let config_path = expand_tilde(&cli.config);
    let config = config::parse_config(&config_path)
        .with_context(|| format!("Failed to parse configuration file: {}", config_path.display()))?;

    // Set output directory
    let output_dir = if let Some(dir) = &cli.output_dir {
        expand_tilde(dir)
    } else {
        PathBuf::from("generated-keys")
    };

    // Ensure output directory exists
    std::fs::create_dir_all(&output_dir)
        .with_context(|| format!("Failed to create output directory: {}", output_dir.display()))?;

    // Generate keys
    println!("Generating keys...");
    let keys = key_generation::generate_keys(&config.keys, &output_dir)
        .context("Failed to generate keys")?;

    // Inject keys into files
    if cli.dry_run {
        println!("Running in dry-run mode. Files will not be modified.");
        println!("Keys have been generated and saved to: {}", output_dir.display());
    } else {
        println!("Injecting keys into files...");
        injectors::inject_keys(&config.injectors, &keys, &output_dir)
            .context("Failed to inject keys into files")?;
        println!("Keys have been generated and injected into files.");
    }

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
