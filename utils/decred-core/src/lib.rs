// SPDX-License-Identifier: Apache-2.0
//! decred-core: the KeyOS adapter over the shared [`dcr_rs`] Decred library.
//!
//! All Decred consensus logic — BLAKE-256, base58check addresses, BIP32 with
//! `dprv`/`dpub` serialization, the transaction wire format, SigHashAll,
//! low-S ECDSA signature scripts and the air-gapped CBOR package (including
//! the optional `account_fp` wrong-wallet hint) — lives in dcr-rs, where it
//! is pinned by dcrd reference vectors, a real mainnet transaction, and the
//! KeyOS/Pulse golden wire bytes. The same library backs the Keystone Decred
//! signer, so every fix lands on all devices at once instead of drifting
//! across vendored copies.
//!
//! This crate is a re-export seam: KeyOS apps keep saying `decred_core::…`
//! while the implementation stays upstream. Anything KeyOS-specific (UI
//! policy like fee-warning thresholds, transport glue like UR framing) lives
//! in the app, not here — this crate must stay pure enough that a reviewer
//! can diff it against dcr-rs in seconds.
//!
//! Run the interop smoke test first:
//!
//! ```text
//! cargo test -p decred-core
//! ```

#![forbid(unsafe_code)]

pub use dcr_rs::*;
