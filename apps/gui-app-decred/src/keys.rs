// SPDX-License-Identifier: GPL-3.0-or-later
//
// THE SEAM.
//
// This is the single most security-sensitive file in the app and the exact
// analogue of gui-app-bitcoin/src/store.rs::load_master_key. The Bitcoin app
// hands the secure element's entropy to BDK's `MasterKey::from_entropy`. BDK
// is Bitcoin-only, so we cannot. Instead we hand the same entropy to
// decred-core (the KeyOS adapter over the shared dcr-rs library), which
// performs BIP39 -> BIP32 (Decred dprv version bytes) -> account/address
// derivation using the *same* audited secp256k1 + HMAC-SHA512 primitives,
// differing from Bitcoin only where Decred actually differs (version bytes,
// BLAKE-256, sighash, tx serialization).
//
// Trust note to carry into review: because BDK can't help here, this app sees
// raw BIP39 entropy. That is a strictly larger trust surface than the Bitcoin
// app's constrained BDK path. The mitigations: derivation/signing happen
// in-process behind the OS user-confirmation gate, the entropy is never
// persisted, dcr-rs zeroizes the 64-byte seed and every derived ExtPrivKey on
// drop, and everything that does NOT need private material (review, receive
// addresses, watch-only export, wrong-wallet detection) runs from a cached
// account-level *public* key instead of re-touching the seed.

use decred_core::hd::{ExtPrivKey, ExtPubKey};
use decred_core::secp256k1::Secp256k1;
use decred_core::Error as DcrError;
use decred_core::Network;

/// The one network this device build signs for.
pub const NETWORK: Network = Network::Mainnet;

/// Errors surfaced from the seed seam. Kept distinct from decred_core::Error so
/// UI can tell "device refused / no seed" apart from "derivation math failed".
#[derive(Debug)]
pub enum KeyError {
    /// The secure element returned AccessDenied (user declined, or perms).
    AccessDenied,
    /// No seed is provisioned on the device.
    NoSeed,
    /// decred-core derivation failed.
    Derive(DcrError),
}

impl core::fmt::Display for KeyError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            KeyError::AccessDenied => f.write_str("seed access denied"),
            KeyError::NoSeed => f.write_str("no seed on device"),
            KeyError::Derive(e) => write!(f, "derivation failed: {e}"),
        }
    }
}

/// Load the Decred BIP32 master key from the secure element.
///
/// `security` is `crate::Security` (constructed via `Security::default()`),
/// passed in so this function stays unit-testable in shape and the caller owns
/// the user-confirmation lifetime. `passphrase` is the optional BIP39 25th word
/// (empty string = none), matching the Bitcoin app's signature.
///
/// The returned key (and every child derived from it) is scrubbed on drop by
/// dcr-rs, so callers should keep its lifetime as short as the operation
/// allows — derive, use, drop.
pub fn load_master_key(security: &crate::Security, passphrase: &str) -> Result<ExtPrivKey, KeyError> {
    // GetSeed triggers the secure element + on-display user confirmation.
    let seed = security.seed().map_err(|_| KeyError::AccessDenied)?.ok_or(KeyError::NoSeed)?;

    // `seed.bytes()` is BIP39 *entropy* (16 or 32 bytes), NOT the 512-bit seed.
    // decred-core expands it via BIP39 (PBKDF2-HMAC-SHA512, 2048 iters) exactly
    // like every other BIP39 wallet, so the derived keys match Cake Wallet *iff*
    // Cake Wallet's Decred wallet also uses standard BIP39 + m/44'/42'. (That
    // compatibility assumption is the one external unknown — verify address 0
    // against a Cake Wallet restore before trusting funds. See README risk #1.)
    let master = ExtPrivKey::from_entropy(seed.bytes(), passphrase, NETWORK).map_err(KeyError::Derive)?;
    Ok(master)
}

/// Derive the neutered account-level key at `m/44'/42'/account'`. Prompts for
/// seed access (via `load_master_key`); the private intermediates are dropped
/// — and therefore zeroized — before this returns. Callers should cache the
/// result per session (see `AppState::account_xpub`) so review, receive and
/// export never touch the seed again.
pub fn derive_account_xpub(
    secp: &Secp256k1<decred_core::secp256k1::All>,
    security: &crate::Security,
    passphrase: &str,
    account: u32,
) -> Result<ExtPubKey, KeyError> {
    let master = load_master_key(security, passphrase)?;
    let account_key = master.account_key(secp, account).map_err(KeyError::Derive)?;
    Ok(account_key.neuter(secp))
}

/// External-branch receive address at `m/44'/42'/account'/0/index`, derived
/// with public CKD only — no seed access, no private material.
pub fn receive_address(
    secp: &Secp256k1<decred_core::secp256k1::All>,
    account_xpub: &ExtPubKey,
    index: u32,
) -> Result<String, KeyError> {
    let key = account_xpub
        .derive_path(secp, &[decred_core::hd::BRANCH_EXTERNAL, index])
        .map_err(KeyError::Derive)?;
    Ok(key.p2pkh_address())
}
