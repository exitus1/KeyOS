// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use fs::messages::CloseDir;
use slint_keyos_platform::{async_archive, async_scalar, spawn_local};

use crate::{fs_permissions::FileSystemPermissions, FileSystem};

pub mod copy_move;
pub mod list;

pub async fn rename_entry(
    fs: FileSystem,
    location: fs::Location,
    dir: &str,
    from_name: &str,
    to_name: &str,
) -> Result<(String, String), fs::Error> {
    let from = join_path(dir, from_name);
    let to = join_path(dir, to_name);
    let req = fs.rename_async(&from, &to, location)?;
    async_archive::<FileSystemPermissions, _>(req).await?;
    Ok((from, to))
}

pub async fn delete_all(fs: FileSystem, location: fs::Location, paths: Vec<String>) {
    let tasks = paths.into_iter().map(|path| {
        let fs = fs.clone();
        let task = spawn_local({
            let path = path.clone();
            async move {
                let req = fs.remove_async(&path, location)?;
                async_archive::<FileSystemPermissions, _>(req).await?;
                Ok::<_, fs::Error>(())
            }
        });
        (path, task)
    });

    for (path, task) in tasks {
        match task.await {
            Ok(()) => {
                log::info!("deleted file {path}")
            }
            Err(e) => {
                log::error!("delete failed: {path} {e:?}")
            }
        }
    }
}

pub async fn create_dir(
    fs: FileSystem,
    location: fs::Location,
    dir: &str,
    name: &str,
) -> Result<String, fs::Error> {
    let path = join_path(dir, name);
    let req = fs.create_dir_async(&path, location)?;
    let handle = async_archive::<FileSystemPermissions, _>(req).await?;
    let _ = async_scalar::<FileSystemPermissions, _>(CloseDir(handle)).await;
    Ok(path)
}

pub fn join_path(dir: &str, name: &str) -> String {
    let dir = dir.trim_end_matches('/');
    if dir.is_empty() {
        format!("/{name}")
    } else {
        format!("{dir}/{name}")
    }
}
