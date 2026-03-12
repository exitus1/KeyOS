// SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Read;
use std::io::Seek;
use std::io::Write;

use fs::BLOCK_SIZE;

use super::BlockDevice;

const VIRTUAL_SYSTEM_VOLUME_SIZE: u64 = 128 * 1024 * 1024;
const VIRTUAL_USER_VOLUME_SIZE: u64 = 8 * 1024 * 1024 * 1024;

impl BlockDevice for std::fs::File {
    fn read_blocks(&mut self, block_idx: u32, block_buf: &mut [u8]) -> Result<(), std::io::Error> {
        self.seek(std::io::SeekFrom::Start(block_idx as u64 * BLOCK_SIZE as u64))?;
        self.read_exact(block_buf)
    }

    fn write_blocks(&mut self, block_idx: u32, block_buf: &[u8]) -> Result<(), std::io::Error> {
        self.seek(std::io::SeekFrom::Start(block_idx as u64 * BLOCK_SIZE as u64))?;
        self.write_all(block_buf)
    }

    fn flush_blocks(&mut self) -> Result<(), std::io::Error> { self.flush() }
}

/// Create files to be used as disk partitions in hosted mode.
pub fn init_file(name: &str, size: u64) -> Result<(), std::io::Error> {
    log::info!("initializing disk files");

    // Don't truncate an existing file
    if let Ok(file) = std::fs::OpenOptions::new().read(true).open(name) {
        log::info!("found disk.dat");
        match fatfs::FileSystem::new(file, fatfs::FsOptions::new()) {
            Ok(_) => return Ok(()),
            Err(e) => log::info!("disk.dat is not a valid fatfs ({e:?})"),
        }
    }

    // Create a 8GiB partition 0 formatted as FAT32.
    log::info!("Creating disk.dat");
    let mut file =
        std::fs::OpenOptions::new().write(true).read(true).truncate(true).create(true).open(name)?;
    file.seek(std::io::SeekFrom::End(size as i64 - 1))?;
    file.write_all(&[0])?;
    file.flush()?;

    file.seek(std::io::SeekFrom::Start(0))?;

    fatfs::format_volume(
        &mut file,
        fatfs::FormatVolumeOptions::new()
            .fat_type(fatfs::FatType::Fat32)
            .total_sectors((size / BLOCK_SIZE as u64) as u32)
            .bytes_per_cluster(64 * BLOCK_SIZE as u32),
    )?;
    file.seek(std::io::SeekFrom::Start(0))?;

    // Fix up partition entry a bit
    file.seek(std::io::SeekFrom::Start(446))?;
    file.write_all(&[
        0x80, // Active
        0x00, 0x00, 0x00, // First sector in CHS format
        0x0c, // Type: FAT32 LBA
        0xff, 0xff, 0xff, // Last sector in CHS format (unused)
        0x00, 0x00, 0x00, 0x00, // First sector in LBA
        0x00, 0x00, 0x00, 0x01, // Last sector in LBA, in little endian (0x1000000)
    ])?;
    Ok(())
}

pub fn init_files() -> Result<(), std::io::Error> {
    init_file("disk.dat", VIRTUAL_USER_VOLUME_SIZE)?;
    init_file("disk_system.dat", VIRTUAL_SYSTEM_VOLUME_SIZE)?;

    let system = std::fs::OpenOptions::new().write(true).read(true).open("disk_system.dat")?;
    let system_fs =
        fatfs::FileSystem::new(system, fatfs::FsOptions::new()).expect("virtual system disk not formatted");
    system_fs.root_dir().create_dir("keyos").ok();
    Ok(())
}
