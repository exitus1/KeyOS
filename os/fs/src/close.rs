// SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Write;

use fs::messages::{CloseDir, CloseFile};
use server::xous;

use crate::{Error, Server};

impl server::BlockingScalarHandler<CloseFile> for Server {
    fn handle(
        &mut self,
        close: CloseFile,
        sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> Result<(), Error> {
        let open = &mut self.files.get_mut(&sender).ok_or(Error::FileNotOpen)?.open;
        open.remove(&close.0).ok_or(Error::FileNotOpen)?.file.flush()?;
        if open.is_empty() {
            self.files.remove(&sender).unwrap();
        }
        Ok(())
    }
}

impl server::BlockingScalarHandler<CloseDir> for Server {
    fn handle(
        &mut self,
        close: CloseDir,
        sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> Result<(), Error> {
        let open = &mut self.dirs.get_mut(&sender).ok_or(Error::FileNotOpen)?.open;
        open.remove(&close.0).ok_or(Error::FileNotOpen)?;
        if open.is_empty() {
            self.dirs.remove(&sender).unwrap();
        }
        Ok(())
    }
}
