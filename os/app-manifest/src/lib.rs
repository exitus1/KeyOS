// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::collections::{BTreeMap, BTreeSet};
#[cfg(not(keyos))]
use std::path::Path;

use serde::{Deserialize, Serialize};

/// Full server manifest - contains all fields required for a server crate.
/// Used by `#[derive(Server)]`, `#[derive(Permissions)]` macros and xtask builder.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub app_name: BTreeMap<Locale, String>,
    pub app_id: String,
    #[serde(default)]
    pub servers: BTreeMap<String, BTreeMap<String, Message>>,
    #[serde(default)]
    pub permissions: BTreeMap<String, BTreeSet<String>>,
    #[serde(default)]
    pub memory: Vec<String>,
    #[serde(default)]
    pub syscall: Vec<String>,
}

/// Locale format, e.g. "en", "fr", etc.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct Locale(pub String);

impl From<String> for Locale {
    fn from(value: String) -> Self { Locale(value) }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub id: usize,
    pub r#type: MessageType,
    pub description: Option<String>,
    pub cfg: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MessageType {
    Move,
    Archive,
    ArchiveEvent,
    Scalar,
    BlockingScalar,
    ScalarEvent,
    LendMut,
    DeferredLendMut,
}

impl Manifest {
    #[cfg(not(keyos))]
    pub fn load(crate_dir: &Path, templates_dir: &Path) -> Self {
        Self::load_with_tracking(crate_dir, templates_dir, |_| {})
    }

    /// Load server manifest with tracking of all loaded paths
    #[cfg(not(keyos))]
    pub fn load_with_tracking(crate_dir: &Path, templates_dir: &Path, mut track: impl FnMut(&Path)) -> Self {
        load::load_server_manifest(crate_dir, templates_dir, &mut track)
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> { serde_json::from_slice(bytes) }

    pub fn app_id_bytes(&self) -> [u8; 16] { hex::decode(&self.app_id[2..]).unwrap().try_into().unwrap() }

    pub fn app_name_en(&self) -> String {
        self.app_name.get(&Locale("en".into())).cloned().unwrap_or("N/A".to_string())
    }
}

/// API-only manifest - contains only server message definitions.
/// Used by `#[derive(Message)]` macro.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiManifest {
    #[serde(default)]
    pub extends: Option<String>,
    #[serde(default)]
    pub servers: BTreeMap<String, BTreeMap<String, Message>>,
}

impl ApiManifest {
    /// Load API manifest with tracking of all loaded paths (for proc macro rebuild detection)
    #[cfg(not(keyos))]
    pub fn load_with_tracking(crate_dir: &Path, mut track: impl FnMut(&Path)) -> Self {
        load::load_api_manifest(crate_dir, &mut track)
    }
}

#[cfg(not(keyos))]
mod load {
    use serde::de::DeserializeOwned;

    use super::*;

    /// Load an API manifest
    pub fn load_api_manifest(crate_dir: &Path, track: &mut impl FnMut(&Path)) -> ApiManifest {
        let mut manifest: ApiManifest = load_manifest(crate_dir, track);

        if let Some(extends) = &manifest.extends {
            let extends = crate_dir.join(extends);
            let extends = std::fs::canonicalize(&extends)
                .unwrap_or_else(|e| panic!("Failed to resolve extends path {:?}: {:?}", extends, e));

            let extends_manifest = load_api_manifest(&extends, track);

            for (name, messages) in extends_manifest.servers {
                let entry = manifest.servers.entry(name).or_default();
                for (msg_name, msg) in messages {
                    entry.entry(msg_name).or_insert(msg);
                }
            }
        }

        manifest
    }

    /// Load a full server manifest
    pub fn load_server_manifest(
        crate_dir: &Path,
        templates_dir: &Path,
        track: &mut impl FnMut(&Path),
    ) -> Manifest {
        let mut manifest: Manifest = load_manifest(crate_dir, track);
        let api_manifest = load_api_manifest(crate_dir, track);

        manifest.servers = api_manifest.servers;

        expand_permission_templates(&mut manifest, templates_dir);

        // Convert hex data to lowercase for easy app ID matching
        manifest.app_id = manifest.app_id.to_lowercase();
        manifest
    }

    pub fn load_manifest<T: DeserializeOwned>(crate_dir: &Path, track: &mut impl FnMut(&Path)) -> T {
        let path = crate_dir.join("manifest.toml");
        track(&path);

        let file = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Failed to read manifest file at {:?}: {:?}", path, e));
        toml::from_str(&file)
            .unwrap_or_else(|e| panic!("Failed to parse manifest file at {:?}: {:?}", path, e))
    }

    /// Expand permission templates into actual permissions
    fn expand_permission_templates(manifest: &mut Manifest, templates_dir: &Path) {
        let path = templates_dir.join("permission_templates.toml");
        let template_file = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Failed to read permission template file at {:?}: {:?}", path, e));
        let templates: BTreeMap<String, BTreeMap<String, Vec<String>>> = toml::from_str(&template_file)
            .unwrap_or_else(|e| panic!("Failed to parse permission template file at {:?}: {:?}", path, e));

        if let Some(used_templates) = manifest.permissions.get_mut("template") {
            let mut remaining = BTreeSet::new();
            for template_name in used_templates.clone().iter() {
                let Some(additional_permissions) = templates.get(template_name) else {
                    remaining.insert(template_name.clone());
                    continue;
                };
                for (server_name, messages) in additional_permissions {
                    manifest
                        .permissions
                        .entry(server_name.clone())
                        .or_default()
                        .extend(messages.iter().cloned());
                }
            }
            if remaining.is_empty() {
                manifest.permissions.remove("template");
            } else {
                manifest.permissions.insert("template".into(), remaining);
            }
        }
    }
}
