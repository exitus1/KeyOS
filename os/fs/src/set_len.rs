// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::{Seek, Write};

use fs::messages::SetLen;

use crate::{Error, Server};

impl server::ArchiveHandler<SetLen> for Server {
    fn handle(
        &mut self,
        msg: SetLen,
        sender: server::xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> <SetLen as server::Archive>::Response {
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

        let current_pos = open.file.seek(std::io::SeekFrom::Current(0))?;

        let current_size = open.file.seek(std::io::SeekFrom::End(0))?;

        if msg.len > current_size {
            // extend the file
            let mut remaining = (msg.len - current_size) as usize;
            let zeros = [0u8; ZERO_BUF_SIZE];
            while remaining > 0 {
                let write_size = std::cmp::min(remaining, ZERO_BUF_SIZE);
                open.file.write_all(&zeros[..write_size])?;
                remaining -= write_size;
            }
        } else if msg.len < current_size {
            // truncate the file
            open.file.seek(std::io::SeekFrom::Start(msg.len))?;
            open.file.truncate()?;
        }

        // restore original position
        open.file.seek(std::io::SeekFrom::Start(current_pos))?;
        Ok(())
    }
}

const ZERO_BUF_SIZE: usize = 2048;
