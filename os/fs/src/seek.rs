// SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::{Error, OpenFile, Server},
    fs::messages::SeekFile,
    server::xous,
    std::io::Seek,
};

impl server::ArchiveHandler<SeekFile> for Server {
    fn handle(
        &mut self,
        seek: SeekFile,
        sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> Result<u64, Error> {
        let OpenFile { file, .. } = self
            .files
            .get_mut(&sender)
            .ok_or(Error::FileNotOpen)?
            .open
            .get_mut(&seek.file)
            .ok_or(Error::FileNotOpen)?;
        file.seek(seek.pos.into()).map_err(Into::into)
    }
}
