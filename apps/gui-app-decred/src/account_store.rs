// SPDX-License-Identifier: GPL-3.0-or-later
//
// Persistent named-account list (m/44'/42'/index'). The user creates accounts
// with a name (e.g. "Main", "Savings"); the device auto-increments the index.
// Seed/wallet creation lives in the OS Seed Vault, NOT here — this app only
// manages BIP44 sub-accounts within the active seed/passphrase context.
//
// Storage: the platform fs API at fs::Location::AppData (per-app sandboxed,
// works identically in the hosted simulator and on hardware). Writes go
// through FileBacked<String>, which gives atomic rename-into-place with a
// .old fallback, so a crash mid-write can never corrupt the account list.
//
// Format (line-based, no serde needed):
//   active:N
//   <index>\t<name>
//   <index>\t<name>

use slint_keyos_platform::{file_backed::FileBacked, fs};

use crate::fs_permissions::FileSystemPermissions;

/// Account list file inside this app's AppData sandbox.
const ACCOUNTS_FILE: &str = "accounts.txt";

#[derive(Clone, Debug)]
pub struct NamedAccount {
    pub index: u32,
    pub name: String,
}

#[derive(Clone, Debug, Default)]
pub struct AccountStore {
    pub accounts: Vec<NamedAccount>,
    pub active: u32,
    /// True for passphrase (hidden) wallets: keep the account list in memory
    /// only and never write it to disk, so hidden wallets leave no trace.
    pub ephemeral: bool,
}

impl AccountStore {
    /// Load the account list from AppData. An empty store means "first launch"
    /// (no accounts created yet), which the onboarding flow keys off of.
    pub fn load() -> Self {
        let mut store = AccountStore::default();
        match FileBacked::<String, FileSystemPermissions>::load(ACCOUNTS_FILE, fs::Location::AppData) {
            Ok(fb) => {
                for line in fb.lines() {
                    if let Some(rest) = line.strip_prefix("active:") {
                        store.active = rest.trim().parse().unwrap_or(0);
                    } else if let Some((idx, name)) = line.split_once('\t') {
                        if let Ok(i) = idx.trim().parse::<u32>() {
                            store.accounts.push(NamedAccount {
                                index: i,
                                name: name.to_string(),
                            });
                        }
                    }
                }
                log::info!(
                    "AccountStore: loaded {} account(s), active={} from AppData",
                    store.accounts.len(),
                    store.active,
                );
            }
            Err(_) => {
                log::info!("AccountStore: no saved accounts (first launch)");
            }
        }
        store
    }

    /// A fresh in-memory store for a hidden (passphrase) wallet: one default
    /// account, nothing ever written to disk.
    pub fn ephemeral() -> Self {
        AccountStore {
            accounts: vec![NamedAccount { index: 0, name: "Main".into() }],
            active: 0,
            ephemeral: true,
        }
    }

    /// Persist the account list to AppData (atomic write with .old fallback).
    /// No-op for hidden wallets.
    pub fn save(&self) {
        if self.ephemeral {
            return;
        }
        let mut out = format!("active:{}\n", self.active);
        for a in &self.accounts {
            out.push_str(&format!("{}\t{}\n", a.index, a.name));
        }
        let (mut fb, _restored) =
            FileBacked::<String, FileSystemPermissions>::new(ACCOUNTS_FILE, fs::Location::AppData);
        *fb.guard() = out;
        fb.save();
        log::info!("AccountStore: saved {} account(s) to AppData", self.accounts.len());
    }

    pub fn is_empty(&self) -> bool {
        self.accounts.is_empty()
    }

    /// The next free account index (max existing + 1, or 0 if none).
    pub fn next_index(&self) -> u32 {
        self.accounts.iter().map(|a| a.index).max().map_or(0, |m| m + 1)
    }

    /// Returns Some(error message) if the name is invalid, None if OK.
    pub fn validate_name(&self, name: &str) -> Option<&'static str> {
        let n = name.trim();
        if n.is_empty() {
            return Some("Name cannot be empty");
        }
        if self.accounts.iter().any(|a| a.name.eq_ignore_ascii_case(n)) {
            return Some("Name already used");
        }
        None
    }

    /// Create a new named account at the next index, persist, and make it
    /// active. Returns the new index, or Err(message) on validation failure.
    pub fn create(&mut self, name: &str) -> Result<u32, &'static str> {
        if let Some(e) = self.validate_name(name) {
            return Err(e);
        }
        let index = self.next_index();
        self.accounts.push(NamedAccount {
            index,
            name: name.trim().to_string(),
        });
        self.active = index;
        self.save();
        log::info!("AccountStore: created account {} \"{}\"", index, name.trim());
        Ok(index)
    }

    /// Switch the active account (only if it exists), and persist the choice.
    pub fn set_active(&mut self, index: u32) {
        if self.accounts.iter().any(|a| a.index == index) {
            self.active = index;
            self.save();
        }
    }

    /// Name of the active account (empty string if none).
    pub fn active_name(&self) -> String {
        self.accounts
            .iter()
            .find(|a| a.index == self.active)
            .map(|a| a.name.clone())
            .unwrap_or_default()
    }
}
