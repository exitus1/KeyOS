// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::BTreeMap;

use crypto::messages::*;
use hmac::{Hmac, Mac};
use server::xous::{MemoryRange, PID};
use sha2::{Digest, Sha224, Sha256, Sha384, Sha512};

use crate::{CryptoError, CryptoServer, ShaAlgo};

/// Maximum number of concurrent streaming SHA contexts per process
const MAX_SHA_CONTEXTS_PER_PROCESS: usize = 4;

/// Streaming SHA context for hosted/simulator mode
enum ShaHasher {
    Sha224(Sha224),
    Sha256(Sha256),
    Sha384(Sha384),
    Sha512(Sha512),
}

struct HostedShaContext {
    hasher: ShaHasher,
    total_len: usize,
    bytes_processed: usize,
}

pub struct Inner {
    /// Streaming SHA contexts keyed by (PID, context_id)
    sha_contexts: BTreeMap<(PID, u8), HostedShaContext>,
}

impl Default for Inner {
    fn default() -> Self { Self { sha_contexts: Default::default() } }
}

impl CryptoServer {
    pub fn new() -> Self { Self(Inner::default()) }

    pub fn aes_setup(&mut self, _msg: AesSetup, _sender: PID) -> Result<usize, CryptoError> { Ok(0) }

    pub fn aes_execute(&mut self, msg: AesExecute, _sender: PID) -> Result<usize, CryptoError> {
        Ok(msg.blocks)
    }

    pub fn aes_clear(&mut self, _id: AesClear, _sender: PID) {}

    /// Initializes a streaming SHA context
    ///
    /// Returns a context ID for use with [`sha_update`](Self::sha_update) and
    /// [`sha_finalize`](Self::sha_finalize) Multiple contexts can exist concurrently; context state is
    /// preserved on each operation
    pub fn sha_init(&mut self, sender: PID, algo: ShaAlgo, total_len: usize) -> Result<u8, CryptoError> {
        // Check if this process has too many contexts
        let process_context_count = self.0.sha_contexts.keys().filter(|(pid, _)| *pid == sender).count();
        if process_context_count >= MAX_SHA_CONTEXTS_PER_PROCESS {
            return Err(CryptoError::TooManyShaContexts);
        }

        // Find an available context ID for this process (must be 0-3 due to message encoding)
        // The ShaUpdate message only uses 2 bits for context_id, so IDs must be in range 0-3
        let id = (0..MAX_SHA_CONTEXTS_PER_PROCESS as u8)
            .find(|&id| !self.0.sha_contexts.contains_key(&(sender, id)))
            .ok_or(CryptoError::TooManyShaContexts)?;

        let hasher = match algo {
            ShaAlgo::Sha224 => ShaHasher::Sha224(Sha224::new()),
            ShaAlgo::Sha256 => ShaHasher::Sha256(Sha256::new()),
            ShaAlgo::Sha384 => ShaHasher::Sha384(Sha384::new()),
            ShaAlgo::Sha512 => ShaHasher::Sha512(Sha512::new()),
        };

        self.0.sha_contexts.insert((sender, id), HostedShaContext { hasher, total_len, bytes_processed: 0 });

        Ok(id)
    }

    pub fn sha_update(
        &mut self,
        sender: PID,
        context_id: u8,
        buf: MemoryRange,
        offset: usize,
        length: usize,
    ) -> Result<usize, CryptoError> {
        let context =
            self.0.sha_contexts.get_mut(&(sender, context_id)).ok_or(CryptoError::InvalidParameter)?;

        if context.bytes_processed + length > context.total_len {
            return Err(CryptoError::InvalidDataLength);
        }

        let data = &buf.as_slice()[offset..offset + length];
        match &mut context.hasher {
            ShaHasher::Sha224(h) => h.update(data),
            ShaHasher::Sha256(h) => h.update(data),
            ShaHasher::Sha384(h) => h.update(data),
            ShaHasher::Sha512(h) => h.update(data),
        }

        context.bytes_processed += length;
        Ok(length)
    }

    pub fn sha_finalize(&mut self, sender: PID, context_id: u8) -> Result<Vec<u8>, CryptoError> {
        {
            let context =
                self.0.sha_contexts.get(&(sender, context_id)).ok_or(CryptoError::InvalidParameter)?;
            if context.bytes_processed != context.total_len {
                return Err(CryptoError::InvalidDataLength);
            }
        }

        let context = self.0.sha_contexts.remove(&(sender, context_id)).unwrap();

        let hash = match context.hasher {
            ShaHasher::Sha224(h) => h.finalize().to_vec(),
            ShaHasher::Sha256(h) => h.finalize().to_vec(),
            ShaHasher::Sha384(h) => h.finalize().to_vec(),
            ShaHasher::Sha512(h) => h.finalize().to_vec(),
        };

        Ok(hash)
    }

    /// Aborts/cleans up a streaming SHA context without finalizing
    /// Used when a context needs to be cleaned up on error paths or early returns
    pub fn sha_abort(&mut self, sender: PID, context_id: u8) {
        self.0.sha_contexts.remove(&(sender, context_id));
    }

    pub fn hmac(&self, algo: ShaAlgo, key: &[u8], msg: &[u8]) -> Result<Vec<u8>, CryptoError> {
        Ok(match algo {
            ShaAlgo::Sha224 => {
                type HmacSha224 = Hmac<Sha224>;
                let mut mac = HmacSha224::new_from_slice(key).map_err(|_| CryptoError::InvalidParameter)?;
                mac.update(msg);
                mac.finalize().into_bytes().to_vec()
            }
            ShaAlgo::Sha256 => {
                type HmacSha256 = Hmac<Sha256>;
                let mut mac = HmacSha256::new_from_slice(key).map_err(|_| CryptoError::InvalidParameter)?;
                mac.update(msg);
                mac.finalize().into_bytes().to_vec()
            }
            ShaAlgo::Sha384 => {
                type HmacSha384 = Hmac<Sha384>;
                let mut mac = HmacSha384::new_from_slice(key).map_err(|_| CryptoError::InvalidParameter)?;
                mac.update(msg);
                mac.finalize().into_bytes().to_vec()
            }
            ShaAlgo::Sha512 => {
                type HmacSha512 = Hmac<Sha512>;
                let mut mac = HmacSha512::new_from_slice(key).map_err(|_| CryptoError::InvalidParameter)?;
                mac.update(msg);
                mac.finalize().into_bytes().to_vec()
            }
        })
    }
}
