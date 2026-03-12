// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{fmt::Write, fs, process};

use anyhow::{Context, Result};
use base64::Engine;
use p256::{
    ecdsa::{SigningKey, VerifyingKey},
    pkcs8::{EncodePrivateKey, EncodePublicKey},
};
use rand::rngs::OsRng;

use crate::error::SecretsGenError;

/// P256 key generator
pub struct P256KeyGenerator;

impl super::KeyGenerator for P256KeyGenerator {
    fn generate(&self, _params: Option<&toml::Value>) -> Result<Box<dyn super::Key>> {
        let keypair = P256KeyPair::generate()?;
        Ok(Box::new(keypair))
    }
}

/// P256 keypair
#[derive(Debug)]
pub struct P256KeyPair {
    private_key: SigningKey,
    public_key: VerifyingKey,
}

impl P256KeyPair {
    /// Generate a new P256 keypair
    pub fn generate() -> Result<Self> {
        let private_key = SigningKey::random(&mut OsRng);
        let public_key = VerifyingKey::from(&private_key);

        Ok(Self { private_key, public_key })
    }
}

impl super::Key for P256KeyPair {
    fn format(&self, format: &str) -> Result<String> {
        match format {
            "private-hex" => {
                let bytes = self.private_key.to_bytes();
                Ok(hex::encode(bytes))
            }
            "public-hex" => {
                let encoded_point = self.public_key.to_encoded_point(false);
                Ok(hex::encode(encoded_point.as_bytes()))
            }
            "public-compressed-hex" => {
                let encoded_point = self.public_key.to_encoded_point(true);
                Ok(hex::encode(encoded_point.as_bytes()))
            }
            "private-pem" => {
                let pem = self
                    .private_key
                    .to_pkcs8_pem(Default::default())
                    .context("Failed to convert private key to PEM")?;
                Ok(pem.to_string())
            }
            "ec-private-pem" => {
                // Use openssl to convert the PKCS#8 PEM to EC PEM
                // First, get the PKCS#8 PEM
                let pkcs8_pem = self
                    .private_key
                    .to_pkcs8_pem(Default::default())
                    .context("Failed to convert private key to PEM")?;

                // Create a temporary file for the PKCS#8 PEM
                let temp_dir = std::env::temp_dir();
                let pkcs8_path = temp_dir.join("temp_pkcs8.pem");
                std::fs::write(&pkcs8_path, pkcs8_pem.as_bytes())
                    .context("Failed to write temporary PKCS#8 PEM file")?;

                // Create a path for the EC PEM
                let ec_path = temp_dir.join("temp_ec.pem");

                // Use openssl to convert the PKCS#8 PEM to EC PEM
                let output = std::process::Command::new("openssl")
                    .args(&["ec", "-in", pkcs8_path.to_str().unwrap(), "-out", ec_path.to_str().unwrap()])
                    .output()
                    .context("Failed to execute openssl command")?;

                if !output.status.success() {
                    return Err(anyhow::anyhow!(
                        "openssl command failed: {}",
                        String::from_utf8_lossy(&output.stderr)
                    ));
                }

                // Read the EC PEM
                let ec_pem = std::fs::read_to_string(&ec_path).context("Failed to read EC PEM file")?;

                // Clean up temporary files
                let _ = std::fs::remove_file(&pkcs8_path);
                let _ = std::fs::remove_file(&ec_path);

                Ok(ec_pem)
            }
            "public-pem" => {
                let pem = self
                    .public_key
                    .to_public_key_pem(Default::default())
                    .context("Failed to convert public key to PEM")?;
                Ok(pem)
            }
            _ => Err(SecretsGenError::UnsupportedKeyFormat(format.to_string()).into()),
        }
    }
}
