// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use crypto::{
    error::CryptoError, messages::*, Direction, AES_BLOCK_SIZE, SHA224_HASH_SIZE, SHA256_HASH_SIZE,
    SHA384_HASH_SIZE, SHA512_HASH_SIZE, SHA_DMA_ALIGNMENT,
};
use {
    atsama5d27::{
        aes::{Aes, AesMode, Iv},
        pmc::PeripheralId,
        sha::{Algorithm, Sha, Sha224, Sha256, Sha384, Sha512, ShaHwContext},
    },
    dma::error::DmaError,
    securam_manager::SecuramManager,
    server::xous::{flush_cache, syscall, CacheOperation, MemoryAddress, MemoryFlags, MemoryRange, PID},
    sha2::{Digest, Sha224 as SwSha224, Sha256 as SwSha256, Sha384 as SwSha384, Sha512 as SwSha512},
    std::collections::BTreeMap,
};

use crate::CryptoServer;

dma::use_api!();
power_manager::use_api!();

/// Maximum number of concurrent SHA contexts per process
const MAX_SHA_CONTEXTS_PER_PROCESS: usize = 4;

pub(crate) struct Inner {
    aes_contexts: BTreeMap<(PID, u8), AesContext>,
    next_context_id: u8,
    aes: Aes,
    sha: Sha,
    dma_aes_tx: DmaTransfer,
    dma_aes_rx: DmaTransfer,
    dma_sha: DmaTransfer,
    power_manager: PowerManagerApi,
    securam_manager: SecuramManager,
    securam_slot_occupied: [bool; securam_manager::NUM_SECURAM_AES_KEYS],
    sha_contexts: BTreeMap<(PID, u8), ShaContext>,
}

#[derive(Clone)]
struct ShaContext {
    bytes_processed: usize,
    hw_context: ShaHwContext,

    // Software fallback for the final unaligned chunk if the total length isn't a multiple of 4 bytes.
    // This is needed because the hardware requires 32-bit aligned data, but the final chunk may be smaller
    // and unaligned.
    sw_hasher: Option<SwHasher>,
    sw_final_hash: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
enum AesContext {
    Ecb { key_slot: usize },

    Cbc { key_slot: usize, iv: atsama5d27::aes::Iv },
}

#[derive(Clone)]
enum SwHasher {
    Sha224(SwSha224),
    Sha256(SwSha256),
    Sha384(SwSha384),
    Sha512(SwSha512),
}

impl SwHasher {
    fn new(algo: Algorithm) -> Result<Self, CryptoError> {
        match algo {
            Algorithm::Sha224 => Ok(SwHasher::Sha224(SwSha224::new())),
            Algorithm::Sha256 => Ok(SwHasher::Sha256(SwSha256::new())),
            Algorithm::Sha384 => Ok(SwHasher::Sha384(SwSha384::new())),
            Algorithm::Sha512 => Ok(SwHasher::Sha512(SwSha512::new())),
            _ => Err(CryptoError::InvalidParameter),
        }
    }

    fn update(&mut self, data: &[u8]) {
        match self {
            SwHasher::Sha224(h) => h.update(data),
            SwHasher::Sha256(h) => h.update(data),
            SwHasher::Sha384(h) => h.update(data),
            SwHasher::Sha512(h) => h.update(data),
        }
    }

    fn finalize_to_vec(self) -> Vec<u8> {
        match self {
            SwHasher::Sha224(h) => h.finalize().to_vec(),
            SwHasher::Sha256(h) => h.finalize().to_vec(),
            SwHasher::Sha384(h) => h.finalize().to_vec(),
            SwHasher::Sha512(h) => h.finalize().to_vec(),
        }
    }
}

impl CryptoServer {
    pub fn new() -> Self {
        let aes_csr = syscall::map_memory(
            MemoryAddress::new(utralib::HW_AES_BASE),
            None,
            0x1000,
            MemoryFlags::W | MemoryFlags::DEV,
        )
        .unwrap();

        let sha_csr = syscall::map_memory(
            MemoryAddress::new(utralib::HW_SHA_BASE),
            None,
            0x1000,
            MemoryFlags::W | MemoryFlags::DEV,
        )
        .unwrap();

        let securam = syscall::map_memory(
            MemoryAddress::new(utralib::HW_SECURAM_MEM),
            None,
            0x1000,
            MemoryFlags::W | MemoryFlags::DEV,
        )
        .unwrap();

        let aes = Aes::with_alt_base_addr(aes_csr.as_ptr() as u32);
        let sha = Sha::with_alt_base_addr(sha_csr.as_ptr() as u32);
        let dma = Dma::default();

        let dma_aes_tx = dma.peripheral_transfer(aes.dma_tx_addr() as _, Aes::TX_DMA_CONFIG).unwrap();
        let dma_aes_rx = dma.peripheral_transfer(aes.dma_rx_addr() as _, Aes::RX_DMA_CONFIG).unwrap();
        let dma_sha = dma.peripheral_transfer(sha.dma_in_address() as _, Sha::DMA_CONFIG).unwrap();

        Self(Inner {
            aes_contexts: Default::default(),
            next_context_id: 1,
            power_manager: Default::default(),
            aes,
            sha,
            dma_aes_tx,
            dma_aes_rx,
            dma_sha,
            securam_manager: unsafe { SecuramManager::new(securam.as_mut_ptr()).unwrap() },
            securam_slot_occupied: Default::default(),
            sha_contexts: Default::default(),
        })
    }

    pub fn aes_setup(&mut self, msg: AesSetup, sender: PID) -> Result<usize, CryptoError> {
        for _ in 0..255 {
            let id = self.0.next_context_id;
            self.0.next_context_id += 1;

            // Sorry Clippy, we use a &mut self method in the body of this if.
            #[allow(clippy::map_entry)]
            if !self.0.aes_contexts.contains_key(&(sender, id)) {
                let context = match msg {
                    AesSetup::Ecb { key_buf, key_len } => {
                        let key_slot = self.allocate_securam_slot(key_buf, key_len)?;
                        AesContext::Ecb { key_slot }
                    }
                    AesSetup::Cbc { key_buf, key_len } => {
                        let key_slot = self.allocate_securam_slot(key_buf, key_len)?;
                        let iv = Iv::try_from_slice(&key_buf.as_slice()[key_len..key_len + 32]).unwrap();
                        AesContext::Cbc { key_slot, iv }
                    }
                };

                self.0.aes_contexts.insert((sender, id), context);
                return Ok(id as usize);
            }
        }
        Err(CryptoError::TooManyAesContexts)
    }

    fn allocate_securam_slot(&mut self, key_buf: MemoryRange, key_len: usize) -> Result<usize, CryptoError> {
        let key_slot =
            self.0.securam_slot_occupied.iter().position(|s| !*s).ok_or(CryptoError::TooManySecuramKeys)?;
        if let Err(e) = self.0.securam_manager.set_aes_key(key_slot, &key_buf.as_slice()[..key_len]) {
            match e {
                securam_manager::Error::WrongKeySize => return Err(CryptoError::InvalidKeyLength),
                securam_manager::Error::MagicMismatch | securam_manager::Error::ChecksumMismatch => {
                    panic!("SECURAM is corrupted")
                }
            }
        }
        self.0.securam_slot_occupied[key_slot] = true;
        log::info!("Allocated slot {key_slot}");
        Ok(key_slot)
    }

    fn deallocate_securam_slot(&mut self, key_slot: usize) {
        log::info!("Deallocated slot {key_slot}");
        self.0.securam_manager.set_aes_key(key_slot, &[0; 32]).expect("SECURAM is corrupted");
        self.0.securam_slot_occupied[key_slot] = false;
    }

    pub fn aes_execute(&mut self, msg: AesExecute, sender: PID) -> Result<usize, CryptoError> {
        if (msg.offset + msg.blocks * AES_BLOCK_SIZE) > msg.buf.len() || msg.blocks == 0 {
            return Err(CryptoError::InvalidDataLength);
        }
        let context =
            self.0.aes_contexts.get(&(sender, msg.transfer_id)).ok_or(CryptoError::InvalidParameter)?;

        let mode = match context {
            AesContext::Ecb { key_slot } => {
                AesMode::Ecb { key: self.0.securam_manager.aes_key(*key_slot).expect("SECURAM is corrupted") }
            }
            AesContext::Cbc { key_slot, iv } => AesMode::Cbc {
                key: self.0.securam_manager.aes_key(*key_slot).expect("SECURAM is corrupted"),
                iv: iv.clone(),
            },
        };

        self.0.power_manager.enable_peripheral(PeripheralId::Aes)?;
        match msg.direction {
            Direction::Encrypt => self.0.aes.init_encrypt(mode),
            Direction::Decrypt => self.0.aes.init_decrypt(mode),
        };
        self.0.aes.setup_for_dma();

        let buf_part =
            msg.buf.subrange(msg.offset, msg.blocks * AES_BLOCK_SIZE).ok_or(CryptoError::InvalidParameter)?;
        flush_cache(buf_part, CacheOperation::CleanAndInvalidate).ok();
        unsafe {
            self.0.dma_aes_tx.execute(buf_part).map_err(convert_dma_error)?;
            self.0.dma_aes_rx.execute(buf_part).map_err(convert_dma_error)?;
        }
        self.0.dma_aes_rx.wait().map_err(convert_dma_error)?;

        self.0.power_manager.disable_peripheral(PeripheralId::Aes)?;

        Ok(msg.blocks)
    }

    // TODO (SFT-5088): If the keys are all zero, this should return an error
    pub fn disk_encrypt(&mut self, msg: DiskEncryptUnsafe, sender: PID) -> Result<usize, CryptoError> {
        if (msg.len % AES_BLOCK_SIZE) != 0 || msg.len == 0 {
            return Err(CryptoError::InvalidDataLength);
        }
        self.0.power_manager.enable_peripheral(PeripheralId::Aes)?;

        let keys = self.0.securam_manager.disk_encryption_keys().expect("SECURAM is corrupted");
        let mode = AesMode::Xts { key1: keys.0, key2: keys.1, tweak: msg.tweak, j: msg.j };

        match msg.direction {
            Direction::Encrypt => self.0.aes.init_encrypt(mode),
            Direction::Decrypt => self.0.aes.init_decrypt(mode),
        };
        self.0.aes.setup_for_dma();

        unsafe {
            self.0
                .dma_aes_tx
                .execute_for_pid(MemoryRange::new(msg.src, msg.len)?, sender)
                .map_err(convert_dma_error)?;
            self.0
                .dma_aes_rx
                .execute_for_pid(MemoryRange::new(msg.dst, msg.len)?, sender)
                .map_err(convert_dma_error)?;
        }
        self.0.dma_aes_rx.wait().map_err(convert_dma_error)?;

        self.0.power_manager.disable_peripheral(PeripheralId::Aes)?;

        Ok(msg.len)
    }

    pub fn aes_clear(&mut self, msg: AesClear, sender: PID) {
        if let Some(context) = self.0.aes_contexts.remove(&(sender, msg.0)) {
            match context {
                AesContext::Ecb { key_slot } => self.deallocate_securam_slot(key_slot),
                AesContext::Cbc { key_slot, .. } => self.deallocate_securam_slot(key_slot),
            }
        }
    }

    pub fn hmac(&self, algo: ShaAlgo, key: &[u8], msg: &[u8]) -> Result<Vec<u8>, CryptoError> {
        self.0.power_manager.enable_peripheral(PeripheralId::Sha)?;
        let hash = match algo {
            ShaAlgo::Sha224 => self.0.sha.hmac::<Sha224>(key, msg).to_vec(),
            ShaAlgo::Sha256 => self.0.sha.hmac::<Sha256>(key, msg).to_vec(),
            ShaAlgo::Sha384 => self.0.sha.hmac::<Sha384>(key, msg).to_vec(),
            ShaAlgo::Sha512 => self.0.sha.hmac::<Sha512>(key, msg).to_vec(),
        };
        self.0.power_manager.disable_peripheral(PeripheralId::Sha)?;
        Ok(hash)
    }

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

        let algo = convert_sha_algo(algo);
        let needs_sw_hasher = (total_len % 4) != 0;
        let context = ShaContext {
            bytes_processed: 0,
            hw_context: ShaHwContext::new(algo, total_len),
            sw_hasher: if needs_sw_hasher { Some(SwHasher::new(algo)?) } else { None },
            sw_final_hash: None,
        };

        self.0.sha_contexts.insert((sender, id), context);
        Ok(id)
    }

    /// Update the streaming SHA hash with more data via DMA
    /// Hardware context is restored before processing and saved after
    pub fn sha_update(
        &mut self,
        sender: PID,
        context_id: u8,
        buf: MemoryRange,
        offset: usize,
        length: usize,
    ) -> Result<usize, CryptoError> {
        if !offset.is_multiple_of(SHA_DMA_ALIGNMENT) {
            return Err(CryptoError::InvalidParameter);
        }

        let context =
            self.0.sha_contexts.get_mut(&(sender, context_id)).ok_or(CryptoError::InvalidParameter)?;

        // Check it doesn't exceed the total length
        if context.bytes_processed + length > context.hw_context.total_len {
            return Err(CryptoError::InvalidDataLength);
        }

        let is_final = context.bytes_processed + length == context.hw_context.total_len;
        let data = buf.subrange(offset, length).ok_or(CryptoError::InvalidParameter)?;

        if length % 4 != 0 {
            let is_single_shot = context.bytes_processed == 0 && length == context.hw_context.total_len;
            if !is_single_shot {
                if !is_final {
                    return Err(CryptoError::InvalidDataLength);
                }

                if let Some(hasher) = context.sw_hasher.as_mut() {
                    hasher.update(data.as_slice());
                }

                // Finalize software hash for unaligned final chunk and store for finalize().
                let hash = context.sw_hasher.take().ok_or(CryptoError::InvalidParameter)?.finalize_to_vec();
                let hash_len = hash_size_hw(context.hw_context.algorithm);
                context.hw_context.hash_state[..hash_len].copy_from_slice(&hash[..hash_len]);
                context.hw_context.bytes_remaining = 0;
                context.bytes_processed += length;
                context.sw_final_hash = Some(hash);
                return Ok(length);
            }

            let (block_size, length_field_bytes) = match context.hw_context.algorithm {
                Algorithm::Sha224 | Algorithm::Sha256 => (64usize, 8usize),
                Algorithm::Sha384 | Algorithm::Sha512 => (128usize, 16usize),
                _ => return Err(CryptoError::InvalidParameter),
            };
            let padded = sha_pad(data.as_slice(), block_size, length_field_bytes);

            self.0.power_manager.enable_peripheral(PeripheralId::Sha)?;
            let hash = match context.hw_context.algorithm {
                Algorithm::Sha224 => self.0.sha.hash_padded::<Sha224>(&padded).to_vec(),
                Algorithm::Sha256 => self.0.sha.hash_padded::<Sha256>(&padded).to_vec(),
                Algorithm::Sha384 => self.0.sha.hash_padded::<Sha384>(&padded).to_vec(),
                Algorithm::Sha512 => self.0.sha.hash_padded::<Sha512>(&padded).to_vec(),
                _ => {
                    self.0.power_manager.disable_peripheral(PeripheralId::Sha)?;
                    return Err(CryptoError::InvalidParameter);
                }
            };
            self.0.power_manager.disable_peripheral(PeripheralId::Sha)?;

            let hash_len = hash_size_hw(context.hw_context.algorithm);
            context.hw_context.hash_state[..hash_len].copy_from_slice(&hash[..hash_len]);
            context.hw_context.bytes_remaining = 0;
            context.bytes_processed += length;
            return Ok(length);
        }

        if let Some(hasher) = context.sw_hasher.as_mut() {
            hasher.update(data.as_slice());
        }

        let dma_range = buf.subrange(offset, length).ok_or(CryptoError::InvalidParameter)?;

        self.0.power_manager.enable_peripheral(PeripheralId::Sha)?;

        let is_first_update = context.bytes_processed == 0;
        if is_first_update {
            match context.hw_context.algorithm {
                Algorithm::Sha224 => self.0.sha.init_streaming::<Sha224>(context.hw_context.total_len),
                Algorithm::Sha256 => self.0.sha.init_streaming::<Sha256>(context.hw_context.total_len),
                Algorithm::Sha384 => self.0.sha.init_streaming::<Sha384>(context.hw_context.total_len),
                Algorithm::Sha512 => self.0.sha.init_streaming::<Sha512>(context.hw_context.total_len),
                _ => return Err(CryptoError::InvalidParameter),
            }
        } else {
            self.0.sha.restore_context(&context.hw_context);
        }

        flush_cache(dma_range, CacheOperation::Clean)?;

        if is_final {
            self.0.sha.update_dma_final(|| -> Result<(), CryptoError> {
                unsafe { self.0.dma_sha.execute(dma_range).map_err(convert_dma_error)? };
                self.0.dma_sha.wait().map_err(convert_dma_error)?;
                Ok(())
            })?;
        } else {
            self.0.sha.update_dma(|| -> Result<(), CryptoError> {
                unsafe { self.0.dma_sha.execute(dma_range).map_err(convert_dma_error)? };
                self.0.dma_sha.wait().map_err(convert_dma_error)?;
                Ok(())
            })?;
        }

        self.0.sha.save_context(&mut context.hw_context);
        context.bytes_processed += length;

        Ok(length)
    }

    /// Finalize the streaming SHA hash and return the result.
    /// The context is removed after finalization.
    pub fn sha_finalize(&mut self, sender: PID, context_id: u8) -> Result<Vec<u8>, CryptoError> {
        // Check if all the data was processed before removing context
        {
            let context =
                self.0.sha_contexts.get(&(sender, context_id)).ok_or(CryptoError::InvalidParameter)?;
            if context.bytes_processed != context.hw_context.total_len {
                return Err(CryptoError::InvalidDataLength);
            }
        }

        let context = self.0.sha_contexts.remove(&(sender, context_id)).unwrap();

        let hash = match context.sw_final_hash {
            Some(hash) => hash,
            None => context.hw_context.hash_state[..hash_size_hw(context.hw_context.algorithm)].to_vec(),
        };

        // Disable SHA peripheral if no more contexts are active
        if self.0.sha_contexts.is_empty() {
            self.0.power_manager.disable_peripheral(PeripheralId::Sha).ok();
        }

        Ok(hash)
    }

    /// Abort/cleanup a streaming SHA context without finalizing.
    /// Used when a context needs to be cleaned up on error paths or early exit.
    pub fn sha_abort(&mut self, sender: PID, context_id: u8) {
        if self.0.sha_contexts.remove(&(sender, context_id)).is_some() {
            // Disable SHA peripheral if no more contexts are active
            if self.0.sha_contexts.is_empty() {
                self.0.power_manager.disable_peripheral(PeripheralId::Sha).ok();
            }
        }
    }
}

fn convert_sha_algo(value: ShaAlgo) -> Algorithm {
    match value {
        ShaAlgo::Sha224 => Algorithm::Sha224,
        ShaAlgo::Sha256 => Algorithm::Sha256,
        ShaAlgo::Sha384 => Algorithm::Sha384,
        ShaAlgo::Sha512 => Algorithm::Sha512,
    }
}

fn hash_size_hw(algo: Algorithm) -> usize {
    match algo {
        Algorithm::Sha224 => SHA224_HASH_SIZE,
        Algorithm::Sha256 => SHA256_HASH_SIZE,
        Algorithm::Sha384 => SHA384_HASH_SIZE,
        Algorithm::Sha512 => SHA512_HASH_SIZE,
        _ => 0,
    }
}

fn sha_pad(data: &[u8], block_size: usize, length_field_bytes: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len() + block_size);
    out.extend_from_slice(data);
    out.push(0x80);

    let len_mod = (out.len() + length_field_bytes) % block_size;
    let zero_len = if len_mod == 0 { 0 } else { block_size - len_mod };
    out.extend(core::iter::repeat(0).take(zero_len));

    let bit_len = (data.len() as u128) * 8;
    if length_field_bytes == 8 {
        out.extend_from_slice(&(bit_len as u64).to_be_bytes());
    } else {
        out.extend_from_slice(&bit_len.to_be_bytes());
    }

    out
}

fn convert_dma_error(value: DmaError) -> CryptoError {
    match value {
        DmaError::XousError(e) => CryptoError::XousError(e),
        DmaError::InvalidParameter => CryptoError::InvalidParameter,
        DmaError::InvalidAddress => CryptoError::InvalidAddress,
        DmaError::InvalidAlignment => CryptoError::InvalidDataLength,
        DmaError::BufferNotContiguous => CryptoError::BufferNotContiguous,
        _ => CryptoError::DmaError,
    }
}
