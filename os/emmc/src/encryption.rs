// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

#![cfg(keyos)]

use crypto::AES_BLOCK_SIZE;
use xous::MemoryRange;

use crate::error::EmmcError;
use crate::implementation::{Direction, EmmcServer};
use crate::{
    messages::{ReadEncryptedBlocks, WriteEncryptedBlocks},
    BLOCK_SIZE,
};
use crate::{SD_BUFFER_BLOCKS, TOTAL_FLASH_BLOCKS};

pub const XTS_CLUSTER_BLOCKS: usize = 64;

impl EmmcServer {
    fn crypt_blocks(
        &mut self,
        src: MemoryRange,
        dst: MemoryRange,
        mut block_idx: usize,
        mut remaining_blocks: usize,
        direction: crypto::Direction,
    ) -> Result<(), EmmcError> {
        if self.crypto_api.is_none() {
            self.crypto_api = Some(crate::CryptoApi::default());
        }
        let crypto_api = self.crypto_api.as_mut().unwrap();

        let mut tweak = [0u8; 16];
        let mut buffer_offset = 0;

        while remaining_blocks > 0 {
            tweak[..4].copy_from_slice(&(block_idx / XTS_CLUSTER_BLOCKS).to_le_bytes());
            let block_within_xts_cluster = block_idx % XTS_CLUSTER_BLOCKS;
            let j = block_within_xts_cluster * BLOCK_SIZE / AES_BLOCK_SIZE;

            let blocks = (XTS_CLUSTER_BLOCKS - block_within_xts_cluster).min(remaining_blocks);
            unsafe {
                crypto_api.disk_encrypt_unsafe(
                    tweak,
                    j,
                    src.subrange(buffer_offset, blocks * BLOCK_SIZE).unwrap(),
                    dst.subrange(buffer_offset, blocks * BLOCK_SIZE).unwrap(),
                    direction,
                )?;
            }
            remaining_blocks -= blocks;
            block_idx += blocks;
            buffer_offset += blocks * BLOCK_SIZE
        }
        Ok(())
    }
}

impl server::LendMutHandler<ReadEncryptedBlocks> for EmmcServer {
    fn handle(
        &mut self,
        msg: ReadEncryptedBlocks,
        _sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> Result<usize, EmmcError> {
        log::trace!("{msg:?}");
        let len = msg.block_count * BLOCK_SIZE;
        if len > msg.buf.len() || msg.block_count > SD_BUFFER_BLOCKS {
            return Err(EmmcError::BufferTooLarge);
        }
        if (msg.block_index as usize).saturating_add(msg.block_count) > TOTAL_FLASH_BLOCKS {
            return Err(EmmcError::OutOfRange);
        }

        xous::flush_cache(
            msg.buf.subrange(0, len).ok_or(EmmcError::InternalError)?,
            xous::CacheOperation::Invalidate,
        )?;
        self.hardware_request(Direction::Read, msg.block_index, msg.block_count, self.tmp_buf.as_mut_ptr())?;
        self.crypt_blocks(
            self.tmp_buf.subrange(0, len).ok_or(EmmcError::InternalError)?,
            msg.buf.subrange(0, len).ok_or(EmmcError::InternalError)?,
            msg.block_index as usize,
            msg.block_count,
            crypto::Direction::Decrypt,
        )?;
        Ok(msg.block_count)
    }
}

impl server::LendMutHandler<WriteEncryptedBlocks> for EmmcServer {
    fn handle(
        &mut self,
        msg: WriteEncryptedBlocks,
        _sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> Result<usize, EmmcError> {
        log::trace!("{msg:?}");
        let len = msg.block_count * BLOCK_SIZE;
        if len > msg.buf.len() || msg.block_count > SD_BUFFER_BLOCKS {
            return Err(EmmcError::BufferTooLarge);
        }
        if (msg.block_index as usize).saturating_add(msg.block_count) > TOTAL_FLASH_BLOCKS {
            return Err(EmmcError::OutOfRange);
        }
        xous::flush_cache(msg.buf.subrange(0, len).unwrap(), xous::CacheOperation::Clean)?;
        self.crypt_blocks(
            msg.buf.subrange(0, len).unwrap(),
            self.tmp_buf.subrange(0, len).unwrap(),
            msg.block_index as usize,
            msg.block_count,
            crypto::Direction::Encrypt,
        )?;
        self.hardware_request(Direction::Write, msg.block_index, msg.block_count, self.tmp_buf.as_mut_ptr())?;

        Ok(msg.block_count)
    }
}
