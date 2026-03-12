// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::{Path, PathBuf};

const KEYOS_UI_LIBRARY_NAME: &str = "ui";

pub fn workspace_dir() -> PathBuf {
    let output = std::process::Command::new(env!("CARGO"))
        .arg("locate-project")
        .arg("--workspace")
        .arg("--message-format=plain")
        .output()
        .unwrap()
        .stdout;
    let cargo_path = Path::new(std::str::from_utf8(&output).unwrap().trim());
    cargo_path.parent().unwrap().to_path_buf()
}

pub fn library_paths() -> std::collections::HashMap<String, PathBuf> {
    std::collections::HashMap::from([(KEYOS_UI_LIBRARY_NAME.to_owned(), workspace_dir().join("ui/ui"))])
}

pub fn parse_nine_slice_filename(path: &Path) -> Option<(String, [u16; 4])> {
    let (image_name, nine_slice_str) = path.file_stem()?.to_str()?.split_once("__")?;
    let values: Vec<u16> = nine_slice_str.split('-').map(|s| s.parse().ok()).collect::<Option<Vec<_>>>()?;
    let ns_values: [u16; 4] = values.try_into().ok()?;
    Some((image_name.to_string(), ns_values))
}
