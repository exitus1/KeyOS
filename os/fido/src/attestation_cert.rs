// SPDX-FileCopyrightText: 2026 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Self-signed X.509 v3 attestation certificate generation for FIDO.
//!
//! This module generates an attestation certificate at runtime when the SE
//! contains a non-official FIDO key (e.g., for development or testing).

use const_oid::db::rfc4519::CN;
use p256::ecdsa::signature::SignatureEncoding;
use sha2::Digest;
use x509_cert::{
    certificate::{Certificate, TbsCertificate, Version},
    der::{
        asn1::{Any, BitString, Utf8StringRef},
        DateTime, Encode,
    },
    name::Name,
    serial_number::SerialNumber,
    spki::{AlgorithmIdentifierOwned, SubjectPublicKeyInfoOwned},
    time::{Time, Validity},
};

use crate::error::FidoError;

/// OID for ecdsa-with-SHA256 (1.2.840.10045.4.3.2)
const ECDSA_WITH_SHA256: const_oid::ObjectIdentifier =
    const_oid::ObjectIdentifier::new_unwrap("1.2.840.10045.4.3.2");

/// OID for ecPublicKey (1.2.840.10045.2.1)
const EC_PUBLIC_KEY: const_oid::ObjectIdentifier =
    const_oid::ObjectIdentifier::new_unwrap("1.2.840.10045.2.1");

/// OID for P-256 curve (1.2.840.10045.3.1.7)
const SECP256R1: const_oid::ObjectIdentifier = const_oid::ObjectIdentifier::new_unwrap("1.2.840.10045.3.1.7");

fn generate_serial_number(pubkey_64: &[u8; 64]) -> Result<SerialNumber, FidoError> {
    let serial_hash = sha2::Sha256::digest(pubkey_64);
    let mut serial_bytes = serial_hash[..20].to_vec();
    serial_bytes[0] &= 0x7F; // Ensure positive
    if serial_bytes.iter().all(|&b| b == 0) {
        serial_bytes[19] = 1; // Ensure non-zero
    }
    SerialNumber::new(&serial_bytes).map_err(|_| FidoError::Other)
}

fn build_name() -> Result<Name, FidoError> {
    use x509_cert::{attr::AttributeTypeAndValue, name::RelativeDistinguishedName};

    let cn_value = Utf8StringRef::new("Foundation Devices FIDO Attestation").map_err(|_| FidoError::Other)?;
    let atv = AttributeTypeAndValue { oid: CN, value: cn_value.into() };
    let rdn = RelativeDistinguishedName::try_from(vec![atv]).map_err(|_| FidoError::Other)?;
    Ok(vec![rdn].into())
}

fn build_validity() -> Result<Validity, FidoError> {
    let not_before_dt = DateTime::new(2026, 1, 1, 0, 0, 0).map_err(|_| FidoError::Other)?;
    let not_after_dt = DateTime::new(2036, 1, 1, 0, 0, 0).map_err(|_| FidoError::Other)?;

    let not_before = Time::UtcTime(not_before_dt.try_into().map_err(|_| FidoError::Other)?);
    let not_after = Time::UtcTime(not_after_dt.try_into().map_err(|_| FidoError::Other)?);

    Ok(Validity { not_before, not_after })
}

fn build_spki(pubkey_64: &[u8; 64]) -> Result<SubjectPublicKeyInfoOwned, FidoError> {
    let algorithm = AlgorithmIdentifierOwned { oid: EC_PUBLIC_KEY, parameters: Some(Any::from(&SECP256R1)) };

    let mut pubkey_uncompressed = vec![0x04];
    pubkey_uncompressed.extend_from_slice(pubkey_64);
    let subject_public_key = BitString::from_bytes(&pubkey_uncompressed).map_err(|_| FidoError::Other)?;

    Ok(SubjectPublicKeyInfoOwned { algorithm, subject_public_key })
}

fn signature_algorithm() -> AlgorithmIdentifierOwned {
    AlgorithmIdentifierOwned { oid: ECDSA_WITH_SHA256, parameters: None }
}

fn raw_sig_to_der(sig: &[u8; 64]) -> Result<Vec<u8>, FidoError> {
    let signature = p256::ecdsa::Signature::from_bytes(sig.into()).map_err(|_| FidoError::Ecdsa)?;
    Ok(signature.to_der().to_vec())
}

/// Builds a self-signed X.509 v3 attestation certificate.
///
/// # Arguments
/// * `pubkey_64` - The 64-byte raw public key (without 0x04 prefix)
/// * `sign_fn` - A callback that signs a SHA-256 hash and returns 64-byte raw signature
///
/// # Returns
/// DER-encoded X.509 certificate
pub fn build_attestation_certificate(
    pubkey_64: &[u8; 64],
    sign_fn: impl FnOnce([u8; 32]) -> Result<[u8; 64], FidoError>,
) -> Result<Vec<u8>, FidoError> {
    let name = build_name()?;

    let tbs_certificate = TbsCertificate {
        version: Version::V3,
        serial_number: generate_serial_number(pubkey_64)?,
        signature: signature_algorithm(),
        issuer: name.clone(),
        validity: build_validity()?,
        subject: name,
        subject_public_key_info: build_spki(pubkey_64)?,
        issuer_unique_id: None,
        subject_unique_id: None,
        extensions: None,
    };

    let tbs_der = tbs_certificate.to_der().map_err(|_| FidoError::Other)?;
    let tbs_hash: [u8; 32] = sha2::Sha256::digest(&tbs_der).into();

    let raw_signature = sign_fn(tbs_hash)?;
    let sig_der = raw_sig_to_der(&raw_signature)?;
    let signature = BitString::from_bytes(&sig_der).map_err(|_| FidoError::Other)?;

    let certificate = Certificate { tbs_certificate, signature_algorithm: signature_algorithm(), signature };

    certificate.to_der().map_err(|_| FidoError::Other)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_attestation_certificate() {
        // Test public key (64 bytes) - using the official pubkey for testing
        let pubkey_64: [u8; 64] = [
            0xa8, 0x61, 0xfe, 0xad, 0x21, 0xc2, 0xdc, 0x3e, 0xe9, 0x81, 0xb2, 0xbc, 0x27, 0x91, 0x33, 0x23,
            0x83, 0xf0, 0x9e, 0xe6, 0xce, 0x9f, 0x1e, 0x25, 0x00, 0x34, 0x46, 0x2c, 0xac, 0x12, 0xae, 0xfa,
            0x03, 0x26, 0xff, 0xc2, 0x3d, 0x2a, 0xf0, 0xe2, 0xe8, 0x87, 0xff, 0xf9, 0x05, 0x93, 0x08, 0xa7,
            0x7f, 0x10, 0x69, 0x70, 0x5d, 0xaf, 0x41, 0x4d, 0xb2, 0xb0, 0x6d, 0xcd, 0x35, 0x77, 0xc3, 0x58,
        ];

        // Use the dev attestation private key for testing
        let fido_private_key = security::DEV_FIDO_ATTESTATION_PRIVATE_KEY;
        let secret_key = p256::SecretKey::from_slice(&fido_private_key).expect("Failed to parse private key");
        let signing_key = p256::ecdsa::SigningKey::from(&secret_key);

        let cert = build_attestation_certificate(&pubkey_64, |hash| {
            use p256::ecdsa::signature::hazmat::PrehashSigner;
            let sig: p256::ecdsa::Signature = signing_key.sign_prehash(&hash).unwrap();
            let sig_bytes = sig.to_bytes();
            Ok(sig_bytes.into())
        })
        .expect("Failed to build certificate");

        // Basic validation: should start with SEQUENCE tag
        assert_eq!(cert[0], 0x30);
        // Should be a reasonable size for an X.509 certificate
        assert!(cert.len() > 200 && cert.len() < 1000);

        println!("Generated certificate ({} bytes):", cert.len());
        for (i, chunk) in cert.chunks(16).enumerate() {
            print!("{:04x}: ", i * 16);
            for b in chunk {
                print!("{:02x} ", b);
            }
            println!();
        }
    }
}
