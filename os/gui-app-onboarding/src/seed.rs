// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Seed-related utilities for BIP39 mnemonics and SeedQR format handling

/// Converts a seed to BIP39 mnemonic words
pub fn seed_to_words(seed: &security::Seed) -> Result<Vec<String>, bip39::Error> {
    // Convert to mnemonic (32 bytes)
    let mnemonic = bip39::Mnemonic::from_entropy(seed.bytes())?;

    // Convert to word list
    let words: Vec<String> = mnemonic.words().map(|word| word.to_string()).collect();

    Ok(words)
}

/// Converts a BIP39 mnemonic to a seed
pub fn mnemonic_to_seed(mnemonic: &bip39::Mnemonic) -> security::Seed {
    let entropy = mnemonic.to_entropy();
    security::Seed::from_bytes(&entropy)
}

/// Error type for parse_seedqr function
#[derive(Clone, Debug, thiserror::Error)]
pub enum ParseSeedQrError {
    #[error("Invalid UTF-8 in word index: {0}")]
    InvalidUtf8(#[from] std::str::Utf8Error),

    #[error("Failed to parse word index: {0}")]
    InvalidWordIndex(#[from] std::num::ParseIntError),

    #[error("Word index {0} out of range")]
    WordIndexOutOfRange(usize),

    #[error("Invalid mnemonic: {0}")]
    InvalidMnemonic(#[from] bip39::Error),
}

/// Parse Standard/Compact SeedQR format
/// https://github.com/SeedSigner/seedsigner/blob/dev/docs/seed_qr/README.md
pub fn parse_seedqr(qr_data: &[u8]) -> Result<bip39::Mnemonic, ParseSeedQrError> {
    // 12 or 24 word standard qr
    if qr_data.len() == 48 || qr_data.len() == 96 {
        let words = qr_data
            .chunks(4)
            .map(|index| -> Result<&'static str, ParseSeedQrError> {
                let index_str = std::str::from_utf8(index)?;
                let index: usize = index_str.parse()?;
                let word = bip39::Language::English
                    .word_list()
                    .get(index)
                    .copied()
                    .ok_or(ParseSeedQrError::WordIndexOutOfRange(index))?;
                Ok(word)
            })
            .collect::<Result<Vec<&'static str>, _>>()?
            .join(" ");

        return bip39::Mnemonic::parse(words.as_str()).map_err(|e| e.into());
    }

    if let Ok(text) = std::str::from_utf8(qr_data) {
        if let Ok(mnemonic) = bip39::Mnemonic::parse_normalized(&text) {
            return Ok(mnemonic);
        }
    }

    bip39::Mnemonic::from_entropy(qr_data).map_err(|e| e.into())
}

/// Generate standard SeedQR format data (4-digit padded indices)
pub fn generate_standard_seed_qr_data(seed: &security::Seed) -> Result<Vec<u8>, bip39::Error> {
    let mnemonic = bip39::Mnemonic::from_entropy(seed.bytes())?;

    // Create standard SeedQR format: 4-digit padded indices
    let indices: String = mnemonic.word_indices().map(|idx| format!("{idx:04}")).collect();

    Ok(indices.into_bytes())
}

/// Generate compact SeedQR format data (raw entropy bytes)
pub fn generate_compact_seed_qr_data(seed: &security::Seed) -> Result<Vec<u8>, bip39::Error> {
    let mnemonic = bip39::Mnemonic::from_entropy(seed.bytes())?;
    Ok(mnemonic.to_entropy())
}

/// Represents a seed word verification challenge
#[derive(Clone, Debug, PartialEq)]
pub struct SeedWordChallenge {
    /// Which word position we're verifying (0-based)
    pub word_index: usize,
    /// The 4 multiple choice options
    pub options: [String; 4],
    /// Which option is correct (0-3)
    pub correct_option_index: usize,
}

/// Generate seed word verification challenges
///
/// # Arguments
/// * `seed` - The seed to generate challenges for
/// * `num_challenges` - Number of challenges to generate (will be capped at number of seed words)
///
/// # Returns
/// A vector of challenges, each containing a word index and 4 multiple choice options
pub fn generate_seed_word_challenge(
    seed: &security::Seed,
    num_challenges: usize,
) -> Result<Vec<SeedWordChallenge>, bip39::Error> {
    const NUM_OPTIONS: usize = 4;

    if num_challenges == 0 {
        return Ok(Vec::new());
    }

    let mnemonic = bip39::Mnemonic::from_entropy(seed.bytes())?;

    let words: Vec<&str> = mnemonic.words().collect();
    let word_list = bip39::Language::English.word_list();

    let num_challenges = num_challenges.min(words.len());

    let mut indices: Vec<usize> = (0..words.len()).collect();
    shuffle(&mut indices);

    let mut challenges = Vec::new();

    for &word_index in &indices[..num_challenges] {
        let correct_word = words[word_index];

        // generate 3 incorrect options
        let mut option_list = vec![correct_word.to_string()];
        let mut used_indices = vec![word_index];

        if let Some(correct_word_bip39_index) = word_list.iter().position(|&w| w == correct_word) {
            if !used_indices.contains(&correct_word_bip39_index) {
                used_indices.push(correct_word_bip39_index);
            }
        }

        // generate 3 random incorrect words
        while option_list.len() < NUM_OPTIONS {
            let random_index = random_index(word_list.len());
            if !used_indices.contains(&random_index) {
                used_indices.push(random_index);
                option_list.push(word_list[random_index].to_string());
            }
        }

        shuffle(&mut option_list);

        let correct_option_index = option_list.iter().position(|w| w.as_str() == correct_word).unwrap();

        let options: [String; NUM_OPTIONS] =
            option_list.try_into().expect("option_list should have exactly 4 elements");

        challenges.push(SeedWordChallenge { word_index, options, correct_option_index });
    }

    Ok(challenges)
}

/// Generate random index within range
fn random_index(max: usize) -> usize {
    if max == 0 {
        return 0;
    }

    let mut bytes = [0u8; 4];
    getrandom::getrandom(&mut bytes).expect("failed to get random bytes");
    let random_u32 = u32::from_le_bytes(bytes);
    (random_u32 as usize) % max
}

/// shuffle a vector in place
fn shuffle<T>(vec: &mut [T]) {
    for i in (1..vec.len()).rev() {
        let j = random_index(i + 1);
        vec.swap(i, j);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_seed() -> security::Seed {
        let mut seed_bytes = [0u8; 16];
        getrandom::getrandom(&mut seed_bytes).unwrap();
        security::Seed::Twelve(seed_bytes)
    }

    #[test]
    fn test_mnemonic_to_seed_roundtrip() {
        let original_seed = create_test_seed();
        let mnemonic = bip39::Mnemonic::from_entropy(original_seed.bytes()).unwrap();
        let recovered_seed = mnemonic_to_seed(&mnemonic).to_vec();

        // The recovered seed should have the same entropy bytes
        assert_eq!(
            &original_seed.bytes()[..mnemonic.to_entropy().len()],
            &recovered_seed[..mnemonic.to_entropy().len()]
        );
    }

    #[test]
    fn test_parse_seedqr_standard_12_word() {
        // Standard SeedQR format: 48 bytes (12 words * 4)
        let qr_data = b"192402220235174306311124037817700641198012901210";

        let result = parse_seedqr(qr_data).unwrap().word_indices().collect::<Vec<_>>();

        let expected = vec![1924, 222, 235, 1743, 631, 1124, 378, 1770, 641, 1980, 1290, 1210];

        assert_eq!(result, expected, "Word indices should match expected values");
    }

    #[test]
    fn test_parse_seedqr_standard_24_word() {
        // Create a valid 24-word mnemonic first
        let mut entropy = [0u8; 32];
        getrandom::getrandom(&mut entropy).unwrap();
        let mnemonic = bip39::Mnemonic::from_entropy(&entropy).unwrap();

        // Generate standard QR data from it
        let indices: String = mnemonic.word_indices().map(|idx| format!("{:04}", idx)).collect();
        let qr_data = indices.as_bytes();

        // Parse it back
        let result = parse_seedqr(qr_data).unwrap();

        // Verify it matches the original
        assert_eq!(result, mnemonic);
        assert_eq!(result.word_count(), 24);
    }

    #[test]
    fn test_parse_seedqr_compact() {
        fn test(entropy: &mut [u8]) {
            getrandom::getrandom(entropy).unwrap();
            let mnemonic = bip39::Mnemonic::from_entropy(entropy).unwrap();
            let result = parse_seedqr(entropy).unwrap();
            assert_eq!(result, mnemonic);
        }

        let mut entropy_12 = [0u8; 16];
        test(&mut entropy_12);

        let mut entropy_24 = [0u8; 32];
        test(&mut entropy_24);
    }

    #[test]
    fn test_parse_seedqr_plaintext() {
        let mut entropy = [0u8; 16];
        getrandom::getrandom(&mut entropy).unwrap();
        let mnemonic = bip39::Mnemonic::from_entropy(&entropy).unwrap();

        let qr_data = mnemonic.to_string();
        let result = parse_seedqr(qr_data.as_bytes()).unwrap();

        assert_eq!(result, mnemonic);
    }

    #[test]
    fn test_parse_seedqr_plaintext_with_extra_whitespace() {
        let mut entropy = [0u8; 32];
        getrandom::getrandom(&mut entropy).unwrap();
        let mnemonic = bip39::Mnemonic::from_entropy(&entropy).unwrap();
        let words = mnemonic.words().collect::<Vec<_>>();
        let qr_data = format!("  {}\n{}\n  ", words[..12].join("  "), words[12..].join("\n"));

        let result = parse_seedqr(qr_data.as_bytes()).unwrap();

        assert_eq!(result, mnemonic);
    }

    #[test]
    fn test_seedqr_generation_roundtrip() {
        let seed = create_test_seed();

        // Test standard format roundtrip
        let standard_data = generate_standard_seed_qr_data(&seed).unwrap();
        let parsed_standard = parse_seedqr(&standard_data).unwrap();
        let recovered_seed = mnemonic_to_seed(&parsed_standard).to_vec();
        assert_eq!(
            &seed.bytes()[..parsed_standard.to_entropy().len()],
            &recovered_seed[..parsed_standard.to_entropy().len()]
        );

        // Test compact format roundtrip
        let compact_data = generate_compact_seed_qr_data(&seed).unwrap();
        let parsed_compact = parse_seedqr(&compact_data).unwrap();
        let recovered_seed = mnemonic_to_seed(&parsed_compact).to_vec();
        assert_eq!(
            &seed.bytes()[..parsed_compact.to_entropy().len()],
            &recovered_seed[..parsed_compact.to_entropy().len()]
        );
    }

    #[test]
    fn test_generate_seed_word_challenge_various_counts() {
        let seed = create_test_seed();

        // Test with different challenge counts
        for num_challenges in [0, 1, 4, 6, 12] {
            let challenges = generate_seed_word_challenge(&seed, num_challenges).unwrap();
            assert_eq!(
                challenges.len(),
                num_challenges,
                "Expected {} challenges, got {}",
                num_challenges,
                challenges.len()
            );
        }

        // Test with more challenges than available words
        let challenges = generate_seed_word_challenge(&seed, 100).unwrap();
        assert!(challenges.len() <= 24); // Should be capped at number of seed words
    }

    #[test]
    fn test_challenge_validity() {
        let seed = create_test_seed();
        let challenges = generate_seed_word_challenge(&seed, 4).unwrap();

        // Get the actual seed words for validation
        let seed_words = seed_to_words(&seed).unwrap();

        for challenge in &challenges {
            // Each challenge should have 4 options
            assert_eq!(challenge.options.len(), 4);

            // Correct option index should be valid
            assert!(
                challenge.correct_option_index < 4,
                "Invalid correct option index: {}",
                challenge.correct_option_index
            );

            // Word index should be valid for the seed
            assert!(challenge.word_index < seed_words.len(), "Invalid word index: {}", challenge.word_index);

            // The correct option should match the actual seed word
            let correct_option = &challenge.options[challenge.correct_option_index];
            let expected_word = &seed_words[challenge.word_index];
            assert_eq!(correct_option, expected_word, "Correct option doesn't match seed word");
        }
    }

    #[test]
    fn test_no_duplicate_options() {
        let seed = create_test_seed();
        let challenges = generate_seed_word_challenge(&seed, 4).unwrap();

        for challenge in &challenges {
            // Check for duplicates using a simple nested loop
            for i in 0..4 {
                for j in (i + 1)..4 {
                    assert_ne!(
                        challenge.options[i], challenge.options[j],
                        "Duplicate option found: {}",
                        challenge.options[i]
                    );
                }
            }
        }
    }

    #[test]
    fn test_random_index() {
        // Test edge cases
        assert_eq!(random_index(0), 0);
        assert_eq!(random_index(1), 0);

        // Test that random_index produces values in range
        for max in [10, 100, 1000] {
            for _ in 0..100 {
                let idx = random_index(max);
                assert!(idx < max, "Index {} out of range [0, {})", idx, max);
            }
        }
    }

    #[test]
    fn test_parse_seedqr_errors() {
        // Test invalid UTF-8 in standard format (48 bytes)
        let invalid_utf8 = vec![0xFF; 48];
        assert!(matches!(parse_seedqr(&invalid_utf8), Err(ParseSeedQrError::InvalidUtf8(_))));

        // Test invalid number format in standard format
        let invalid_number = b"abcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcd"; // 48 bytes
        assert!(matches!(parse_seedqr(invalid_number), Err(ParseSeedQrError::InvalidWordIndex(_))));

        // Test out of range index
        let out_of_range = b"999999999999999999999999999999999999999999999999"; // 48 bytes
        assert!(matches!(parse_seedqr(out_of_range), Err(ParseSeedQrError::WordIndexOutOfRange(9999))));

        // Test invalid compact format (not valid entropy)
        let invalid_compact = b"invalid"; // 7 bytes - not a valid entropy length
        assert!(matches!(parse_seedqr(invalid_compact), Err(ParseSeedQrError::InvalidMnemonic(_))));
    }
}
