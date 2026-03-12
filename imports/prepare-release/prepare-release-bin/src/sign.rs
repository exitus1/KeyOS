// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Bootloader signing using SAM-BA cipher.

use std::{
    env, fs,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

const BOOT_BIN_PATH: &str = "target/armv7a-unknown-xous-elf/release/images/boot.bin";
const BOOT_CIP_PATH: &str = "target/armv7a-unknown-xous-elf/release/images/boot.cip";

/// Sign the bootloader using SAM-BA cipher.
pub fn sign_bootloader(secrets_dir: &Path, stdout: &mut impl Write) -> Result<(), Error> {
    writeln!(stdout, "Signing bootloader with SAM-BA cipher...").map_err(Error::Stdout)?;

    // Expand ~ in secrets_dir if needed
    let secrets_dir = expand_tilde(secrets_dir);

    // Validate secrets directory exists
    if !secrets_dir.exists() {
        return Err(Error::SecretsDirNotFound(secrets_dir));
    }

    // Check required secret files
    let activation_file = secrets_dir.join("sam-ba-license-activation.txt");
    let cust_key_file = secrets_dir.join("cust.key");

    if !activation_file.exists() {
        return Err(Error::SecretFileNotFound(activation_file));
    }
    if !cust_key_file.exists() {
        return Err(Error::SecretFileNotFound(cust_key_file));
    }

    // Get SAM-BA cipher path from environment
    let samba_path = env::var("SECURE_SAMBA_CIPHER_PATH").map_err(|_| Error::SambaCipherPathNotSet)?;
    let samba_path = expand_tilde(&PathBuf::from(samba_path));

    if !samba_path.exists() {
        return Err(Error::SambaCipherNotFound(samba_path));
    }

    // Find Python interpreter
    let python = find_python(&samba_path)?;
    writeln!(stdout, "Using Python interpreter: {}", python.display()).map_err(Error::Stdout)?;

    // Verify boot.bin exists
    let boot_bin = PathBuf::from(BOOT_BIN_PATH);
    if !boot_bin.exists() {
        return Err(Error::BootBinNotFound(boot_bin));
    }

    let images_dir = boot_bin.parent().unwrap();
    writeln!(stdout, "Signing {} -> boot.cip", boot_bin.display()).map_err(Error::Stdout)?;

    // Run SAM-BA cipher from the images directory with simple filenames
    let status = Command::new(&python)
        .args([
            samba_path.to_str().unwrap(),
            "bootstrap",
            "-d",
            "sama5d2x",
            "-l",
            activation_file.to_str().unwrap(),
            "-k",
            cust_key_file.to_str().unwrap(),
            "-i",
            "boot.bin",
            "-o",
            "boot.cip",
        ])
        .current_dir(images_dir)
        .env("PYTHONUNBUFFERED", "1")
        .status()
        .map_err(Error::CommandFailed)?;

    if !status.success() {
        return Err(Error::SigningFailed);
    }

    // Handle device-specific output naming (boot_sama5d2x.cip -> boot.cip)
    // SAM-BA cipher appends device name to output, so we always rename it
    let boot_cip = PathBuf::from(BOOT_CIP_PATH);
    let device_cip = images_dir.join("boot_sama5d2x.cip");
    if !device_cip.exists() {
        return Err(Error::BootCipNotCreated);
    }
    fs::rename(&device_cip, &boot_cip).map_err(Error::RenameFile)?;

    writeln!(stdout, "✅ boot.cip created: {}", boot_cip.display()).map_err(Error::Stdout)?;

    // Security: Remove boot.bin (contains unencrypted EXTRA_ENTROPY)
    if boot_bin.exists() {
        fs::remove_file(&boot_bin).map_err(Error::RemoveFile)?;
        writeln!(stdout, "🗑️  Removed boot.bin (contains unencrypted secrets)").map_err(Error::Stdout)?;
    }

    Ok(())
}

fn expand_tilde(path: &Path) -> PathBuf {
    if let Some(path_str) = path.to_str() {
        if let Some(rest) = path_str.strip_prefix("~/") {
            if let Ok(home) = env::var("HOME") {
                return PathBuf::from(home).join(rest);
            }
        }
    }
    path.to_path_buf()
}

fn find_python(samba_path: &Path) -> Result<PathBuf, Error> {
    // Check SECURE_SAMBA_PYTHON env var first
    if let Ok(py) = env::var("SECURE_SAMBA_PYTHON") {
        let py_path = expand_tilde(&PathBuf::from(py));
        if py_path.exists() {
            return Ok(py_path);
        }
    }

    // Try venv near the SAM-BA tool
    let samba_dir = samba_path.parent().unwrap_or(Path::new("."));
    for venv_name in &["venv", ".venv"] {
        let venv_python = samba_dir.join(venv_name).join("bin/python");
        if venv_python.exists() {
            return Ok(venv_python);
        }
    }

    // Fall back to system python3
    find_in_path("python3").ok_or(Error::PythonNotFound)
}

fn find_in_path(executable: &str) -> Option<PathBuf> {
    let path_var = env::var_os("PATH")?;
    for dir in env::split_paths(&path_var) {
        let candidate = dir.join(executable);
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

#[derive(Debug)]
pub enum Error {
    SecretsDirNotFound(PathBuf),
    SecretFileNotFound(PathBuf),
    SambaCipherPathNotSet,
    SambaCipherNotFound(PathBuf),
    PythonNotFound,
    BootBinNotFound(PathBuf),
    CommandFailed(std::io::Error),
    SigningFailed,
    RenameFile(std::io::Error),
    RemoveFile(std::io::Error),
    BootCipNotCreated,
    Stdout(std::io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::SecretsDirNotFound(path) => {
                write!(f, "secrets directory not found: {}", path.display())
            }
            Error::SecretFileNotFound(path) => {
                write!(f, "required secret file not found: {}", path.display())
            }
            Error::SambaCipherPathNotSet => {
                write!(
                    f,
                    "SECURE_SAMBA_CIPHER_PATH environment variable is not set.\n\
                     Set it to the path of secure-sam-ba-cipher.py"
                )
            }
            Error::SambaCipherNotFound(path) => {
                write!(f, "SAM-BA cipher tool not found at: {}", path.display())
            }
            Error::PythonNotFound => {
                write!(
                    f,
                    "Python interpreter not found.\n\
                     Set SECURE_SAMBA_PYTHON or ensure python3 is in PATH"
                )
            }
            Error::BootBinNotFound(path) => {
                write!(
                    f,
                    "boot.bin not found at: {}\n\
                     Build the bootloader first.",
                    path.display()
                )
            }
            Error::CommandFailed(e) => {
                write!(f, "failed to execute SAM-BA cipher: {}", e)
            }
            Error::SigningFailed => {
                write!(f, "SAM-BA cipher signing failed")
            }
            Error::RenameFile(e) => {
                write!(f, "failed to rename output file: {}", e)
            }
            Error::RemoveFile(e) => {
                write!(f, "failed to remove boot.bin: {}", e)
            }
            Error::BootCipNotCreated => {
                write!(f, "boot.cip was not created by SAM-BA cipher")
            }
            Error::Stdout(e) => {
                write!(f, "failed to write to stdout: {}", e)
            }
        }
    }
}

impl std::error::Error for Error {}
