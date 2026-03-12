// SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use mass_storage_server::MassStorageEvent;
use server::ScalarEventHandler;

use crate::{disk::DynamicDisk, FileSystemEvent, FileSystemEventType, Location, MassStorageApi, Server};

impl ScalarEventHandler<MassStorageEvent> for Server {
    fn handle(
        &mut self,
        msg: MassStorageEvent,
        _sender: server::xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) {
        match msg {
            // TODO: Block count and block size should probably
            //       be cross-checked with the partition info
            MassStorageEvent::Connect { .. } => {
                self.clean_fs_usb();
                let ms = MassStorageApi::default();
                match fatfs::FileSystem::new(DynamicDisk::new(ms.into(), 0), fatfs::FsOptions::new()) {
                    Ok(fs) => {
                        self.fs_usb = Box::into_raw(Box::new(fs));
                        self.send_filesystem_event(FileSystemEvent {
                            location: Location::Usb,
                            event_type: FileSystemEventType::Mounted,
                        });
                    }
                    Err(e) => log::warn!("Could not initialize FS: {e:?}"),
                }
            }
            MassStorageEvent::Disconnect => {
                self.send_filesystem_event(FileSystemEvent {
                    location: Location::Usb,
                    event_type: FileSystemEventType::Unmounted,
                });
                self.clean_fs_usb()
            }
        }
    }
}

impl Server {
    fn clean_fs_usb(&mut self) {
        for files in self.files.values_mut() {
            files.open.retain(|_, f| f.location != Location::Usb);
        }
        for dirs in self.dirs.values_mut() {
            dirs.open.retain(|_, d| d.location != Location::Usb);
        }
        if !self.fs_usb.is_null() {
            unsafe { drop(Box::from_raw(self.fs_usb)) }
            self.fs_usb = std::ptr::null_mut();
        }
    }
}
