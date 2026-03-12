// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::{Read, Write};

use backup::DO_NOT_BACKUP_FOLDER;

use crate::FileSystem;

pub trait Persister: Sized {
    type Error: From<std::io::Error> + From<fs::Error> + std::fmt::Display;

    fn from_bytes(data: &[u8]) -> Result<Self, Self::Error>;
    fn to_bytes(&self) -> Result<Vec<u8>, Self::Error>;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum PersistenceState {
    Dirty,
    SavedToSystem,
    SavedToAppdata,
}

/// A wrapper around a value, persisted to a file.
///
/// The only way to get access to the inner value is via [`FileBacked::as_ref()`] or [`FileBacked::as_mut()`]
///
/// [`FileBacked::as_mut()`] will return a [`FileBackedGuard`], which will mark the file as dirty on a
/// mutation and when the guard is dropped, the file will be saved.
pub struct FileBacked<T: Persister> {
    fs: FileSystem,
    path: String,
    state: PersistenceState,
    value: T,
}

impl<T: Persister> FileBacked<T> {
    pub fn get_or_init(path: String, init: impl FnOnce() -> Result<T, T::Error>) -> Result<Self, T::Error> {
        let fs = FileSystem::default();

        let read_file = |path: String, location, state| -> Result<(T, PersistenceState), T::Error> {
            let mut file = fs.open_file(path, location, fs::OpenFlags::READ_ONLY)?;
            let mut data = vec![];
            file.read_to_end(&mut data)?;
            let value = T::from_bytes(&data)?;
            Ok((value, state))
        };

        let (value, state) =
            read_file(app_data_path(&path), fs::Location::AppData, PersistenceState::SavedToAppdata)
                .or_else(|e| {
                    log::info!("Could not read state file in AppData ({e}, trying System");
                    read_file(path.clone(), fs::Location::SystemAppData, PersistenceState::SavedToSystem)
                })
                .or_else(|_e| -> Result<_, T::Error> { Ok((init()?, PersistenceState::Dirty)) })?;

        let mut state = Self { fs, value, path, state };
        state.save()?;
        Ok(state)
    }

    fn wipe_file_in_system(&self) -> Result<(), fs::Error> {
        let Ok(meta) = self.fs.metadata(&self.path, fs::Location::SystemAppData) else {
            return Ok(());
        };
        // Overwrite the whole cluster with zeroes before deleting it will make sure
        // there are no traces of the original data left (at least from the MCU's
        // perspective over the SD interface), because fatfs reuses clusters when
        // writing data.
        {
            let mut file =
                self.fs.open_file(&self.path, fs::Location::SystemAppData, fs::OpenFlags::READ_WRITE)?;
            file.write_all(&vec![0; meta.size.next_multiple_of(64 * 1024) as usize])?;
        }

        self.fs.remove(&self.path, fs::Location::SystemAppData).ok();
        log::info!("Wiped state file from System");

        Ok(())
    }

    pub fn save(&mut self) -> Result<(), T::Error> {
        if self.state == PersistenceState::SavedToAppdata {
            return Ok(());
        }
        let write_value_to_file = |mut file: crate::File| -> Result<(), T::Error> {
            file.write_all(&self.value.to_bytes()?)?;
            file.truncate()?;
            Ok(())
        };

        let _ = self.fs.create_dir(DO_NOT_BACKUP_FOLDER, fs::Location::AppData);
        let app_data_path = app_data_path(&self.path);
        let _ = self.fs.ensure_parent_dir_exists(&app_data_path, fs::Location::AppData);

        if let Ok(file) = self.fs.open_file(&app_data_path, fs::Location::AppData, fs::OpenFlags::CREATE) {
            write_value_to_file(file)?;
            self.state = PersistenceState::SavedToAppdata;
            // We successfully saved to appdata. If we still have the state file in system,
            // securely wipe it.
            if let Err(e) = self.wipe_file_in_system() {
                log::error!("Could not wipe old state file: {e:?}");
            }
        } else {
            if self.state == PersistenceState::SavedToSystem {
                return Ok(());
            }
            log::debug!("Could not create {} in AppData, trying System", self.path);
            self.fs.ensure_parent_dir_exists(&self.path, fs::Location::SystemAppData)?;
            let file = self.fs.open_file(&self.path, fs::Location::SystemAppData, fs::OpenFlags::CREATE)?;
            write_value_to_file(file)?;
            self.state = PersistenceState::SavedToSystem;
        };
        Ok(())
    }

    pub fn mark_dirty(&mut self) { self.state = PersistenceState::Dirty }

    pub fn guard(&mut self) -> FileBackedGuard<'_, T> { FileBackedGuard { inner: self } }
}

fn app_data_path(path: &str) -> String { format!("{DO_NOT_BACKUP_FOLDER}/{path}") }

impl<T: Persister> Drop for FileBacked<T> {
    fn drop(&mut self) {
        if let Err(e) = self.save() {
            log::error!("Failed to save file: {}", e);
        }
    }
}

/// A guard that marks the [`FileBacked`] as dirty upon a mutation
/// if the value is mutated, the value will be persisted when the guard is dropped
pub struct FileBackedGuard<'a, T: Persister> {
    inner: &'a mut FileBacked<T>,
}

impl<T: Persister> std::ops::Deref for FileBackedGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target { &self.inner.value }
}

impl<T: Persister> std::ops::DerefMut for FileBackedGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.state = PersistenceState::Dirty;
        &mut self.inner.value
    }
}

impl<T: Persister> Drop for FileBackedGuard<'_, T> {
    fn drop(&mut self) {
        if let Err(e) = self.inner.save() {
            log::error!("Failed to save file: {}", e);
        }
    }
}
