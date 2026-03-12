// SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>trait
// SPDX-License-Identifier: GPL-3.0-or-later

//! `emmc` server-based `fatfs` storage adapter.

use super::BlockDevice;
use crate::MassStorageApi;

impl BlockDevice for MassStorageApi {
    fn read_blocks(&mut self, block_idx: u32, block_buf: &mut [u8]) -> Result<(), std::io::Error> {
        self.read_blocks(block_idx, block_buf).map_err(|_| std::io::ErrorKind::Other.into())
    }

    fn write_blocks(&mut self, block_idx: u32, block_buf: &[u8]) -> Result<(), std::io::Error> {
        self.write_blocks(block_idx, block_buf).map_err(|_| std::io::ErrorKind::Other.into())
    }

    fn flush_blocks(&mut self) -> Result<(), std::io::Error> {
        // Removable mass storage devices very rarely have any volatile caches;
        // their write operations block until complete, because they expect to
        // be yanked out at any moment.
        Ok(())
    }
}
