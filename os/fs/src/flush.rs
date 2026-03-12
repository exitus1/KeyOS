// SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Write;

use fs::messages::{Flush, FlushFs};
use {
    crate::{Error, Server},
    server::xous,
};

impl server::BlockingScalarHandler<Flush> for Server {
    fn handle(
        &mut self,
        flush: Flush,
        sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> Result<(), Error> {
        let open = &mut self.files.get_mut(&sender).ok_or(Error::FileNotOpen)?.open;
        open.get_mut(&flush.0).ok_or(Error::FileNotOpen)?.file.flush()?;
        Ok(())
    }
}

impl server::BlockingScalarHandler<FlushFs> for Server {
    fn handle(
        &mut self,
        flush: FlushFs,
        _sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> Result<(), Error> {
        self.flush_fs(flush.0)
    }
}
