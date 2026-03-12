// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Write;

use fs::adapter::FsAdapter;
use fs::OpenFlags;
use quantum_link::foundation_api::backup::{ArchivedRestoreMagicBackupEvent, BackupMetadata};
use whence::WhenceExt;

pub struct BackupDownloader<F: FsAdapter> {
    fs: F,
    state: State<F>,
}

#[derive(Default)]
enum State<F: FsAdapter> {
    #[default]
    Idle,
    InProgress {
        file: F::File,
        file_path: String,
        metadata: BackupMetadata,
        chunks_received: u32,
    },
}

#[derive(Debug)]
pub struct Complete {
    pub file_path: String,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("no backup found")]
    NoBackupFound,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Fs(#[from] fs::Error),
    #[error("envoy error")]
    EnvoyError,
    #[error("invalid state")]
    InvalidState,
    #[error("out of order chunk")]
    InvalidChunk,
}

impl<F: FsAdapter> BackupDownloader<F> {
    pub fn new(fs: F) -> Self { Self { fs, state: State::Idle } }

    pub fn reset_state(&mut self) { self.state = State::Idle; }

    pub fn handle_event(
        &mut self,
        event: &ArchivedRestoreMagicBackupEvent,
    ) -> whence::Result<Option<Complete>, Error>
    where
        F::Permissions: fs::adapter::BasicFsPermissions,
    {
        (|| match event {
            ArchivedRestoreMagicBackupEvent::NotFound => {
                log::error!("No backup found on Envoy");
                Err(Error::NoBackupFound).whence()
            }

            ArchivedRestoreMagicBackupEvent::Starting(metadata) => {
                if !matches!(self.state, State::Idle) {
                    log::info!("resetting MagicBackupMachine state");
                }

                let file_path = String::from("backup/restore.tar");
                let location = fs::Location::EncryptedRoot;
                self.fs.create_dir("backup", location).ok();
                self.fs.remove(&file_path, location).ok();

                let file = self.fs.open_file(&file_path, location, OpenFlags::CREATE).whence()?;

                log::info!("Started backup restore: {} chunks expected", metadata.total_chunks);

                let metadata = BackupMetadata { total_chunks: metadata.total_chunks.to_native() };

                self.state = State::InProgress { file, file_path, metadata, chunks_received: 0 };

                Ok(None)
            }

            ArchivedRestoreMagicBackupEvent::Chunk(chunk) => {
                let State::InProgress { ref mut file, ref metadata, ref mut chunks_received, .. } =
                    self.state
                else {
                    log::error!("Received chunk but not in progress state");
                    return Err(Error::InvalidState).whence();
                };

                if chunk.chunk_index != *chunks_received {
                    log::error!(
                        "Out of order chunk: expected {}, got {}",
                        chunks_received,
                        chunk.chunk_index
                    );
                    return Err(Error::InvalidChunk).whence();
                }

                file.write_all(&chunk.data).whence()?;
                *chunks_received += 1;

                if chunk.chunk_index == chunk.total_chunks - 1 {
                    if *chunks_received != metadata.total_chunks {
                        log::error!(
                            "Chunk count mismatch: expected {}, received {}",
                            metadata.total_chunks,
                            chunks_received
                        );
                        return Err(Error::InvalidChunk).whence();
                    }

                    let State::InProgress { mut file, file_path, .. } = std::mem::take(&mut self.state)
                    else {
                        return Err(Error::InvalidState).whence();
                    };

                    file.flush().whence()?;
                    log::info!("Backup restore complete: {}", file_path);

                    Ok(Some(Complete { file_path }))
                } else {
                    Ok(None)
                }
            }

            ArchivedRestoreMagicBackupEvent::Error { error } => {
                log::error!("envoy reported error during restore {error}");
                self.state = State::Idle;
                Err(Error::EnvoyError).whence()
            }
        })()
        .inspect_err(|_| {
            self.reset_state();
        })
    }

    pub fn get_downloading_progress(&self) -> Option<(u32, u32)> {
        match &self.state {
            State::Idle => None,
            State::InProgress { chunks_received, metadata, .. } => {
                Some((*chunks_received, metadata.total_chunks))
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use fs::adapter::test_utils::FsTest;
    use quantum_link::foundation_api::backup::{BackupChunk, RestoreMagicBackupEvent};

    use super::*;

    struct TestDownloader(BackupDownloader<FsTest>);

    impl Default for TestDownloader {
        fn default() -> Self { Self(BackupDownloader::new(FsTest::default())) }
    }

    impl TestDownloader {
        fn handle_event(
            &mut self,
            event: RestoreMagicBackupEvent,
        ) -> whence::Result<Option<Complete>, Error> {
            let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&event).unwrap();
            let archived =
                rkyv::access::<ArchivedRestoreMagicBackupEvent, rkyv::rancor::Error>(&bytes).unwrap();
            self.0.handle_event(archived)
        }

        pub fn get_downloading_progress(&self) -> Option<(u32, u32)> { self.0.get_downloading_progress() }
    }

    #[test]
    fn basic_flow() {
        let mut machine = TestDownloader::default();

        assert!(machine.get_downloading_progress().is_none());

        let metadata = BackupMetadata { total_chunks: 3 };
        let result = machine.handle_event(RestoreMagicBackupEvent::Starting(metadata));
        assert!(matches!(result, Ok(None)));
        assert_eq!(machine.get_downloading_progress(), Some((0, 3)));

        let chunk = BackupChunk { chunk_index: 0, total_chunks: 3, data: vec![1, 2, 3] };
        let result = machine.handle_event(RestoreMagicBackupEvent::Chunk(chunk));
        assert!(matches!(result, Ok(None)));
        assert_eq!(machine.get_downloading_progress(), Some((1, 3)));

        let chunk = BackupChunk { chunk_index: 1, total_chunks: 3, data: vec![4, 5, 6] };
        let result = machine.handle_event(RestoreMagicBackupEvent::Chunk(chunk));
        assert!(matches!(result, Ok(None)));
        assert_eq!(machine.get_downloading_progress(), Some((2, 3)));

        let chunk = BackupChunk { chunk_index: 2, total_chunks: 3, data: vec![7, 8, 9] };
        let result = machine.handle_event(RestoreMagicBackupEvent::Chunk(chunk));
        assert!(result.is_ok());
        let complete = result.unwrap();
        assert!(complete.is_some());
        assert!(complete.unwrap().file_path.contains("backup/restore.tar"));
    }

    #[test]
    fn out_of_order_chunk() {
        let mut machine = TestDownloader::default();

        let metadata = BackupMetadata { total_chunks: 3 };
        machine.handle_event(RestoreMagicBackupEvent::Starting(metadata)).unwrap();

        let chunk = BackupChunk { chunk_index: 1, total_chunks: 3, data: vec![4, 5, 6] };
        let result = machine.handle_event(RestoreMagicBackupEvent::Chunk(chunk));
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err().error, Error::InvalidChunk));
    }

    #[test]
    fn chunk_before_start() {
        let mut machine = TestDownloader::default();

        let chunk = BackupChunk { chunk_index: 0, total_chunks: 3, data: vec![1, 2, 3] };
        let result = machine.handle_event(RestoreMagicBackupEvent::Chunk(chunk));
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err().error, Error::InvalidState));
    }
    #[test]
    fn no_backup_found() {
        let mut machine = TestDownloader::default();

        let result = machine.handle_event(RestoreMagicBackupEvent::NotFound);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err().error, Error::NoBackupFound));
    }

    #[test]
    fn test_envoy_error() {
        let mut machine = TestDownloader::default();

        let result = machine.handle_event(RestoreMagicBackupEvent::Error { error: "test error".to_string() });
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err().error, Error::EnvoyError));
    }
}
