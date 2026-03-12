// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;

use fs::OpenFlags;
use futures_lite::future::or;
use quantum_link::{
    foundation_api::backup::{
        BackupChunk, CreateMagicBackupEvent, CreateMagicBackupResult, SeedFingerprint, StartMagicBackup,
    },
    messages::SendMagicBackupEvent,
};
use security::messages::GetSeedFingerprint;
use whence::WhenceExt;
use worker::WorkerHandle;

use super::core::BackupFile;
use crate::security_permissions::SecurityPermissions;
use crate::FileSystem;
use crate::{quantum_link_permissions::QuantumLinkPermissions, QlStatus};

#[derive(Debug, thiserror::Error)]
pub(super) enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Fs(#[from] fs::Error),
    #[error(transparent)]
    Security(#[from] security::AccessDenied),
    #[error(transparent)]
    Ql(#[from] quantum_link::SendMessageError),
}

#[derive(Debug, Clone)]
enum Outcome {
    Success,
    EnvoyError(String),
    UploadError(whence::Error<Arc<Error>>),
}

struct Context<'a> {
    worker: &'a WorkerHandle,
    ql_status: &'a QlStatus,
    backup_file: &'a BackupFile,
}

pub(super) async fn publish_backup(
    worker: &WorkerHandle,
    ql_status: &QlStatus,
    backup_file: &BackupFile,
) -> whence::Result<(), Arc<Error>> {
    log::info!("starting backup");

    let cx = Context { worker, ql_status, backup_file };

    loop {
        match try_publish_until_envoy_responds(&cx).await {
            Outcome::Success => {
                log::info!("magic backup confirmed by envoy");
                return Ok(());
            }
            Outcome::EnvoyError(e) => {
                log::warn!("backup failed on envoy: {e}, retrying...");
            }
            Outcome::UploadError(e) => {
                log::warn!("backup publish failed: {e:?}");
                return Err(e);
            }
        }
    }
}

async fn try_publish_until_envoy_responds(cx: &Context<'_>) -> Outcome {
    let upload = async {
        match publish_chunks(cx).await {
            Ok(()) => Outcome::Success,
            Err(e) => Outcome::UploadError(e.map(Arc::new)),
        }
    };

    let confirmation = shared_fut::Shared::new(async {
        match await_confirmation(cx.worker).await {
            CreateMagicBackupResult::Success => Outcome::Success,
            CreateMagicBackupResult::Error { error } => Outcome::EnvoyError(error),
        }
    });

    match or(upload, confirmation.clone()).await {
        Outcome::Success => confirmation.await,
        outcome => outcome,
    }
}

async fn await_confirmation(worker: &WorkerHandle) -> CreateMagicBackupResult {
    worker
        .async_archive::<QuantumLinkPermissions, _>(quantum_link::messages::AwaitCreateMagicBackupResult)
        .await
}

async fn publish_chunks(cx: &Context<'_>) -> whence::Result<(), Error> {
    log::info!("waiting for ql connection");
    cx.ql_status.ready().await;
    log::info!("ql ready");

    let seed_fingerprint =
        cx.worker.async_archive::<SecurityPermissions, _>(GetSeedFingerprint).await.whence()?;

    let fs = FileSystem::default();
    let file = fs.open_file(&cx.backup_file.path, cx.backup_file.location, OpenFlags::READ_ONLY).whence()?;
    let metadata = file.metadata().whence()?;
    let total_chunks = metadata.size.div_ceil(super::utils::DEFAULT_CHUNK_SIZE as u64) as u32;

    log::info!("sending backup with {total_chunks}");

    let start_event = SendMagicBackupEvent {
        event: CreateMagicBackupEvent::Start(StartMagicBackup {
            seed_fingerprint: SeedFingerprint(seed_fingerprint),
            total_chunks,
            hash: cx.backup_file.hash,
        }),
    };
    cx.worker.async_archive::<QuantumLinkPermissions, _>(start_event.clone()).await.whence()?;

    let mut reader = super::utils::DefaultChunkedReader::new(file);
    let mut chunk_index = 0;

    while let Some(data) = reader.next_chunk().whence()? {
        let chunk = BackupChunk { chunk_index, total_chunks, data: data.to_vec() };
        let msg = SendMagicBackupEvent { event: CreateMagicBackupEvent::Chunk(chunk) };
        cx.worker.async_archive::<QuantumLinkPermissions, _>(msg).await.whence()?;
        chunk_index += 1;
    }

    log::info!("successfully published backup");
    Ok(())
}
