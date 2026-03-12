// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

pub mod error;
pub mod messages;

use server::{xous::MemoryRange, CheckedConn, CheckedPermissions, MessageAllowed};

use crate::error::{CryptoError, ShamirError};
use crate::messages::*;

pub const AES_BLOCK_SIZE: usize = 16;
pub const SHA224_HASH_SIZE: usize = 28;
pub const SHA256_HASH_SIZE: usize = 32;
pub const SHA384_HASH_SIZE: usize = 48;
pub const SHA512_HASH_SIZE: usize = 64;

/// The SHA hw needs to be fed chunks of 64, so if the buffer is not aligned, the first page will have a
/// length that's less than the input chunk size, so it will not fill all the registers of the HW. The next
/// page will stall because the HW is still waiting for the correct amount of data.
pub const SHA_DMA_ALIGNMENT: usize = 64;

#[macro_export]
macro_rules! use_api {
    () => {
        mod crypto_permissions {
            use crypto::messages::*;
            #[derive(Clone, Default, server::Permissions)]
            #[server_name = "os/crypto"]
            pub struct CryptoPermissions;
        }
        type CryptoApi = crypto::CryptoApi<crypto_permissions::CryptoPermissions>;
    };
}

#[derive(Default)]
pub struct CryptoApi<P: CheckedPermissions> {
    conn: CheckedConn<P>,
}

pub struct AesContext<P: CheckedPermissions + MessageAllowed<AesClear>> {
    conn: CheckedConn<P>,
    id: u8,
}

/// A streaming SHA context for hashing data in chunks
pub struct ShaStreamingContext<
    P: CheckedPermissions + MessageAllowed<ShaUpdate> + MessageAllowed<ShaFinalize> + MessageAllowed<ShaAbort>,
> {
    conn: CheckedConn<P>,
    id: u8,
    algo: ShaAlgo,
}

pub type Sha256StreamingContext<P> = ShaStreamingContext<P>;

#[derive(Clone)]
pub enum AesMode<'a> {
    Ecb { key: &'a [u8] },
    Cbc { key: &'a [u8], iv: &'a [u8; 16] },
}

#[derive(Debug, Clone, Copy, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub enum Direction {
    Encrypt,
    Decrypt,
}

impl<P: CheckedPermissions> CryptoApi<P> {
    /// Set up an AES context for encryption/decryption. Creates a copy of the key in SECURAM. Caller is
    /// responsible for cleaning up their own copy of the key, e.g. by using Zeroize.
    pub fn setup_aes(&self, mode: AesMode) -> Result<AesContext<P>, CryptoError>
    where
        P: MessageAllowed<AesSetup>,
        P: MessageAllowed<AesClear>,
    {
        let mut key_buf = xous::DropDeallocate::new(
            server::xous::map_memory(None, None, 0x1000, server::xous::MemoryFlags::W).unwrap(),
        );

        let setup = match mode {
            AesMode::Ecb { key } => {
                key_buf.as_slice_mut()[..key.len()].copy_from_slice(key);
                AesSetup::Ecb { key_buf: *key_buf, key_len: key.len() }
            }

            AesMode::Cbc { key, iv } => {
                key_buf.as_slice_mut()[..key.len()].copy_from_slice(key);
                key_buf.as_slice_mut()[key.len()..key.len() + iv.len()].copy_from_slice(iv);

                AesSetup::Ecb { key_buf: *key_buf, key_len: key.len() }
            }
        };
        let result = self.conn.lend_mut(setup);
        key_buf.as_slice_mut::<u32>().fill(0);

        Ok(AesContext { id: result? as u8, conn: self.conn.clone() })
    }

    /// Encrypt/decrypt using DMA on the provided memory ranges directly.
    /// Source and destination pointer and length do not need to be page-aligned, but they need to be
    /// word-aligned.
    /// Cache on the source buffer needs to be cleaned before the operation, and invalidated
    /// on the destination after the operation.
    ///
    /// # Safety
    /// Caller has to make sure both buffers stay mapped during the duration of the operation.
    #[cfg(keyos)]
    pub unsafe fn disk_encrypt_unsafe(
        &self,
        tweak: [u8; 16],
        j: usize,
        src: MemoryRange,
        dst: MemoryRange,
        direction: Direction,
    ) -> Result<usize, CryptoError>
    where
        P: MessageAllowed<DiskEncryptUnsafe>,
    {
        self.conn.send_archive(DiskEncryptUnsafe {
            tweak,
            j,
            src: src.as_ptr() as usize,
            dst: dst.as_ptr() as usize,
            len: src.len().min(dst.len()),
            direction,
        })
    }

    /// Calculate the SHA224 hash of part of the buffer.
    /// The address and length of `buf` has to be page-aligned.
    /// Offset is the offset into the buffer where the to-be-hased data resides.
    /// Offset must be aligned to [`SHA_DMA_ALIGNMENT`] bytes
    /// Length is the length of the actual data, in bytes. It doesn't need to be aligned.
    pub fn sha224(
        &self,
        buf: MemoryRange,
        offset: usize,
        length: usize,
    ) -> Result<[u8; SHA224_HASH_SIZE], CryptoError>
    where
        P: MessageAllowed<ShaInit>,
        P: MessageAllowed<ShaUpdate>,
        P: MessageAllowed<ShaFinalize>,
        P: MessageAllowed<ShaAbort>,
    {
        let ctx = self.sha_init(ShaAlgo::Sha224, length)?;
        ctx.update(buf, offset, length)?;
        let hash = ctx.finalize()?;
        Ok(hash.try_into().unwrap())
    }

    /// Calculate the SHA256 hash of part of the buffer.
    /// The address and length of `buf` has to be page-aligned.
    /// Offset is the offset into the buffer where the to-be-hased data resides.
    /// Offset must be aligned to [`SHA_DMA_ALIGNMENT`] bytes
    /// Length is the length of the actual data, in bytes. It doesn't need to be aligned.
    pub fn sha256(
        &self,
        buf: MemoryRange,
        offset: usize,
        length: usize,
    ) -> Result<[u8; SHA256_HASH_SIZE], CryptoError>
    where
        P: MessageAllowed<ShaInit>,
        P: MessageAllowed<ShaUpdate>,
        P: MessageAllowed<ShaFinalize>,
        P: MessageAllowed<ShaAbort>,
    {
        let ctx = self.sha_init(ShaAlgo::Sha256, length)?;
        ctx.update(buf, offset, length)?;
        let hash = ctx.finalize()?;
        Ok(hash.try_into().unwrap())
    }

    /// Calculate the SHA384 hash of part of the buffer.
    /// The address and length of `buf` has to be page-aligned.
    /// Offset is the offset into the buffer where the to-be-hased data resides.
    /// Offset must be aligned to [`SHA_DMA_ALIGNMENT`] bytes
    /// Length is the length of the actual data, in bytes. It doesn't need to be aligned.
    pub fn sha384(
        &self,
        buf: MemoryRange,
        offset: usize,
        length: usize,
    ) -> Result<[u8; SHA384_HASH_SIZE], CryptoError>
    where
        P: MessageAllowed<ShaInit>,
        P: MessageAllowed<ShaUpdate>,
        P: MessageAllowed<ShaFinalize>,
        P: MessageAllowed<ShaAbort>,
    {
        let ctx = self.sha_init(ShaAlgo::Sha384, length)?;
        ctx.update(buf, offset, length)?;
        let hash = ctx.finalize()?;
        Ok(hash.try_into().unwrap())
    }

    /// Calculate the SHA512 hash of part of the buffer.
    /// The address and length of `buf` has to be page-aligned.
    /// Offset is the offset into the buffer where the to-be-hased data resides.
    /// Offset must be aligned to [`SHA_DMA_ALIGNMENT`] bytes
    /// Length is the length of the actual data, in bytes. It doesn't need to be aligned.
    pub fn sha512(
        &self,
        buf: MemoryRange,
        offset: usize,
        length: usize,
    ) -> Result<[u8; SHA512_HASH_SIZE], CryptoError>
    where
        P: MessageAllowed<ShaInit>,
        P: MessageAllowed<ShaUpdate>,
        P: MessageAllowed<ShaFinalize>,
        P: MessageAllowed<ShaAbort>,
    {
        let ctx = self.sha_init(ShaAlgo::Sha512, length)?;
        ctx.update(buf, offset, length)?;
        let hash = ctx.finalize()?;
        Ok(hash.try_into().unwrap())
    }

    /// Initialize a streaming SHA hash context for any supported algorithm.
    ///
    /// Returns a [`ShaStreamingContext`] for use with [`update`](ShaStreamingContext::update)
    /// and [`finalize`](ShaStreamingContext::finalize).
    ///
    /// `algo`: SHA algorithm to use
    /// `total_len`: the total length of all data that will be hashed
    pub fn sha_init(&self, algo: ShaAlgo, total_len: usize) -> Result<ShaStreamingContext<P>, CryptoError>
    where
        P: MessageAllowed<ShaInit>,
        P: MessageAllowed<ShaUpdate>,
        P: MessageAllowed<ShaFinalize>,
        P: MessageAllowed<ShaAbort>,
    {
        let id = self.conn.send_archive(ShaInit { algo, total_len })? as u8;
        Ok(ShaStreamingContext { conn: self.conn.clone(), id, algo })
    }

    /// Initialize a streaming SHA-256 hash context
    ///
    /// This is a convenience method equivalent to `sha_init(ShaAlgo::Sha256, total_len)`
    ///
    /// Returns a [`Sha256StreamingContext`] for use with [`update`](ShaStreamingContext::update)
    /// and [`finalize`](ShaStreamingContext::finalize).
    ///
    /// `total_len`: the total length of all data that will be hashed
    pub fn sha256_init(&self, total_len: usize) -> Result<Sha256StreamingContext<P>, CryptoError>
    where
        P: MessageAllowed<ShaInit>,
        P: MessageAllowed<ShaUpdate>,
        P: MessageAllowed<ShaFinalize>,
        P: MessageAllowed<ShaAbort>,
    {
        self.sha_init(ShaAlgo::Sha256, total_len)
    }

    /// Calculate the HMAC-224 hash of some data using the provided key.
    pub fn hmac224(&self, key: Vec<u8>, data: Vec<u8>) -> Result<Vec<u8>, CryptoError>
    where
        P: MessageAllowed<Hmac>,
    {
        self.conn.send_archive(Hmac { algo: ShaAlgo::Sha224, key, data })
    }

    /// Calculate the HMAC-256 hash of some data using the provided key.
    pub fn hmac256(&self, key: Vec<u8>, data: Vec<u8>) -> Result<Vec<u8>, CryptoError>
    where
        P: MessageAllowed<Hmac>,
    {
        self.conn.send_archive(Hmac { algo: ShaAlgo::Sha256, key, data })
    }

    /// Calculate the HMAC-384 hash of some data using the provided key.
    pub fn hmac384(&self, key: Vec<u8>, data: Vec<u8>) -> Result<Vec<u8>, CryptoError>
    where
        P: MessageAllowed<Hmac>,
    {
        self.conn.send_archive(Hmac { algo: ShaAlgo::Sha384, key, data })
    }

    /// Calculate the HMAC-512 hash of some data using the provided key.
    pub fn hmac512(&self, key: Vec<u8>, data: Vec<u8>) -> Result<Vec<u8>, CryptoError>
    where
        P: MessageAllowed<Hmac>,
    {
        self.conn.send_archive(Hmac { algo: ShaAlgo::Sha512, key, data })
    }

    /// Split a secret into shares using Shamir's Secret Sharing.
    pub fn split_secret(
        &self,
        secret: Vec<u8>,
        num_shares: usize,
        threshold: usize,
    ) -> Result<Vec<Vec<u8>>, ShamirError>
    where
        P: MessageAllowed<ShamirSplit>,
    {
        self.conn.send_archive(ShamirSplit { secret, num_shares, threshold })
    }

    /// Recover a secret from shares using Shamir's Secret Sharing.
    pub fn recover_secret(&self, indexes: Vec<usize>, shares: Vec<Vec<u8>>) -> Result<Vec<u8>, ShamirError>
    where
        P: MessageAllowed<ShamirRecover>,
    {
        self.conn.send_archive(ShamirRecover { indexes, shares })
    }
}

impl<P: CheckedPermissions + MessageAllowed<AesClear>> AesContext<P> {
    /// Encrypt/decrypt part of the buffer.
    /// The address and length of `buf` has to be page-aligned.
    /// Offset is the offset where the actual to-be-crypted data starts.
    /// Blocks is the number of AES blocks (i.e. 16 bytes)
    pub fn execute(
        &self,
        buf: MemoryRange,
        offset: usize,
        blocks: usize,
        direction: Direction,
    ) -> Result<usize, CryptoError>
    where
        P: MessageAllowed<AesExecute>,
    {
        self.conn.lend_mut(AesExecute { buf, transfer_id: self.id, blocks, direction, offset })
    }
}

impl<P: CheckedPermissions + MessageAllowed<AesClear>> Drop for AesContext<P> {
    fn drop(&mut self) { self.conn.try_send_scalar(AesClear(self.id)).ok(); }
}

impl<P> ShaStreamingContext<P>
where
    P: CheckedPermissions,
    P: MessageAllowed<ShaUpdate>,
    P: MessageAllowed<ShaFinalize>,
    P: MessageAllowed<ShaAbort>,
{
    /// Returns the hash size in bytes for this context's algorithm
    pub fn hash_size(&self) -> usize {
        match self.algo {
            ShaAlgo::Sha224 => SHA224_HASH_SIZE,
            ShaAlgo::Sha256 => SHA256_HASH_SIZE,
            ShaAlgo::Sha384 => SHA384_HASH_SIZE,
            ShaAlgo::Sha512 => SHA512_HASH_SIZE,
        }
    }

    /// Update the hash with more data
    ///
    /// The buffer must be page-aligned, and the offset must be aligned to [`SHA_DMA_ALIGNMENT`]
    /// Can be called multiple times with arbitrary data lengths
    ///
    /// `buf`: page-aligned memory buffer containing the data
    /// `offset`: offset into the buffer where data starts (must be aligned to [`SHA_DMA_ALIGNMENT`])
    /// `length`: length of data to hash in bytes
    pub fn update(&self, buf: MemoryRange, offset: usize, length: usize) -> Result<usize, CryptoError> {
        self.conn.lend_mut(ShaUpdate { context_id: self.id, buf, offset, length })
    }

    /// Finalize the hash and return the result
    /// Consumes the context
    pub fn finalize(self) -> Result<Vec<u8>, CryptoError> {
        // Prevent Drop from sending ShaAbort by forgetting self after extracting needed fields
        let conn = unsafe { core::ptr::read(&self.conn) };
        let id = self.id;
        core::mem::forget(self);
        conn.send_archive(ShaFinalize { context_id: id })
    }
}

impl<P> Drop for ShaStreamingContext<P>
where
    P: CheckedPermissions,
    P: MessageAllowed<ShaUpdate>,
    P: MessageAllowed<ShaFinalize>,
    P: MessageAllowed<ShaAbort>,
{
    fn drop(&mut self) {
        // Send abort to clean up the server-side context
        self.conn.try_send_scalar(ShaAbort(self.id)).ok();
    }
}
