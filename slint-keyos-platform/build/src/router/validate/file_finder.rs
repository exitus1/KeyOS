// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{fs::ReadDir, path::PathBuf};

use miette::{Context, IntoDiagnostic};

// BFS iterator for finding all files with a given name
pub struct FileFinder {
    file_name: &'static str,
    dir_stack: Vec<PathBuf>,
    current_read_dir: Option<ReadDir>,
}

pub fn find_props_iter(path: impl Into<PathBuf>) -> FileFinder {
    FileFinder { file_name: "props.slint", dir_stack: vec![path.into()], current_read_dir: None }
}

pub fn find_page_iter(path: impl Into<PathBuf>) -> FileFinder {
    FileFinder { file_name: "page.slint", dir_stack: vec![path.into()], current_read_dir: None }
}

impl Iterator for FileFinder {
    type Item = Result<PathBuf, miette::Report>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(read_dir) = self.current_read_dir.as_mut() {
                match read_dir.next().transpose().into_diagnostic() {
                    Ok(Some(entry)) => {
                        let entry_path = entry.path();
                        if entry_path.is_file() {
                            if let Some(file_name) = entry_path.file_name() {
                                if file_name == self.file_name {
                                    return match entry_path.canonicalize().into_diagnostic() {
                                        Ok(path) => Some(Ok(path)),
                                        Err(e) => Some(Err(e)),
                                    };
                                }
                            }
                        } else if entry_path.is_dir() {
                            self.dir_stack.push(entry_path);
                        }
                    }
                    Ok(None) => self.current_read_dir = None,
                    Err(e) => return Some(Err(e)),
                }
            } else {
                match self.dir_stack.pop() {
                    Some(current_path) => match std::fs::read_dir(&current_path)
                        .into_diagnostic()
                        .wrap_err_with(|| format!("Failed to read directory: {}", current_path.display()))
                    {
                        Ok(read_dir) => self.current_read_dir = Some(read_dir),
                        Err(e) => {
                            return Some(Err(e));
                        }
                    },
                    None => return None,
                }
            }
        }
    }
}
