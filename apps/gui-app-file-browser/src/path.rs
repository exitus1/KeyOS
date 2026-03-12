// SPDX-FileCopyrightText: 2026 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later
use slint_keyos_platform::slint::SharedString;

/// normalized file path without leading/trailing slashes
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct FsPath(String);

impl FsPath {
    pub fn new(raw: impl AsRef<str>) -> Self {
        let normalized = raw.as_ref().trim_matches('/');
        Self(normalized.to_string())
    }

    pub fn as_str(&self) -> &str { &self.0 }

    pub fn to_shared_string(&self) -> SharedString { SharedString::from(self.0.as_str()) }

    pub fn join(&mut self, segment: &str) {
        let segment = segment.trim_matches('/');
        if segment.is_empty() {
            return;
        }
        if self.0.is_empty() {
            self.0 = segment.to_string();
        } else {
            self.0.push('/');
            self.0.push_str(segment);
        }
    }

    pub fn parent(&self) -> Option<Self> {
        if self.0.is_empty() {
            return None;
        }
        if let Some((parent, _)) = self.0.rsplit_once('/') {
            Some(Self(parent.to_string()))
        } else {
            Some(Self(String::new()))
        }
    }
}

impl std::fmt::Display for FsPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self.0) }
}

impl From<FsPath> for String {
    fn from(value: FsPath) -> Self { value.0 }
}

impl From<SharedString> for FsPath {
    fn from(value: SharedString) -> Self { FsPath::new(value.as_str()) }
}

impl AsRef<str> for FsPath {
    fn as_ref(&self) -> &str { self.0.as_str() }
}
