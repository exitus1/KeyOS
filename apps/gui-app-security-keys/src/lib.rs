// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use {
    ordered_table::{SortableCard, TableEntry},
    serde::{Deserialize, Serialize},
};

pub const DATABASE_FILE: &str = "security_key_database_v1.json";

#[derive(PartialEq, Debug, thiserror::Error)]
pub enum KeyDuplicateReason {
    #[error("Duplicate label: {0:?}")]
    Label(String),
    #[error("Duplicate index with label: {0:?}")]
    Index(String),
}

#[derive(PartialEq, Debug, thiserror::Error)]
pub enum KeyValidationError {
    #[error("Invalid label, labels must not be empty")]
    InvalidLabelError,
}

#[repr(u32)]
pub enum KeyCategories {
    Active = 0,
    Archived,
}

// Always provide defaults for new values
// Requires debug to debug associated types in OrderedTable
// Be careful not to debug log the whole thing with private TOTP keys
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Key {
    key_index: usize,
    label: String,
    #[serde(default)]
    pub color: u8,
    #[serde(default)]
    pub archived: bool,
    #[serde(default)]
    date: u64,
    #[serde(default)]
    pub icon: String,
}

impl TableEntry for Key {
    type DuplicateReason = KeyDuplicateReason;
    type ValidationError = KeyValidationError;

    fn validate(&self) -> Result<(), Self::ValidationError> {
        KeyEditField::Label(self.label.clone()).validate()?;
        Ok(())
    }

    fn is_duplicate(&self, other: &Self) -> Option<Self::DuplicateReason> {
        if self.key_index == other.key_index {
            return Some(KeyDuplicateReason::Index(other.label.clone()));
        }

        if self.label == other.label {
            return Some(KeyDuplicateReason::Label(self.label.clone()));
        }

        None
    }
}

impl SortableCard for Key {
    fn get_label(&self) -> &String { &self.label }

    fn get_date(&self) -> u64 { self.date }
}

impl Key {
    pub fn new(
        key_index: usize,
        label: String,
        color: u8,
        date: u64,
        icon: String,
    ) -> Result<Self, KeyValidationError> {
        KeyEditField::Label(label.clone()).validate()?;

        let key = Self { key_index, label, color, archived: false, date, icon };
        Ok(key)
    }

    pub fn edit(&mut self, field: KeyEditField) -> Result<(), KeyValidationError> {
        field.validate()?;
        match field {
            KeyEditField::Label(val) => self.label = val,
        }

        Ok(())
    }

    pub fn get_index(&self) -> usize { self.key_index }

    pub fn get_category(&self) -> u32 {
        (if self.archived { KeyCategories::Archived } else { KeyCategories::Active }) as u32
    }
}

#[derive(Debug, thiserror::Error, Clone)]
pub enum KeyEditField {
    #[error("label: {0:?}")]
    Label(String),
}

impl KeyEditField {
    pub fn validate(&self) -> Result<(), KeyValidationError> {
        match self {
            KeyEditField::Label(val) => {
                if val.len() == 0 {
                    return Err(KeyValidationError::InvalidLabelError);
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key1() -> Result<Key, KeyValidationError> { Ok(Key::new(0, String::from("A"), 0, 0, String::new())?) }

    fn key2() -> Result<Key, KeyValidationError> { Ok(Key::new(0, String::from("B"), 0, 0, String::new())?) }

    fn key3() -> Result<Key, KeyValidationError> { Ok(Key::new(1, String::from("B"), 0, 0, String::new())?) }

    #[test]
    fn create_key() {
        let key = key1().unwrap();
        key.validate().unwrap();
        assert_eq!(key.label, String::from("A"));
    }

    #[test]
    fn validate_key_no_label() {
        let e = Key::new(0, String::from(""), 0, 0, String::new()).unwrap_err();
        assert_eq!(e, KeyValidationError::InvalidLabelError);
    }

    #[test]
    fn not_equal() {
        let key1 = key1().unwrap();
        let key3 = key3().unwrap();
        assert!(key1.is_duplicate(&key3).is_none());
    }

    #[test]
    fn same_index_priority() {
        let key1 = key1().unwrap();
        assert_eq!(key1.is_duplicate(&key1).unwrap(), KeyDuplicateReason::Index(String::from("A")));
    }

    #[test]
    fn same_index() {
        let key1 = key1().unwrap();
        let key2 = key2().unwrap();
        assert_eq!(key1.is_duplicate(&key2).unwrap(), KeyDuplicateReason::Index(String::from("B")));
    }

    #[test]
    fn same_label() {
        let key2 = key2().unwrap();
        let key3 = key3().unwrap();
        assert_eq!(key2.is_duplicate(&key3).unwrap(), KeyDuplicateReason::Label(String::from("B")));
    }

    #[test]
    fn edit_label() {
        let mut key1 = key1().unwrap();
        let field = KeyEditField::Label(String::from("C"));
        key1.edit(field).unwrap();
        assert_eq!(key1.label, String::from("C"));
    }
}
