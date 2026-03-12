// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::str::FromStr;

use anyhow::Context;
use ngwallet::{
    bdk_wallet::bitcoin::{bip32::Fingerprint, Network as NgNetwork},
    config::MultiSigDetails,
};

// Absolutely do not change format after release, unit test heavily to ensure parameter formats don't
// change If it must change, make sure to migrate existing account directories, table entries,
// and account_ids accordingly
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AccountId {
    Single { fingerprint: Fingerprint, network: NgNetwork, index: u32 },
    Multi { policy_threshold: usize, policy_total_keys: usize, network: NgNetwork, sha256_hash: [u8; 16] },
}

impl AccountId {
    pub fn new_single(fingerprint: Fingerprint, network: NgNetwork, index: u32) -> Self {
        Self::Single { fingerprint, network, index }
    }

    pub fn new_multi(multisig: &MultiSigDetails, network: NgNetwork) -> Self {
        let hash = multisig.sha256();
        let mut result = [0; 16];
        result.copy_from_slice(&hash[..16]);

        Self::Multi {
            policy_threshold: multisig.policy_threshold,
            policy_total_keys: multisig.policy_threshold,
            network,
            sha256_hash: result,
        }
    }

    pub fn fingerprint(&self) -> Option<&Fingerprint> {
        match self {
            AccountId::Single { fingerprint, .. } => Some(fingerprint),
            AccountId::Multi { .. } => None,
        }
    }

    pub fn index(&self) -> Option<u32> {
        match self {
            AccountId::Single { index, .. } => Some(*index),
            AccountId::Multi { .. } => None,
        }
    }

    pub fn is_multi(&self) -> bool {
        match self {
            AccountId::Single { .. } => false,
            AccountId::Multi { .. } => true,
        }
    }
}

impl FromStr for AccountId {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        fn parse_network(network: &str) -> anyhow::Result<NgNetwork> {
            match network {
                "bitcoin" => Ok(NgNetwork::Bitcoin),
                "testnet4" => Ok(NgNetwork::Testnet4),
                _ => anyhow::bail!("Invalid network: {}", network),
            }
        }

        let parts: Vec<&str> = value.split('-').collect();

        match parts.as_slice() {
            ["single", fingerprint, network, index] => {
                let fingerprint = fingerprint.parse::<Fingerprint>().context("Invalid fingerprint")?;
                let network = parse_network(network)?;
                let index = index.parse::<u32>().context("Invalid index")?;

                Ok(AccountId::Single { fingerprint, network, index })
            }
            ["multi", threshold, total, network, hash] => {
                let policy_threshold = threshold.parse::<usize>().context("Invalid threshold")?;
                let policy_total_keys = total.parse::<usize>().context("Invalid total keys")?;
                let network = parse_network(network)?;
                let sha256_hash = hex::decode(hash)
                    .context("Invalid hex hash")?
                    .try_into()
                    .map_err(|_| anyhow::anyhow!("Hash must be 32 bytes"))?;

                Ok(AccountId::Multi { policy_threshold, policy_total_keys, network, sha256_hash })
            }
            _ => anyhow::bail!("Invalid AccountId format"),
        }
    }
}

impl std::fmt::Display for AccountId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn network_to_str(network: &NgNetwork) -> &'static str {
            match network {
                NgNetwork::Bitcoin => "bitcoin",
                NgNetwork::Testnet4 => "testnet4",
                _ => "testnet4",
            }
        }

        match self {
            AccountId::Single { fingerprint, network, index } => {
                write!(f, "single-{}-{}-{}", fingerprint, network_to_str(network), index)
            }
            AccountId::Multi { policy_threshold, policy_total_keys, network, sha256_hash } => {
                write!(
                    f,
                    "multi-{}-{}-{}-{}",
                    policy_threshold,
                    policy_total_keys,
                    network_to_str(network),
                    hex::encode(sha256_hash)
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accountid_single_roundtrip() {
        let fingerprint = "12345678".parse::<Fingerprint>().unwrap();
        let original = AccountId::Single { fingerprint, network: NgNetwork::Bitcoin, index: 42 };

        let serialized = original.to_string();
        assert_eq!(serialized, "single-12345678-bitcoin-42");

        let deserialized = AccountId::from_str(&serialized).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_accountid_multi_roundtrip() {
        let hash = [1u8; 16];
        let original = AccountId::Multi {
            policy_threshold: 2,
            policy_total_keys: 3,
            network: NgNetwork::Testnet4,
            sha256_hash: hash,
        };

        let serialized = original.to_string();
        assert_eq!(serialized, "multi-2-3-testnet4-01010101010101010101010101010101");

        let deserialized = AccountId::from_str(&serialized).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_accountid_all_networks() {
        let fingerprint = "ABCDEF12".parse::<Fingerprint>().unwrap();

        for (network, expected) in [(NgNetwork::Bitcoin, "bitcoin"), (NgNetwork::Testnet4, "testnet4")] {
            let original = AccountId::Single { fingerprint, network, index: 0 };
            let serialized = original.to_string();
            assert_eq!(serialized, format!("single-abcdef12-{}-0", expected));

            let deserialized = AccountId::from_str(&serialized).unwrap();
            assert_eq!(original, deserialized);
        }
    }

    #[test]
    fn test_accountid_invalid_format() {
        let invalid_cases = [
            "invalid_format",
            "single:invalid_fingerprint:bitcoin:0",
            "single:12345678:invalid_network:0",
            "single:12345678:bitcoin:invalid_index",
            "multi:invalid_threshold:3:bitcoin:0101010101010101010101010101010101010101010101010101010101010101",
            "multi:2:invalid_total:bitcoin:0101010101010101010101010101010101010101010101010101010101010101",
            "multi:2:3:invalid_network:0101010101010101010101010101010101010101010101010101010101010101",
            "multi:2:3:bitcoin:invalid_hex",
            "multi:2:3:bitcoin:0101", // too short
        ];

        for case in invalid_cases {
            assert!(AccountId::from_str(case).is_err(), "Expected error for: {}", case);
        }
    }
}
