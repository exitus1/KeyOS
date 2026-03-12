// SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use fs::messages::Rename;

use crate::{Error, Server};

impl server::ArchiveHandler<Rename> for Server {
    fn handle(
        &mut self,
        msg: Rename,
        sender: server::xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> <Rename as server::Archive>::Response {
        self.check_write_access(sender, msg.location)?;
        let from = crate::path_of(msg.location, &msg.from, sender);
        let to = crate::path_of(msg.location, &msg.to, sender);
        let mut open_paths = self
            .files
            .iter()
            .flat_map(|(_, files)| files.open.values().map(|open| &open.path))
            .chain(self.dirs.iter().flat_map(|(_, dirs)| dirs.open.values().map(|open| &open.path)));
        if open_paths.any(|p| p == &from) {
            return Err(Error::FileInUse);
        }
        self.root_dir(msg.location)?.rename(&from, &self.root_dir(msg.location)?, &to)?;
        self.flush_fs(msg.location)?;
        Ok(())
    }
}
