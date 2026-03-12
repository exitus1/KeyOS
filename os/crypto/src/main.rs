// SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use crypto::error::{CryptoError, ShamirError};
use crypto::messages::*;
use server::{
    xous::{self},
    ArchiveHandler, LendMutHandler, ScalarHandler, Server,
};

#[cfg(keyos)]
mod atsama5d2;
#[cfg(not(keyos))]
mod hosted;

#[cfg(keyos)]
use atsama5d2::Inner;
#[cfg(not(keyos))]
use hosted::Inner;

#[cfg(keyos)]
power_manager::use_api!();

#[derive(server::Server)]
#[name = "os/crypto"]
pub(crate) struct CryptoServer(Inner);

impl Server for CryptoServer {}

impl LendMutHandler<AesSetup> for CryptoServer {
    fn handle(
        &mut self,
        msg: AesSetup,
        sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> <AesSetup as server::LendMut>::Response {
        self.aes_setup(msg, sender)
    }
}

impl LendMutHandler<AesExecute> for CryptoServer {
    fn handle(
        &mut self,
        msg: AesExecute,
        sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> Result<usize, CryptoError> {
        self.aes_execute(msg, sender)
    }
}

#[cfg(keyos)]
impl ArchiveHandler<DiskEncryptUnsafe> for CryptoServer {
    fn handle(
        &mut self,
        msg: DiskEncryptUnsafe,
        sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> Result<usize, CryptoError> {
        self.disk_encrypt(msg, sender)
    }
}

impl ScalarHandler<AesClear> for CryptoServer {
    fn handle(&mut self, msg: AesClear, sender: xous::PID, _context: &mut server::ServerContext<Self>) {
        self.aes_clear(msg, sender);
    }
}

impl ArchiveHandler<ShaInit> for CryptoServer {
    fn handle(
        &mut self,
        msg: ShaInit,
        sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> Result<usize, CryptoError> {
        self.sha_init(sender, msg.algo, msg.total_len).map(|id| id as usize)
    }
}

impl LendMutHandler<ShaUpdate> for CryptoServer {
    fn handle(
        &mut self,
        msg: ShaUpdate,
        sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> Result<usize, CryptoError> {
        self.sha_update(sender, msg.context_id, msg.buf, msg.offset, msg.length)
    }
}

impl ArchiveHandler<ShaFinalize> for CryptoServer {
    fn handle(
        &mut self,
        msg: ShaFinalize,
        sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> Result<Vec<u8>, CryptoError> {
        self.sha_finalize(sender, msg.context_id)
    }
}

impl ScalarHandler<ShaAbort> for CryptoServer {
    fn handle(&mut self, msg: ShaAbort, sender: xous::PID, _context: &mut server::ServerContext<Self>) {
        self.sha_abort(sender, msg.0);
    }
}

impl ArchiveHandler<Hmac> for CryptoServer {
    fn handle(
        &mut self,
        msg: Hmac,
        _sender: server::xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> <Hmac as server::Archive>::Response {
        self.hmac(msg.algo, &msg.key, &msg.data)
    }
}

impl ArchiveHandler<ShamirSplit> for CryptoServer {
    fn handle(
        &mut self,
        msg: ShamirSplit,
        _sender: server::xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> <ShamirSplit as server::Archive>::Response {
        bc_shamir::split_secret(
            msg.threshold,
            msg.num_shares,
            &msg.secret,
            &mut bc_rand::SecureRandomNumberGenerator,
        )
        .map_err(convert_shamir_error)
    }
}

impl ArchiveHandler<ShamirRecover> for CryptoServer {
    fn handle(
        &mut self,
        msg: ShamirRecover,
        _sender: server::xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> <ShamirRecover as server::Archive>::Response {
        bc_shamir::recover_secret(&msg.indexes, &msg.shares).map_err(convert_shamir_error)
    }
}

fn convert_shamir_error(err: bc_shamir::Error) -> ShamirError {
    match err {
        bc_shamir::Error::SecretTooLong => ShamirError::SecretTooLong,
        bc_shamir::Error::TooManyShares => ShamirError::TooManyShares,
        bc_shamir::Error::InterpolationFailure => ShamirError::InterpolationFailure,
        bc_shamir::Error::ChecksumFailure => ShamirError::ChecksumFailure,
        bc_shamir::Error::SecretTooShort => ShamirError::SecretTooShort,
        bc_shamir::Error::SecretNotEvenLen => ShamirError::SecretNotEvenLen,
        bc_shamir::Error::InvalidThreshold => ShamirError::InvalidThreshold,
        bc_shamir::Error::SharesUnequalLength => ShamirError::SharesUnequalLength,
    }
}

fn main() {
    log_server::init_wait(env!("CARGO_CRATE_NAME")).unwrap();
    log::set_max_level(log::LevelFilter::Info);

    xous::set_thread_priority(xous::ThreadPriority::System5).unwrap();

    server::listen(CryptoServer::new())
}
