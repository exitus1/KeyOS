// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Seek;

use byteorder::{LittleEndian, ReadBytesExt};
use fatfs::FileSystem;
use fs::{
    messages::{FormatAirlock, MountAirlock},
    FileSystemEventType,
};

use crate::disk::{DynamicDisk, DynamicDiskBlockDevice, PartitionInfo};
use crate::{disk_image::DiskImage, format_fs, Error, FileSystemEvent, Location, Server};

// --- Airlock ---
// A bit less than 32GB (potentially, on-demand allocated)

// The 0xa00000 is an adjustment (around 14MB), so that the FAT table size comes out to exactly 16374
// sectors, which (with the default 8 reserved sectors) puts the first data sector to 16384 == 0x4000,
// a nicely aligned first data sector.
// This improves the performance by around 30-40% compared to unaligned clusters, but it is also needed
// to ensure offset-less mapping between inner and outer clusters so we can trim them easily.

// WARNING: Changing this value is an incompatible change, as the image file structure depends on it!
//          Use a different filename for a different size.
const AIRLOCK_SIZE: u64 = 32 * 1024 * 1024 * 1024 - 0xa00000; // was e00000
const AIRLOCK_IMAGE_FILE: &str = "airlock.img";
const AIRLOCK_VOLUME_LABEL: [u8; 11] = *b"AIRLOCK    ";

#[derive(Default)]
pub enum AirlockState {
    #[default]
    Uninitialized,
    Unmounted(DynamicDisk),
    Mounted(*mut FileSystem<DynamicDisk>),
}

impl Server {
    pub fn format_airlock(&mut self) -> Result<(), Error> {
        if matches!(&self.airlock, AirlockState::Mounted(_)) {
            log::info!("Airlock is currently mounted");
            return Ok(());
        }
        let mut disk = self.new_airlock_disk()?;
        format_fs(&mut disk, AIRLOCK_VOLUME_LABEL)?;
        self.airlock = AirlockState::Unmounted(disk);
        Ok(())
    }

    pub fn mount_airlock(&mut self) -> Result<(), Error> {
        let disk = match core::mem::take(&mut self.airlock) {
            AirlockState::Uninitialized => {
                log::debug!("Mounting Airlock from Uninitialized state");
                self.new_airlock_disk()?
            }
            AirlockState::Unmounted(disk) => {
                log::debug!("Mounting Airlock from Unmounted state");
                disk
            }
            AirlockState::Mounted(fs) => {
                log::debug!("Mount airlock: already mounted");
                // Set the state back since we took it for the match above
                self.airlock = AirlockState::Mounted(fs);
                return Ok(());
            }
        };

        match self.mount_airlock_fatfs(disk) {
            Ok(fs) => {
                log::info!("Mounting Airlock successful");
                self.airlock = AirlockState::Mounted(Box::into_raw(Box::new(fs)));
                self.send_filesystem_event(FileSystemEvent {
                    location: Location::Airlock,
                    event_type: FileSystemEventType::Mounted,
                });
                match self.trim_airlock() {
                    Err(e) => {
                        log::warn!("Could not trim airlock: {e:?}");
                    }
                    Ok(trimmed) => log::info!("Trimmed {trimmed} blocks from Airlock"),
                };
                Ok(())
            }
            Err(e) => {
                log::error!("Mounting Airlock unsuccessful: {e:?}");
                self.airlock = AirlockState::Unmounted(self.new_airlock_disk()?);
                self.send_filesystem_event(FileSystemEvent {
                    location: Location::Airlock,
                    event_type: FileSystemEventType::Error,
                });
                Err(e.into())
            }
        }
    }

    fn mount_airlock_fatfs(&self, disk: DynamicDisk) -> std::io::Result<fatfs::FileSystem<DynamicDisk>> {
        let result = fatfs::FileSystem::new(disk, fatfs::FsOptions::new())?;
        if self.cluster_size() != result.cluster_size() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid cluster size in Airlock FATFS",
            ));
        }
        let cluster_alignment_check = result.offset_from_cluster(2);
        log::debug!("Alignment of first non-reserved cluster: 0x{cluster_alignment_check:x}");
        if (cluster_alignment_check % self.cluster_size() as u64) != 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Data clusters are misaligned in Airlock FATFS",
            ));
        }
        Ok(result)
    }

    pub fn unmount_airlock(&mut self) -> Result<(), Error> {
        let AirlockState::Mounted(fs) = self.airlock else {
            log::debug!("Unmount airlock: not mounted");
            return Ok(());
        };
        log::info!("Unmounting Airlock");
        for files in self.files.values_mut() {
            files.open.retain(|_, f| f.location != Location::Airlock);
        }
        for dirs in self.dirs.values_mut() {
            dirs.open.retain(|_, d| d.location != Location::Airlock);
        }
        let fs = unsafe { Box::from_raw(fs) };
        match fs.unmount() {
            Ok(mut disk) => {
                disk.seek(std::io::SeekFrom::Start(0)).ok();
                self.airlock = AirlockState::Unmounted(disk);
                self.send_filesystem_event(FileSystemEvent {
                    location: Location::Airlock,
                    event_type: FileSystemEventType::Unmounted,
                });
                Ok(())
            }
            Err(e) => {
                log::error!("Unmounting Airlock unsuccessful: {e:?}");
                self.airlock = AirlockState::Uninitialized;
                Err(e.into())
            }
        }
    }

    pub fn trim_airlock(&mut self) -> Result<u32, Error> {
        let AirlockState::Mounted(fs) = self.airlock else {
            log::debug!("Trim airlock: not mounted");
            return Ok(0);
        };
        let fs = unsafe { &mut *fs };
        let total_clusters = fs.total_clusters();
        let cluster_offset = (fs.offset_from_cluster(0) / self.cluster_size() as u64) as u32;
        let mut trimmed = 0;
        for start_cluster in (0..total_clusters).step_by(0x2000) {
            let mut fat = fs.fat_slice();
            let mut fatdata = vec![0; 0x2000.min((total_clusters - start_cluster) as usize)];
            fat.seek(std::io::SeekFrom::Start(start_cluster as u64 * 4))?;
            fat.read_u32_into::<LittleEndian>(&mut fatdata)?;

            fs.with_disk(|d| {
                let DynamicDiskBlockDevice::DiskImage(d) = &mut d.block_device else {
                    log::error!("Invalid block device under Airlock");
                    return Err(Error::InternalError);
                };
                trimmed += d.trim_clusters(start_cluster + cluster_offset, &fatdata)?;
                Ok(())
            })?
        }
        Ok(trimmed)
    }

    fn cluster_size(&self) -> u32 { unsafe { &*self.fs_user }.cluster_size() }

    fn new_airlock_disk(&self) -> Result<DynamicDisk, Error> {
        let disk_image = DiskImage::new(unsafe { &*self.fs_user }, AIRLOCK_IMAGE_FILE, AIRLOCK_SIZE)?;
        Ok(DynamicDisk::new_with_partition_info(
            disk_image.into(),
            PartitionInfo { start: 0, len_bytes: AIRLOCK_SIZE },
        ))
    }
}

impl server::BlockingScalarHandler<MountAirlock> for Server {
    fn handle(
        &mut self,
        msg: MountAirlock,
        _sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> <MountAirlock as server::BlockingScalar>::Response {
        if msg.0 {
            self.mount_airlock()
        } else {
            self.unmount_airlock()
        }
    }
}

impl server::BlockingScalarHandler<FormatAirlock> for Server {
    fn handle(
        &mut self,
        _msg: FormatAirlock,
        _sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> <FormatAirlock as server::BlockingScalar>::Response {
        self.format_airlock()
    }
}
