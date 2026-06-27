// SPDX-License-Identifier: Apache-2.0
//! HD key derivation for Decred.
//!
//! Path: `m / 44' / 42' / account' / branch / index`
//!   * coin type 42 (SLIP-0044, Decred)
//!   * branch 0 = external (receive), 1 = internal (change)
//!
//! BIP32 master generation is identical to Bitcoin (HMAC key `"Bitcoin seed"`);
//! Decred only differs in the `dprv`/`dpub` serialization version bytes. This is
//! confirmed by the dcrd vector in `tests/vectors.rs`: BIP32 test-vector-1 seed
//! re-encodes to `dprv3hCznBesA6jBt…`.
//!
//! The KeyOS secure element hands the app BIP39 *entropy* (16 or 32 bytes); we
//! expand it to the 512-bit seed exactly as the Bitcoin app's
//! `MasterKey::from_entropy` does, so keys match any BIP39 + `m/44'/42'` wallet.

use bip39::Mnemonic;
use hmac::{Hmac, Mac};
use secp256k1::{PublicKey, Scalar, Secp256k1, SecretKey};
use sha2::Sha512;

use crate::hashing::{check_encode, hash160};
use crate::Error;

type HmacSha512 = Hmac<Sha512>;

pub const COIN_TYPE_DCR: u32 = 42;
pub const HARDENED: u32 = 0x8000_0000;
pub const BRANCH_EXTERNAL: u32 = 0;
pub const BRANCH_INTERNAL: u32 = 1;

// Mainnet extended-key version bytes.
pub const HD_PRIV_MAINNET: [u8; 4] = [0x02, 0xfd, 0xa4, 0xe8]; // dprv
pub const HD_PUB_MAINNET: [u8; 4] = [0x02, 0xfd, 0xa9, 0x26]; // dpub

#[derive(Clone)]
pub struct ExtPrivKey {
    pub secret: SecretKey,
    pub chain_code: [u8; 32],
    pub depth: u8,
    pub parent_fingerprint: [u8; 4],
    pub child_number: u32,
}

impl ExtPrivKey {
    /// BIP32 master from a 512-bit BIP39 seed.
    pub fn master_from_seed(seed: &[u8]) -> Result<Self, Error> {
        let mut mac = HmacSha512::new_from_slice(b"Bitcoin seed").expect("hmac key");
        mac.update(seed);
        let i = mac.finalize().into_bytes();
        let secret = SecretKey::from_slice(&i[..32]).map_err(|_| Error::Derivation)?;
        let mut chain_code = [0u8; 32];
        chain_code.copy_from_slice(&i[32..]);
        Ok(ExtPrivKey {
            secret,
            chain_code,
            depth: 0,
            parent_fingerprint: [0; 4],
            child_number: 0,
        })
    }

    /// Derive from BIP39 entropy (the form KeyOS `security.seed()` returns).
    pub fn from_entropy(entropy: &[u8], passphrase: &str) -> Result<Self, Error> {
        let mnemonic = Mnemonic::from_entropy(entropy).map_err(|_| Error::Derivation)?;
        let seed = mnemonic.to_seed(passphrase);
        Self::master_from_seed(&seed)
    }

    pub fn public_key(&self, secp: &Secp256k1<secp256k1::All>) -> PublicKey {
        PublicKey::from_secret_key(secp, &self.secret)
    }

    /// 33-byte compressed pubkey — the form committed in Decred addresses/scripts.
    pub fn compressed_pubkey(&self, secp: &Secp256k1<secp256k1::All>) -> [u8; 33] {
        self.public_key(secp).serialize()
    }

    pub fn fingerprint(&self, secp: &Secp256k1<secp256k1::All>) -> [u8; 4] {
        let h = hash160(&self.compressed_pubkey(secp));
        [h[0], h[1], h[2], h[3]]
    }

    /// BIP32 CKDpriv. `index >= HARDENED` performs hardened derivation.
    pub fn derive_child(
        &self,
        secp: &Secp256k1<secp256k1::All>,
        index: u32,
    ) -> Result<Self, Error> {
        let mut mac = HmacSha512::new_from_slice(&self.chain_code).expect("hmac key");
        if index >= HARDENED {
            mac.update(&[0u8]);
            mac.update(&self.secret.secret_bytes());
        } else {
            mac.update(&self.compressed_pubkey(secp));
        }
        mac.update(&index.to_be_bytes());
        let i = mac.finalize().into_bytes();

        let tweak = Scalar::from_be_bytes(
            <[u8; 32]>::try_from(&i[..32]).unwrap(),
        )
        .map_err(|_| Error::Derivation)?;
        let secret = self
            .secret
            .add_tweak(&tweak)
            .map_err(|_| Error::Derivation)?;

        let mut chain_code = [0u8; 32];
        chain_code.copy_from_slice(&i[32..]);

        Ok(ExtPrivKey {
            secret,
            chain_code,
            depth: self.depth + 1,
            parent_fingerprint: self.fingerprint(secp),
            child_number: index,
        })
    }

    pub fn derive_path(
        &self,
        secp: &Secp256k1<secp256k1::All>,
        path: &[u32],
    ) -> Result<Self, Error> {
        let mut key = self.clone();
        for &idx in path {
            key = key.derive_child(secp, idx)?;
        }
        Ok(key)
    }

    /// Account key at `m/44'/42'/account'`.
    pub fn account_key(
        &self,
        secp: &Secp256k1<secp256k1::All>,
        account: u32,
    ) -> Result<Self, Error> {
        self.derive_path(
            secp,
            &[44 | HARDENED, COIN_TYPE_DCR | HARDENED, account | HARDENED],
        )
    }

    /// Address key at `.../branch/index` relative to an account key.
    pub fn address_key(
        &self,
        secp: &Secp256k1<secp256k1::All>,
        branch: u32,
        index: u32,
    ) -> Result<Self, Error> {
        self.derive_path(secp, &[branch, index])
    }

    /// Serialize as a `dprv…` extended private key (mainnet) — used by tests
    /// and for export/debugging. Not needed for signing.
    pub fn to_dprv(&self) -> String {
        let mut data = Vec::with_capacity(78);
        data.extend_from_slice(&HD_PRIV_MAINNET);
        data.push(self.depth);
        data.extend_from_slice(&self.parent_fingerprint);
        data.extend_from_slice(&self.child_number.to_be_bytes());
        data.extend_from_slice(&self.chain_code);
        data.push(0x00);
        data.extend_from_slice(&self.secret.secret_bytes());
        // Decred extended keys are base58check with the standard 4-byte
        // double-blake256 checksum, but the version is already the first 4
        // bytes here, so we checksum the whole 78-byte body.
        let cksum = crate::blake256::sum256d(&data);
        data.extend_from_slice(&cksum[..4]);
        bs58::encode(data).into_string()
    }

    /// Serialize the NEUTERED extended public key (`dpub…`, mainnet). This is
    /// what a watch-only companion (Cake Wallet) needs to track balances and
    /// build unsigned transactions; it carries no private material. Uses the
    /// compressed public key in the key-data slot and the dpub version bytes,
    /// with the same double-BLAKE256 base58 checksum as dprv.
    pub fn to_dpub(&self, secp: &Secp256k1<secp256k1::All>) -> String {
        let mut data = Vec::with_capacity(78);
        data.extend_from_slice(&HD_PUB_MAINNET);
        data.push(self.depth);
        data.extend_from_slice(&self.parent_fingerprint);
        data.extend_from_slice(&self.child_number.to_be_bytes());
        data.extend_from_slice(&self.chain_code);
        data.extend_from_slice(&self.compressed_pubkey(secp)); // 33 bytes, no 0x00 pad
        let cksum = crate::blake256::sum256d(&data);
        data.extend_from_slice(&cksum[..4]);
        bs58::encode(data).into_string()
    }
}

/// Convenience: mainnet P2PKH address string for this key.
pub fn p2pkh_address(secp: &Secp256k1<secp256k1::All>, key: &ExtPrivKey) -> String {
    let h = hash160(&key.compressed_pubkey(secp));
    check_encode(&h, crate::address::PKH_ADDR_ID_MAINNET)
}
