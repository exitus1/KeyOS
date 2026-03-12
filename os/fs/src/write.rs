// SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::{Error, FileHandle, OpenFile, Server},
    fs::messages::{AsyncWrite, WriteFile},
    server::xous,
    std::io::Write,
};

impl server::LendMutHandler<WriteFile> for Server {
    fn handle(
        &mut self,
        write: WriteFile,
        sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> Result<usize, Error> {
        self.write_file(
            write.handle,
            write.buf.subrange(0, write.write_len).ok_or(Error::InvalidBufferLength)?.as_slice(),
            sender,
        )
    }
}

impl server::ArchiveHandler<AsyncWrite> for Server {
    fn handle(
        &mut self,
        msg: AsyncWrite,
        sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> Result<usize, Error> {
        self.write_file(msg.handle, &msg.buffer, sender)
    }
}

impl Server {
    fn write_file(&mut self, handle: FileHandle, buffer: &[u8], sender: xous::PID) -> Result<usize, Error> {
        let files = self.files.get_mut(&sender).ok_or(Error::FileNotOpen)?;
        let OpenFile { file, flags, .. } = files.open.get_mut(&handle).ok_or(Error::FileNotOpen)?;
        if !flags.write {
            return Err(Error::InvalidOperation);
        }
        file.write_all(buffer).map_err(|_| Error::Io)?;
        Ok(buffer.len())
    }
}
