// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::{error::VaultError, IndexedSeedView, PasswordView, SeedView, SeedViewType},
    ordered_table::{SortableCard, TableEntry},
    serde::{Deserialize, Serialize},
    slint_keyos_platform::slint::SharedString,
    std::time::Duration,
};

#[derive(PartialEq, Debug, thiserror::Error)]
pub enum SeedValidationError {
    #[error("Invalid label, labels must not be empty")]
    InvalidLabelError,
    #[error("Invalid password, passwords must not be empty")]
    EmptyPasswordError,
}

#[derive(PartialEq, Debug, thiserror::Error)]
pub enum SeedDuplicateReason {
    #[error("Duplicate label: {0:?}")]
    Label(String),
    #[error("Duplicate 12 word seed with label {0:?}")]
    Bitcoin12(String),
    #[error("Duplicate 24 word seed with label {0:?}")]
    Bitcoin24(String),
    // Note: no current rules on account and password duplication, but these may be useful
    // #[error("Different password for existing account: {0:?}")]
    // DifferentPassword(String),
    // #[error("Same password for existing account: {0:?}")]
    // SamePassword(String),
    #[error("Duplicate Nostr key with label {0:?}")]
    NostrKey(String),
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum SeedType {
    Bitcoin12 { index: u32 },
    Bitcoin24 { index: u32 },
    Password { account: String, password: String },
    NostrKey { index: u32 },
}

impl Default for SeedType {
    fn default() -> Self { Self::Bitcoin12 { index: 0 } }
}

impl std::fmt::Debug for SeedType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bitcoin12 { .. } => write!(f, "Bitcoin12"),
            Self::Bitcoin24 { .. } => write!(f, "Bitcoin24"),
            Self::Password { .. } => write!(f, "Password"),
            Self::NostrKey { .. } => write!(f, "NostrKey"),
        }
    }
}

impl SeedType {
    pub fn from_view_type(
        seed_type: SeedViewType,
        seed_index: Option<SharedString>,
        account: Option<SharedString>,
        password: Option<SharedString>,
    ) -> Result<Self, VaultError> {
        let index: u32 = match (seed_type.is_indexed(), seed_index) {
            (true, Some(i)) => i.trim().parse::<u32>().unwrap_or(0),
            (true, None) => {
                return Err(VaultError::from(anyhow::anyhow!(
                    "Unable to make indexed seed type with an index"
                )))
            }
            (false, _) => 0u32, // Index is irrelevant, just leave it at 0 and ignore
        };

        let new_seed_type = match seed_type {
            SeedViewType::Bitcoin12 => SeedType::Bitcoin12 { index },
            SeedViewType::Bitcoin24 => SeedType::Bitcoin24 { index },
            SeedViewType::Password => {
                let account = account.map(String::from).unwrap_or(String::new());
                let password = password.map(String::from).unwrap_or(String::new());
                SeedType::Password { account, password }
            }
            SeedViewType::NostrKey => SeedType::NostrKey { index },
        };

        // Validate here to avoid repeated parameter validation in match body
        new_seed_type.validate()?;

        Ok(new_seed_type)
    }

    fn validate(&self) -> Result<(), SeedValidationError> {
        match self {
            SeedType::Password { account: _, password } => {
                SeedEditField::Password(password.clone()).validate()?;
            }
            _ => (),
        }

        Ok(())
    }

    // Delegate seed duplication check, but require label for nice error prints
    pub fn is_duplicate(&self, other: &Self, other_label: String) -> Option<SeedDuplicateReason> {
        match (self, other) {
            (SeedType::Bitcoin12 { index: index_a }, SeedType::Bitcoin12 { index: index_b })
                if index_a == index_b =>
            {
                return Some(SeedDuplicateReason::Bitcoin12(other_label));
            }
            (SeedType::Bitcoin24 { index: index_a }, SeedType::Bitcoin24 { index: index_b })
                if index_a == index_b =>
            {
                return Some(SeedDuplicateReason::Bitcoin24(other_label));
            }
            // Note: no current rules on account and password duplication, but these may be useful
            // (
            //     SeedType::Password { account: account_a, password: password_a },
            //     SeedType::Password { account: account_b, password: password_b },
            // ) => {
            //     match (account_a == account_b, password_a == password_b) {
            //         (true, true) => return Some(SeedDuplicateReason::SamePassword(account_a.clone())),
            //         (true, false) => return
            // Some(SeedDuplicateReason::DifferentPassword(account_a.clone())),         (false, _)
            // => (),         // Note: could add error/warning for password reuse
            //     }
            // }
            (SeedType::NostrKey { index: index_a }, SeedType::NostrKey { index: index_b })
                if index_a == index_b =>
            {
                return Some(SeedDuplicateReason::NostrKey(other_label));
            }
            (_, _) => (),
        }

        None
    }
}

// Always provide defaults for new values
// Requires debug to debug associated types in OrderedTable
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Seed {
    pub label: String,
    #[serde(default)]
    pub color: u8,
    #[serde(default)]
    pub archived: bool,
    #[serde(default)]
    date: u64,
    #[serde(default)]
    pub seed: SeedType,
}

impl TableEntry for Seed {
    type DuplicateReason = SeedDuplicateReason;
    type ValidationError = SeedValidationError;

    fn validate(&self) -> Result<(), Self::ValidationError> {
        SeedEditField::Label(self.label.clone()).validate()?;
        self.seed.validate()?;

        Ok(())
    }

    fn is_duplicate(&self, other: &Self) -> Option<Self::DuplicateReason> {
        if self.label == other.label {
            return Some(SeedDuplicateReason::Label(self.label.clone()));
        }

        self.seed.is_duplicate(&other.seed, other.label.clone())
    }
}

impl SortableCard for Seed {
    fn get_label(&self) -> &String { &self.label }

    fn get_date(&self) -> u64 { self.date }
}

#[repr(u32)]
pub enum SeedCategories {
    Active = 0,
    Archived,
}

impl Seed {
    pub fn new(seed: SeedType, label: String, color: u8, date: u64) -> Result<Self, SeedValidationError> {
        SeedEditField::Label(label.clone()).validate()?;
        seed.validate()?;

        let seed = Self { label, color, archived: false, date, seed };

        Ok(seed)
    }

    pub fn get_category(&self) -> u32 {
        (if self.archived { SeedCategories::Archived } else { SeedCategories::Active }) as u32
    }

    pub fn edit(&mut self, field: SeedEditField) -> Result<(), SeedValidationError> {
        field.validate()?;
        match (&mut self.seed, field) {
            (_, SeedEditField::Label(val)) => self.label = val,
            (SeedType::Password { ref mut account, password: _ }, SeedEditField::Account(val)) => {
                *account = val
            }
            (SeedType::Password { account: _, ref mut password }, SeedEditField::Password(val)) => {
                *password = val
            }
            _ => {
                log::warn!("Unsupported edit");
            }
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error, Clone)]
pub enum SeedEditField {
    #[error("label: {0:?}")]
    Label(String),
    #[error("account: {0:?}")]
    Account(String),
    #[error("password: {0:?}")]
    Password(String),
}

impl SeedEditField {
    pub fn validate(&self) -> Result<(), SeedValidationError> {
        match self {
            SeedEditField::Label(val) => {
                if val.is_empty() {
                    return Err(SeedValidationError::InvalidLabelError);
                }
            }
            SeedEditField::Password(val) => {
                if val.is_empty() {
                    return Err(SeedValidationError::EmptyPasswordError);
                }
            }
            _ => (),
        }

        Ok(())
    }
}

impl SeedView {
    pub fn from_seed(seed: &Seed) -> Self {
        let (seed_type, index, account, password) = match seed.seed {
            SeedType::Bitcoin12 { index } => (SeedViewType::Bitcoin12, index, String::new(), String::new()),
            SeedType::Bitcoin24 { index } => (SeedViewType::Bitcoin24, index, String::new(), String::new()),
            SeedType::Password { ref account, ref password } => {
                (SeedViewType::Password, 0u32, account.clone(), password.clone())
            }
            SeedType::NostrKey { index } => (SeedViewType::NostrKey, index, String::new(), String::new()),
        };
        Self {
            label: SharedString::from(seed.get_label()),
            color: seed.color as i32,
            seed_type,
            indexed_seed: IndexedSeedView { index: index.to_string().into() },
            password: PasswordView { account: account.into(), password: password.into() },
            index: -1,
        }
    }

    pub fn with_index(mut self, index: i32) -> Self {
        self.index = index;
        self
    }
}

impl SeedViewType {
    fn is_indexed(&self) -> bool {
        match self {
            SeedViewType::Bitcoin12 | SeedViewType::Bitcoin24 | SeedViewType::NostrKey => true,
            SeedViewType::Password => false,
            // Note, if you're adding a new type and hit a compiler error here,
            // make sure you update the parallel function in ui/callbacks.slint as well.
        }
    }
}

fn get_timestamp_in_seconds() -> u64 {
    #[cfg(not(test))]
    return std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_else(|e| {
            log::error!("Could not get time: {:?}", e);
            Duration::ZERO
        })
        .as_secs();
    #[cfg(test)]
    return 0;
}

impl Seed {
    pub fn from_view(seed_view: SeedView) -> Result<Self, VaultError> {
        let seed_type = SeedType::from_view_type(
            seed_view.seed_type,
            Some(seed_view.indexed_seed.index.clone()),
            Some(seed_view.password.account.clone()),
            Some(seed_view.password.password.clone()),
        )?;
        let time = get_timestamp_in_seconds();
        Ok(Self::new(seed_type, seed_view.label.clone().into(), seed_view.color as u8, time)?)
    }
}
