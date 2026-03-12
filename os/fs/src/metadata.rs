// SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use fs::{messages::GetMetadata, Metadata};

use crate::{date_from_fatfs, datetime_from_fatfs, Error, OpenFile, Server};

impl server::ArchiveHandler<GetMetadata> for Server {
    fn handle(
        &mut self,
        metadata: GetMetadata,
        sender: server::xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> Result<Metadata, Error> {
        match metadata {
            GetMetadata::Path { path, location } => {
                self.check_read_access(sender, location)?;
                let path = crate::path_of(location, &path, sender);
                let (base, name) = path.rsplit_once('/').unwrap_or(("", &path));
                let dir = if base.is_empty() {
                    self.root_dir(location)?
                } else {
                    self.root_dir(location)?.open_dir(base)?
                };
                for entry in dir.iter() {
                    let entry = entry?;
                    if entry.file_name() == name {
                        return Ok(Metadata {
                            created: datetime_from_fatfs(entry.created()),
                            accessed: date_from_fatfs(entry.accessed()),
                            modified: datetime_from_fatfs(entry.modified()),
                            size: entry.len(),
                            is_dir: entry.is_dir(),
                        });
                    }
                }
                Err(Error::FileNotFound)
            }
            GetMetadata::Handle { handle } => {
                let OpenFile { file, .. } = self
                    .files
                    .get_mut(&sender)
                    .ok_or(Error::FileNotOpen)?
                    .open
                    .get_mut(&handle)
                    .ok_or(Error::FileNotOpen)?;
                let Some(entry) = file.entry() else {
                    return Err(Error::FileNotOpen);
                };
                Ok(Metadata {
                    created: datetime_from_fatfs(entry.created()),
                    accessed: date_from_fatfs(entry.accessed()),
                    modified: datetime_from_fatfs(entry.modified()),
                    size: entry.size().unwrap_or(0) as u64,
                    is_dir: entry.is_dir(),
                })
            }
        }
    }
}
