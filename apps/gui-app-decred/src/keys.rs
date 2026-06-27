// SPDX-License-Identifier: GPL-3.0-or-later
//
// THE SEAM.
//
// This is the single most security-sensitive file in the app and the exact
// analogue of gui-app-bitcoin/src/store.rs::load_master_key. The Bitcoin app
// hands the secure element's entropy to BDK's `MasterKey::from_entropy`. BDK
// is Bitcoin-only, so we cannot. Instead we hand the same entropy to
// decred-core, which performs BIP39 -> BIP32 (Decred dprv version bytes) ->
// account/address derivation using the *same* audited secp256k1 + HMAC-SHA512
// primitives, differing from Bitcoin only where Decred actually differs
// (version bytes, BLAKE-256, sighash, tx serialization).
//
// Trust note to carry into review: because BDK can't help here, this app sees
// raw BIP39 entropy. That is a strictly larger trust surface than the Bitcoin
// app's constrained BDK path. The mitigation is that derivation/signing happen
// in-process behind the OS user-confirmation gate, the entropy is never
// persisted, and we zeroize the master key after each use.

use decred_core::hd::ExtPrivKey;
use decred_core::Error as DcrError;
use secp256k1::Secp256k1;

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
/// Mirrors store.rs::load_master_key line-for-line in intent:
///   let entropy = security.seed()?.ok_or(no seed)?;
///   MasterKey::from_entropy(secp, network, entropy.bytes(), passphrase, None)
pub fn load_master_key(
    secp: &Secp256k1<secp256k1::All>,
    security: &crate::Security,
    passphrase: &str,
) -> Result<ExtPrivKey, KeyError> {
    // GetSeed triggers the secure element + on-display user confirmation.
    let seed = security
        .seed()
        .map_err(|_| KeyError::AccessDenied)?
        .ok_or(KeyError::NoSeed)?;

    // `seed.bytes()` is BIP39 *entropy* (16 or 32 bytes), NOT the 512-bit seed.
    // decred-core expands it via BIP39 (PBKDF2-HMAC-SHA512, 2048 iters) exactly
    // like every other BIP39 wallet, so the derived keys match Cake Wallet *iff*
    // Cake Wallet's Decred wallet also uses standard BIP39 + m/44'/42'. (That
    // compatibility assumption is the one external unknown — verify address 0
    // against a Cake Wallet restore before trusting funds. See README risk #1.)
    let master = ExtPrivKey::from_entropy(seed.bytes(), passphrase).map_err(KeyError::Derive)?;
    Ok(master)
}

/// Convenience: external-branch receive address at `m/44'/42'/account'/0/index`.
pub fn receive_address(
    secp: &Secp256k1<secp256k1::All>,
    master: &ExtPrivKey,
    account: u32,
    index: u32,
) -> Result<String, KeyError> {
    let acct = master.account_key(secp, account).map_err(KeyError::Derive)?;
    let addr_key = acct
        .address_key(secp, decred_core::hd::BRANCH_EXTERNAL, index)
        .map_err(KeyError::Derive)?;
    Ok(decred_core::hd::p2pkh_address(secp, &addr_key))
}
