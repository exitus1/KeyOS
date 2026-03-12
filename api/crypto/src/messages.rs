// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use server::{xous::MemoryRange, SimpleMemoryMessage};

use crate::error::{CryptoError, ShamirError};
use crate::Direction;

#[derive(Debug, server::Message)]
#[response(Result<usize, CryptoError>)]
pub enum AesSetup {
    Ecb { key_buf: MemoryRange, key_len: usize },
    Cbc { key_buf: MemoryRange, key_len: usize },
}

impl From<AesSetup> for SimpleMemoryMessage {
    fn from(value: AesSetup) -> Self {
        match value {
            AesSetup::Ecb { key_buf, key_len } => {
                SimpleMemoryMessage { buf: key_buf, arg1: key_len, arg2: 0 }
            }
            AesSetup::Cbc { key_buf, key_len } => {
                SimpleMemoryMessage { buf: key_buf, arg1: key_len, arg2: 1 }
            }
        }
    }
}

impl From<SimpleMemoryMessage> for AesSetup {
    fn from(value: SimpleMemoryMessage) -> Self {
        match value.arg2 {
            0 => AesSetup::Ecb { key_buf: value.buf, key_len: value.arg1 },
            _ => AesSetup::Cbc { key_buf: value.buf, key_len: value.arg1 },
        }
    }
}

#[derive(Debug, server::Message)]
#[response(Result<usize, CryptoError>)]
pub struct AesExecute {
    pub buf: MemoryRange,
    pub transfer_id: u8,
    pub direction: Direction,
    pub offset: usize,
    pub blocks: usize,
}

const BLOCKS_OFFSET: usize = 9;
const TRANSFER_ID_OFFSET: usize = 1;
const DECRYPT_FLAG: usize = 1;

impl From<AesExecute> for SimpleMemoryMessage {
    fn from(value: AesExecute) -> Self {
        Self {
            buf: value.buf,
            arg1: value.offset,
            arg2: (value.blocks << BLOCKS_OFFSET)
                | ((value.transfer_id as usize) << TRANSFER_ID_OFFSET)
                | match value.direction {
                    Direction::Encrypt => 0,
                    Direction::Decrypt => DECRYPT_FLAG,
                },
        }
    }
}

impl From<SimpleMemoryMessage> for AesExecute {
    fn from(value: SimpleMemoryMessage) -> Self {
        Self {
            buf: value.buf,
            offset: value.arg1,
            transfer_id: (value.arg2 >> TRANSFER_ID_OFFSET) as u8,
            direction: if value.arg2 & DECRYPT_FLAG != 0 { Direction::Decrypt } else { Direction::Encrypt },
            blocks: (value.arg2 >> BLOCKS_OFFSET),
        }
    }
}

#[cfg(keyos)]
#[derive(Debug, server::Message, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[response(Result<usize, CryptoError>)]
pub struct DiskEncryptUnsafe {
    pub tweak: [u8; 16],
    pub j: usize,
    pub src: usize,
    pub dst: usize,
    pub len: usize,
    pub direction: Direction,
}

#[derive(Debug, server::Message)]
pub struct AesClear(pub u8);

#[derive(Debug, Clone, Copy, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub enum ShaAlgo {
    Sha224,
    Sha256,
    Sha384,
    Sha512,
}

impl From<usize> for ShaAlgo {
    fn from(value: usize) -> Self {
        match value {
            0 => ShaAlgo::Sha224,
            1 => ShaAlgo::Sha256,
            2 => ShaAlgo::Sha384,
            3 => ShaAlgo::Sha512,
            _ => unreachable!(),
        }
    }
}

impl From<ShaAlgo> for usize {
    fn from(value: ShaAlgo) -> Self {
        match value {
            ShaAlgo::Sha224 => 0,
            ShaAlgo::Sha256 => 1,
            ShaAlgo::Sha384 => 2,
            ShaAlgo::Sha512 => 3,
        }
    }
}

/// Initialize a streaming SHA hash context
/// Returns a context ID that can be used with `ShaUpdate` and `ShaFinalize`
#[derive(Debug, server::Message, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[response(Result<usize, CryptoError>)]
pub struct ShaInit {
    pub algo: ShaAlgo,
    /// Total message size in bytes
    pub total_len: usize,
}

/// Update the SHA hash with more data via DMA
/// The buffer must be page-aligned and the offset must be aligned to `SHA_DMA_ALIGNMENT`.
/// Can be called multiple times with arbitrary block counts.
#[derive(Debug, server::Message)]
#[response(Result<usize, CryptoError>)]
pub struct ShaUpdate {
    pub context_id: u8,
    pub buf: MemoryRange,
    pub offset: usize,
    pub length: usize,
}

const CONTEXT_ID_OFFSET: usize = 2;

impl From<ShaUpdate> for SimpleMemoryMessage {
    fn from(value: ShaUpdate) -> Self {
        Self {
            buf: value.buf,
            arg1: (value.offset << CONTEXT_ID_OFFSET) | (value.context_id as usize),
            arg2: value.length,
        }
    }
}

impl From<SimpleMemoryMessage> for ShaUpdate {
    fn from(value: SimpleMemoryMessage) -> Self {
        Self {
            context_id: (value.arg1 & 0b11) as u8,
            buf: value.buf,
            offset: value.arg1 >> CONTEXT_ID_OFFSET,
            length: value.arg2,
        }
    }
}

/// Finalize the SHA hash and retrieve the result
#[derive(Debug, server::Message, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[response(Result<Vec<u8>, CryptoError>)]
pub struct ShaFinalize {
    pub context_id: u8,
}

/// Abort/cleanup a streaming SHA context without finalizing
/// Used when a context needs to be cleaned up on error paths or early exit
#[derive(Debug, server::Message)]
pub struct ShaAbort(pub u8);

#[derive(Debug, server::Message, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[response(Result<Vec<u8>, CryptoError>)]
pub struct Hmac {
    pub algo: ShaAlgo,
    pub key: Vec<u8>,
    pub data: Vec<u8>,
}

#[derive(Debug, server::Message, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[response(Result<Vec<Vec<u8>>, ShamirError>)]
pub struct ShamirSplit {
    pub secret: Vec<u8>,
    pub num_shares: usize,
    pub threshold: usize,
}

#[derive(Debug, server::Message, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[response(Result<Vec<u8>, ShamirError>)]
pub struct ShamirRecover {
    pub indexes: Vec<usize>,
    pub shares: Vec<Vec<u8>>,
}
