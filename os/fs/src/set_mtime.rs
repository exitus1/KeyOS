// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use fs::messages::SetMtime;

use crate::{Error, Server};

impl server::ArchiveHandler<SetMtime> for Server {
    fn handle(
        &mut self,
        msg: SetMtime,
        sender: server::xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> <SetMtime as server::Archive>::Response {
        let open = self
            .files
            .get_mut(&sender)
            .ok_or(Error::FileNotOpen)?
            .open
            .get_mut(&msg.handle)
            .ok_or(Error::FileNotOpen)?;
        if !open.flags.write {
            return Err(Error::InvalidOperation);
        }

        #[allow(deprecated)]
        open.file.set_modified(crate::datetime_to_fatfs(msg.datetime));
        Ok(())
    }
}
