// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use {
    keyos_integration_test::{assert_eq, fail, pass},
    ordered_table::{FilePersistence, OrderedTable, TableEntry},
    serde::{Deserialize, Serialize},
};

fs::use_api!();

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MyString(String);

impl ToString for MyString {
    fn to_string(&self) -> String { self.0.clone() }
}

impl TableEntry for MyString {
    type DuplicateReason = ();
    type ValidationError = ();

    fn is_duplicate(&self, other: &Self) -> Option<Self::DuplicateReason> {
        if self.0 == other.0 {
            return Some(());
        }
        None
    }

    fn validate(&self) -> Result<(), Self::ValidationError> {
        if self.0.is_empty() {
            return Err(());
        }
        Ok(())
    }
}

const TEST_FILE: &str = "ordered_table_test.json";

fn main() {
    let string = MyString(String::from("A"));
    let mut ordered_table =
        match OrderedTable::<MyString, FilePersistence<fs_permissions::FileSystemPermissions>>::new()
            .with_persistence(FilePersistence::new(String::from(TEST_FILE), fs::Location::AppData))
        {
            Ok(t) => t,
            Err(e) => {
                fail!("Failed to open an empty OrderedTable file: {e}");
            }
        };

    match ordered_table.push(string) {
        Ok(_) => (),
        Err(e) => {
            fail!("Failed to push to empty OrderedTable: {e}");
        }
    };

    let new_table =
        match OrderedTable::<MyString, FilePersistence<fs_permissions::FileSystemPermissions>>::new()
            .with_persistence(FilePersistence::new(String::from(TEST_FILE), fs::Location::AppData))
        {
            Ok(t) => t,
            Err(e) => {
                fail!("Failed to open saved OrderedTable file: {e}");
            }
        };

    assert_eq!(new_table.len(), 1, "OrderedTable did not save properly");

    pass();
}
