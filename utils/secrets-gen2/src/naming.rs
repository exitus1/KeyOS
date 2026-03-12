// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::Path;

use anyhow::Result;
use regex::Regex;

use crate::error::SecretsGenError;

/// Naming convention
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NamingConvention {
    /// PascalCase (e.g., MyTestKey)
    Pascal,
    /// camelCase (e.g., myTestKey)
    Camel,
    /// snake_case (e.g., my_test_key)
    Snake,
    /// SCREAMING_SNAKE_CASE (e.g., MY_TEST_KEY)
    ScreamingSnake,
    /// kebab-case (e.g., my-test-key)
    Kebab,
}

/// Get the appropriate naming convention for a file type
pub fn get_naming_convention_for_file_type(file_path: &Path) -> NamingConvention {
    if let Some(extension) = file_path.extension() {
        match extension.to_str().unwrap_or("").to_lowercase().as_str() {
            "toml" => NamingConvention::Kebab,
            "env" => NamingConvention::ScreamingSnake,
            "rs" => NamingConvention::Snake,
            _ => NamingConvention::Pascal,
        }
    } else {
        NamingConvention::Pascal
    }
}

/// Convert a name to the specified naming convention
pub fn convert_name(name: &str, convention: NamingConvention) -> Result<String> {
    // First, split the name into words
    let words = split_into_words(name)?;

    // Then, convert to the specified convention
    match convention {
        NamingConvention::Pascal => Ok(to_pascal_case(&words)),
        NamingConvention::Camel => Ok(to_camel_case(&words)),
        NamingConvention::Snake => Ok(to_snake_case(&words)),
        NamingConvention::ScreamingSnake => Ok(to_screaming_snake_case(&words)),
        NamingConvention::Kebab => Ok(to_kebab_case(&words)),
    }
}

/// Process a name specification
pub fn process_name_spec(name_spec: &str, key_name: &str, _file_path: &Path) -> Result<String> {
    if name_spec.starts_with('$') {
        let convention = match &name_spec[1..] {
            "pascal" => NamingConvention::Pascal,
            "camel" => NamingConvention::Camel,
            "snake" => NamingConvention::Snake,
            "screaming-snake" => NamingConvention::ScreamingSnake,
            "kebab" => NamingConvention::Kebab,
            _ => return Err(SecretsGenError::InvalidNamingConvention(name_spec.to_string()).into()),
        };
        convert_name(key_name, convention)
    } else {
        Ok(name_spec.to_string())
    }
}

/// Get the name to use for a key in a file
pub fn get_name_for_key(key_name: &str, name_spec: Option<&str>, file_path: &Path) -> Result<String> {
    if let Some(name) = name_spec {
        process_name_spec(name, key_name, file_path)
    } else {
        let convention = get_naming_convention_for_file_type(file_path);
        convert_name(key_name, convention)
    }
}

/// Split a name into words
fn split_into_words(name: &str) -> Result<Vec<String>> {
    let mut words = Vec::new();

    // Check if it's PascalCase or camelCase
    let pascal_camel_regex = Regex::new(r"([A-Z][a-z0-9]*)")?;
    if pascal_camel_regex.is_match(name) {
        // Handle camelCase (first word is lowercase)
        let first_word_regex = Regex::new(r"^([a-z][a-z0-9]*)")?;
        if let Some(captures) = first_word_regex.captures(name) {
            if let Some(first_word) = captures.get(1) {
                words.push(first_word.as_str().to_lowercase());
            }
        }

        // Handle remaining PascalCase words
        for captures in pascal_camel_regex.captures_iter(name) {
            if let Some(word) = captures.get(1) {
                words.push(word.as_str().to_lowercase());
            }
        }
    }
    // Check if it's snake_case or SCREAMING_SNAKE_CASE
    else if name.contains('_') {
        for word in name.split('_') {
            if !word.is_empty() {
                words.push(word.to_lowercase());
            }
        }
    }
    // Check if it's kebab-case
    else if name.contains('-') {
        for word in name.split('-') {
            if !word.is_empty() {
                words.push(word.to_lowercase());
            }
        }
    }
    // If it's a single word
    else {
        words.push(name.to_lowercase());
    }

    Ok(words)
}

/// Convert words to PascalCase
fn to_pascal_case(words: &[String]) -> String {
    words
        .iter()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}

/// Convert words to camelCase
fn to_camel_case(words: &[String]) -> String {
    if words.is_empty() {
        return String::new();
    }

    let mut result = words[0].clone();
    for word in &words[1..] {
        let mut chars = word.chars();
        match chars.next() {
            None => {}
            Some(first) => {
                result.push_str(&(first.to_uppercase().collect::<String>() + chars.as_str()));
            }
        }
    }

    result
}

/// Convert words to snake_case
fn to_snake_case(words: &[String]) -> String { words.join("_") }

/// Convert words to SCREAMING_SNAKE_CASE
fn to_screaming_snake_case(words: &[String]) -> String {
    words.iter().map(|word| word.to_uppercase()).collect::<Vec<String>>().join("_")
}

/// Convert words to kebab-case
fn to_kebab_case(words: &[String]) -> String { words.join("-") }

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_split_into_words() {
        assert_eq!(split_into_words("MyTestKey").unwrap(), vec!["my", "test", "key"]);
        assert_eq!(split_into_words("myTestKey").unwrap(), vec!["my", "test", "key"]);
        assert_eq!(split_into_words("my_test_key").unwrap(), vec!["my", "test", "key"]);
        assert_eq!(split_into_words("MY_TEST_KEY").unwrap(), vec!["my", "test", "key"]);
        assert_eq!(split_into_words("my-test-key").unwrap(), vec!["my", "test", "key"]);
    }

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case(&["my".to_string(), "test".to_string(), "key".to_string()]), "MyTestKey");
    }

    #[test]
    fn test_to_camel_case() {
        assert_eq!(to_camel_case(&["my".to_string(), "test".to_string(), "key".to_string()]), "myTestKey");
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case(&["my".to_string(), "test".to_string(), "key".to_string()]), "my_test_key");
    }

    #[test]
    fn test_to_screaming_snake_case() {
        assert_eq!(
            to_screaming_snake_case(&["my".to_string(), "test".to_string(), "key".to_string()]),
            "MY_TEST_KEY"
        );
    }

    #[test]
    fn test_to_kebab_case() {
        assert_eq!(to_kebab_case(&["my".to_string(), "test".to_string(), "key".to_string()]), "my-test-key");
    }

    #[test]
    fn test_convert_name() {
        assert_eq!(convert_name("MyTestKey", NamingConvention::Pascal).unwrap(), "MyTestKey");
        assert_eq!(convert_name("MyTestKey", NamingConvention::Camel).unwrap(), "myTestKey");
        assert_eq!(convert_name("MyTestKey", NamingConvention::Snake).unwrap(), "my_test_key");
        assert_eq!(convert_name("MyTestKey", NamingConvention::ScreamingSnake).unwrap(), "MY_TEST_KEY");
        assert_eq!(convert_name("MyTestKey", NamingConvention::Kebab).unwrap(), "my-test-key");
    }

    #[test]
    fn test_get_naming_convention_for_file_type() {
        assert_eq!(get_naming_convention_for_file_type(&PathBuf::from("test.toml")), NamingConvention::Kebab);
        assert_eq!(
            get_naming_convention_for_file_type(&PathBuf::from("test.env")),
            NamingConvention::ScreamingSnake
        );
        assert_eq!(get_naming_convention_for_file_type(&PathBuf::from("test.rs")), NamingConvention::Snake);
        assert_eq!(get_naming_convention_for_file_type(&PathBuf::from("test")), NamingConvention::Pascal);
    }

    #[test]
    fn test_process_name_spec() {
        assert_eq!(process_name_spec("$pascal", "MyTestKey", &PathBuf::from("test")).unwrap(), "MyTestKey");
        assert_eq!(process_name_spec("$camel", "MyTestKey", &PathBuf::from("test")).unwrap(), "myTestKey");
        assert_eq!(process_name_spec("$snake", "MyTestKey", &PathBuf::from("test")).unwrap(), "my_test_key");
        assert_eq!(
            process_name_spec("$screaming-snake", "MyTestKey", &PathBuf::from("test")).unwrap(),
            "MY_TEST_KEY"
        );
        assert_eq!(process_name_spec("$kebab", "MyTestKey", &PathBuf::from("test")).unwrap(), "my-test-key");
        assert_eq!(
            process_name_spec("custom-name", "MyTestKey", &PathBuf::from("test")).unwrap(),
            "custom-name"
        );
    }

    #[test]
    fn test_get_name_for_key() {
        assert_eq!(
            get_name_for_key("MyTestKey", Some("$pascal"), &PathBuf::from("test")).unwrap(),
            "MyTestKey"
        );
        assert_eq!(
            get_name_for_key("MyTestKey", Some("custom-name"), &PathBuf::from("test")).unwrap(),
            "custom-name"
        );
        assert_eq!(get_name_for_key("MyTestKey", None, &PathBuf::from("test.toml")).unwrap(), "my-test-key");
        assert_eq!(get_name_for_key("MyTestKey", None, &PathBuf::from("test.env")).unwrap(), "MY_TEST_KEY");
        assert_eq!(get_name_for_key("MyTestKey", None, &PathBuf::from("test.rs")).unwrap(), "my_test_key");
    }
}
