// SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::{DirHandle, Error, FileHandle, OpenDir, OpenFile, Server},
    fs::messages::{CreateDirMessage, OpenDirMessage, OpenFileMessage},
    server::xous,
};

impl server::ArchiveHandler<OpenFileMessage> for Server {
    fn handle(
        &mut self,
        msg: OpenFileMessage,
        sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> Result<FileHandle, Error> {
        if msg.flags.read {
            self.check_read_access(sender, msg.location)?
        }
        if msg.flags.write {
            self.check_write_access(sender, msg.location)?
        }
        if !msg.flags.read && !msg.flags.write {
            return Err(Error::InvalidOperation);
        }
        if msg.flags.create && !msg.flags.write {
            return Err(Error::InvalidOperation);
        }
        self.create_base_dir(msg.location, sender)?;

        let path = crate::path_of(msg.location, &msg.path, sender);
        let file = if msg.flags.create {
            self.root_dir(msg.location)?.create_file(&path)?
        } else {
            self.root_dir(msg.location)?.open_file(&path)?
        };
        let files = self.files.entry(sender).or_default();
        let counter = files.counter;
        files
            .open
            .insert(FileHandle(counter), OpenFile { file, path, flags: msg.flags, location: msg.location });
        files.counter = counter.wrapping_add(1);
        Ok(FileHandle(counter))
    }
}

impl server::ArchiveHandler<OpenDirMessage> for Server {
    fn handle(
        &mut self,
        msg: OpenDirMessage,
        sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> Result<DirHandle, Error> {
        self.check_read_access(sender, msg.location)?;
        self.create_base_dir(msg.location, sender)?;
        let path = crate::path_of(msg.location, &msg.path, sender);
        let mut dir = self.root_dir(msg.location)?;
        if !path.is_empty() {
            dir = dir.open_dir(&path)?;
        };
        let dirs = self.dirs.entry(sender).or_default();
        let counter = dirs.counter;
        dirs.open.insert(DirHandle(counter), OpenDir { iter: dir.iter(), path, location: msg.location });
        dirs.counter = counter.wrapping_add(1);
        Ok(DirHandle(counter))
    }
}

impl server::ArchiveHandler<CreateDirMessage> for Server {
    fn handle(
        &mut self,
        msg: CreateDirMessage,
        sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> Result<DirHandle, Error> {
        self.check_write_access(sender, msg.location)?;
        self.create_base_dir(msg.location, sender)?;
        let path = crate::path_of(msg.location, &msg.path, sender);
        let dir = self.root_dir(msg.location)?.create_dir(&path)?;
        let dirs = self.dirs.entry(sender).or_default();
        let counter = dirs.counter;
        dirs.open.insert(DirHandle(dirs.counter), OpenDir { iter: dir.iter(), path, location: msg.location });
        dirs.counter = counter.wrapping_add(1);
        Ok(DirHandle(counter))
    }
}
