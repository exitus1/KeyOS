// SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use fs::messages::TruncateFile;

use crate::{Error, Server};

impl server::ArchiveHandler<TruncateFile> for Server {
    fn handle(
        &mut self,
        msg: TruncateFile,
        sender: server::xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> <TruncateFile as server::Archive>::Response {
        let open = self
            .files
            .get_mut(&sender)
            .ok_or(Error::FileNotOpen)?
            .open
            .get_mut(&msg.0)
            .ok_or(Error::FileNotOpen)?;
        if !open.flags.write {
            return Err(Error::InvalidOperation);
        }
        open.file.truncate()?;
        Ok(())
    }
}
