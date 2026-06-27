// decred-core: a no_std-friendly Decred signing core for KeyOS apps.
//
// Scope is deliberately narrow: BIP39 entropy -> BIP32 (Decred version bytes)
// -> P2PKH addresses -> SigHashAll -> low-S ECDSA. No staking, no mixing, no
// transaction *construction* policy (Cake Wallet does that). This crate only
// turns an unsigned-tx package into a signed, network-serializable tx.
//
// EC math, HMAC/SHA/RIPEMD, BIP39 wordlist, base58 and CBOR are delegated to
// audited crates. The only Decred-specific cryptographic primitive vendored
// here is BLAKE-256 (the 14-round SHA-3 finalist Decred uses for *everything*,
// not BLAKE2/3), implemented in blake256.rs and checked against dcrd KATs.
//
// Every algorithm here was written against dcrd source and is exercised by
// reference vectors lifted from dcrd in tests/vectors.rs. Run them first:
//
//     cargo test -p decred-core

#![forbid(unsafe_code)]

pub mod address;
pub mod airgap;
pub mod blake256;
pub mod hashing;
pub mod hd;
pub mod sighash;
pub mod sign;
pub mod tx;

/// Crate-wide error. Kept small and `Copy` so it threads cheaply through the
/// signing path and maps cleanly onto on-device UI strings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// BIP39/BIP32 derivation failed (bad entropy length, invalid scalar, or a
    /// 1-in-2^127 child-key overflow). Treated as fatal; the caller should ask
    /// the user to retry rather than silently skipping an index.
    Derivation,
    /// A byte buffer could not be parsed (short read, bad varint, malformed tx).
    Parse,
    /// CBOR encoding of an airgap package failed.
    Encode,
    /// The airgap package declared a FORMAT_VERSION this build does not speak.
    UnsupportedVersion,
    /// A SignRequest referenced an input index outside the tx it carried.
    SigHashIndex,
    /// A re-derived input key did not reproduce the prev_script the host
    /// claimed we were spending. This is the anti-tamper tripwire: refuse.
    ScriptMismatch,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let s = match self {
            Error::Derivation => "key derivation failed",
            Error::Parse => "could not parse data",
            Error::Encode => "could not encode data",
            Error::UnsupportedVersion => "unsupported package version",
            Error::SigHashIndex => "input index out of range",
            Error::ScriptMismatch => "input script does not match key (refusing to sign)",
        };
        f.write_str(s)
    }
}

// Convenience re-exports so app code can `use decred_core::{...}` without
// reaching into submodules for the common path.
pub use address::{decode_p2pkh, p2pkh_from_key, p2pkh_from_pubkey, p2pkh_script};
pub use airgap::{
    decode_sign_request, encode_sign_request, sign_request, InputMeta, OutputMeta, ReviewSummary,
    SignRequest, FORMAT_VERSION,
};
pub use hd::{
    ExtPrivKey, p2pkh_address, BRANCH_EXTERNAL, BRANCH_INTERNAL, COIN_TYPE_DCR, HARDENED,
};
pub use tx::{MsgTx, OutPoint, TxIn, TxOut};
