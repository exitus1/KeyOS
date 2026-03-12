use super::*;

const DEV_PUBKEY1: [u8; 33] = [0x12; 33];

const KNOWN_PUBKEY1: [u8; 33] = [0xAB; 33];
const KNOWN_PUBKEY2: [u8; 33] = [0xCD; 33];
const KNOWN_PUBKEY3: [u8; 33] = [0xEF; 33];

struct Secp256k1Sign([u8; 33]);

impl crate::Secp256k1Sign for Secp256k1Sign {
    fn sign_ecdsa(&self, _msg: [u8; 32]) -> [u8; 64] { [0xAB; 64] }

    fn pubkey(&self) -> [u8; 33] { self.0 }
}

struct Secp256k1Verify;

impl crate::Secp256k1Verify for Secp256k1Verify {
    fn verify_ecdsa(&self, _msg: [u8; 32], _sig: [u8; 64], _pubkey: [u8; 33]) -> VerificationResult {
        VerificationResult::Valid
    }
}

struct Sha256;

impl crate::Sha256 for Sha256 {
    fn hash(&self, _data: &[u8]) -> [u8; 32] { [1; 32] }
}

fn timestamp_for_date(year: i32, month: u32, day: u32) -> u32 {
    chrono::NaiveDate::from_ymd_opt(year, month, day)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc()
        .timestamp()
        .try_into()
        .unwrap()
}

#[test]
fn developer_signing() {
    let binary = [0x01, 0x02, 0x03, 0x04];
    const HEADER_SIZE: usize = 1024;
    let header = Header::sign_new(
        Magic::Atsama5d27KeyOs,
        "1.2.3",
        binary.len().try_into().unwrap(),
        Signer::Developer,
        &binary[..],
        &Sha256,
        &Secp256k1Sign(DEV_PUBKEY1),
        HEADER_SIZE,
    )
    .unwrap();

    assert_eq!(Trust::ThirdParty, header.trust());

    let mut buf = [0u8; HEADER_SIZE + 4];
    header.serialize(&mut buf).unwrap();
    buf[HEADER_SIZE..].copy_from_slice(&binary);
    let parsed = Header::parse(
        &buf[..],
        &[KNOWN_PUBKEY1, KNOWN_PUBKEY2, KNOWN_PUBKEY3],
        &Sha256,
        &Secp256k1Verify,
        HEADER_SIZE,
    )
    .unwrap()
    .unwrap();

    // The parsed header is the same as the serialized header.
    assert_eq!(header.magic(), parsed.magic());
    assert_eq!(header.timestamp(), parsed.timestamp());
    assert_eq!(header.date(), parsed.date());
    assert_eq!(header.version(), parsed.version());
    assert_eq!(header.bin_size(), parsed.bin_size());
    assert_eq!(header.pubkey1(), parsed.pubkey1());
    assert_eq!(header.signature1(), parsed.signature1());
    assert_eq!(header.pubkey2(), parsed.pubkey2());
    assert_eq!(header.signature2(), parsed.signature2());
    assert_eq!(Trust::ThirdParty, parsed.trust());

    // The signing procedure filled in the correct field.
    assert_eq!(header.pubkey1(), [0; 33]);
    assert_eq!(header.signature1(), [0; 64]);
    assert_ne!(header.pubkey2(), [0; 33]);
    assert_ne!(header.signature2(), [0; 64]);

    // The binary representation is correct.
    assert_eq!(buf.len(), HEADER_SIZE + binary.len());
    // Magic number.
    assert_eq!(buf[..4], [0x50, 0x52, 0x4D, 0x31]);
    // Timestamp.
    assert_eq!(buf[4..8], header.timestamp().to_le_bytes(),);
    // Date.
    assert_eq!(&buf[8..20], b"Jan. 1, 1970");
    assert_eq!(&buf[20..22], &[0, 0]);
    // Version.
    assert_eq!(&buf[22..42], b"1.2.3\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0");
    // Binary size.
    assert_eq!(buf[42..46], [4, 0, 0, 0]);
    // Public key 1.
    assert_eq!(&buf[46..79], [0; 33]);
    // Signature 1.
    assert_eq!(&buf[79..143], [0; 64]);
    // Public key 2.
    assert_eq!(&buf[143..176], DEV_PUBKEY1);
    // Signature 2.
    assert_ne!(buf[176..240], [0; 64]);
    // Reserved.
    assert!(&buf[240..HEADER_SIZE].iter().all(|&b| b == 0));
}

#[test]
fn trusted_signing() {
    let binary = [0x01, 0x02, 0x03, 0x04];
    const HEADER_SIZE: usize = 1024;
    let mut header = Header::sign_new(
        Magic::Atsama5d27KeyOs,
        "1.2.3-alpha1",
        binary.len().try_into().unwrap(),
        Signer::Trusted,
        &binary[..],
        &Sha256,
        &Secp256k1Sign(KNOWN_PUBKEY1),
        HEADER_SIZE,
    )
    .unwrap();

    assert_eq!(Trust::Unverified, header.trust());

    let mut buf = [0u8; HEADER_SIZE + 4];
    header.serialize(&mut buf).unwrap();
    buf[HEADER_SIZE..].copy_from_slice(&binary);
    let parsed = Header::parse(
        &buf[..],
        &[KNOWN_PUBKEY1, KNOWN_PUBKEY2, KNOWN_PUBKEY3],
        &Sha256,
        &Secp256k1Verify,
        HEADER_SIZE,
    )
    .unwrap()
    .unwrap();

    // The parsed header is the same as the serialized header.
    assert_eq!(header.magic(), parsed.magic());
    assert_eq!(header.timestamp(), parsed.timestamp());
    assert_eq!(header.date(), parsed.date());
    assert_eq!(header.version(), parsed.version());
    assert_eq!(header.bin_size(), parsed.bin_size());
    assert_eq!(header.pubkey1(), parsed.pubkey1());
    assert_eq!(header.signature1(), parsed.signature1());
    assert_eq!(header.pubkey2(), parsed.pubkey2());
    assert_eq!(header.signature2(), parsed.signature2());
    assert_eq!(Trust::PartiallyTrusted, parsed.trust());

    // The signing procedure filled in the correct field.
    assert_ne!(header.pubkey1(), [0; 33]);
    assert_ne!(header.signature1(), [0; 64]);
    assert_eq!(header.pubkey2(), [0; 33]);
    assert_eq!(header.signature2(), [0; 64]);

    // The binary representation is correct.
    assert_eq!(buf.len(), HEADER_SIZE + binary.len());
    // Magic number.
    assert_eq!(buf[..4], [0x50, 0x52, 0x4D, 0x31]);
    // Timestamp.
    assert_eq!(buf[4..8], header.timestamp().to_le_bytes());
    // Date.
    assert_eq!(&buf[8..20], b"Jan. 1, 1970");
    assert_eq!(&buf[20..22], &[0, 0]);
    // Version.
    assert_eq!(&buf[22..42], b"1.2.3-alpha1\0\0\0\0\0\0\0\0");
    // Binary size.
    assert_eq!(buf[42..46], [4, 0, 0, 0]);
    // Public key 1.
    assert_eq!(&buf[46..79], KNOWN_PUBKEY1);
    // Signature 1.
    assert_ne!(&buf[79..143], [0; 64]);
    // Public key 2.
    assert_eq!(&buf[143..176], [0; 33]);
    // Signature 2.
    assert_eq!(buf[176..240], [0; 64]);
    // Reserved.
    assert!(&buf[240..HEADER_SIZE].iter().all(|&b| b == 0));

    // Add second signature.
    header.add_second_signature(&Secp256k1Sign(KNOWN_PUBKEY3)).unwrap();

    let mut buf = [0u8; HEADER_SIZE + 4];
    header.serialize(&mut buf).unwrap();
    buf[HEADER_SIZE..].copy_from_slice(&binary);
    let parsed = Header::parse(
        &buf[..],
        &[KNOWN_PUBKEY1, KNOWN_PUBKEY2, KNOWN_PUBKEY3],
        &Sha256,
        &Secp256k1Verify,
        HEADER_SIZE,
    )
    .unwrap()
    .unwrap();

    // The parsed header is the same as the serialized header.
    assert_eq!(header.magic(), parsed.magic());
    assert_eq!(header.timestamp(), parsed.timestamp());
    assert_eq!(header.date(), parsed.date());
    assert_eq!(header.version(), parsed.version());
    assert_eq!(header.bin_size(), parsed.bin_size());
    assert_eq!(header.pubkey1(), parsed.pubkey1());
    assert_eq!(header.signature1(), parsed.signature1());
    assert_eq!(header.pubkey2(), parsed.pubkey2());
    assert_eq!(header.signature2(), parsed.signature2());
    assert_eq!(Trust::FullyTrusted, parsed.trust());

    // The signing procedure filled in the correct fields.
    assert_ne!(header.pubkey1(), [0; 33]);
    assert_ne!(header.signature1(), [0; 64]);
    assert_ne!(header.pubkey2(), [0; 33]);
    assert_ne!(header.signature2(), [0; 64]);

    // The binary representation is correct.
    assert_eq!(buf.len(), HEADER_SIZE + binary.len());
    // Magic number.
    assert_eq!(buf[..4], [0x50, 0x52, 0x4D, 0x31]);
    // Timestamp.
    assert_eq!(buf[4..8], header.timestamp().to_le_bytes());
    // Date.
    assert_eq!(&buf[8..20], b"Jan. 1, 1970");
    assert_eq!(&buf[20..22], &[0, 0]);
    // Version.
    assert_eq!(&buf[22..42], b"1.2.3-alpha1\0\0\0\0\0\0\0\0");
    // Binary size.
    assert_eq!(buf[42..46], [4, 0, 0, 0]);
    // Public key 1.
    assert_eq!(&buf[46..79], KNOWN_PUBKEY1);
    // Signature 1.
    assert_ne!(&buf[79..143], [0; 64]);
    // Public key 2.
    assert_eq!(&buf[143..176], KNOWN_PUBKEY3);
    // Signature 2.
    assert_ne!(buf[176..240], [0; 64]);
    // Reserved.
    assert!(&buf[240..HEADER_SIZE].iter().all(|&b| b == 0));
}

#[test]
fn date_format_examples() {
    const HEADER_SIZE: usize = 1024;
    let binary = [0u8; 1];

    let header = Header::sign_new(
        Magic::Atsama5d27KeyOs,
        "1.2.3",
        timestamp_for_date(2026, 1, 13),
        Signer::Trusted,
        &binary[..],
        &Sha256,
        &Secp256k1Sign(KNOWN_PUBKEY1),
        HEADER_SIZE,
    )
    .unwrap();
    assert_eq!(header.date(), "Jan. 13, 2026");

    let header = Header::sign_new(
        Magic::Atsama5d27KeyOs,
        "1.2.3",
        timestamp_for_date(2028, 8, 7),
        Signer::Trusted,
        &binary[..],
        &Sha256,
        &Secp256k1Sign(KNOWN_PUBKEY1),
        HEADER_SIZE,
    )
    .unwrap();
    assert_eq!(header.date(), "Aug. 7, 2028");

    let header = Header::sign_new(
        Magic::Atsama5d27KeyOs,
        "1.2.3",
        timestamp_for_date(2025, 5, 12),
        Signer::Trusted,
        &binary[..],
        &Sha256,
        &Secp256k1Sign(KNOWN_PUBKEY1),
        HEADER_SIZE,
    )
    .unwrap();
    assert_eq!(header.date(), "May 12, 2025");

    let header = Header::sign_new(
        Magic::Atsama5d27KeyOs,
        "1.2.3",
        timestamp_for_date(2027, 6, 9),
        Signer::Trusted,
        &binary[..],
        &Sha256,
        &Secp256k1Sign(KNOWN_PUBKEY1),
        HEADER_SIZE,
    )
    .unwrap();
    assert_eq!(header.date(), "June 9, 2027");

    let header = Header::sign_new(
        Magic::Atsama5d27KeyOs,
        "1.2.3",
        timestamp_for_date(2024, 7, 21),
        Signer::Trusted,
        &binary[..],
        &Sha256,
        &Secp256k1Sign(KNOWN_PUBKEY1),
        HEADER_SIZE,
    )
    .unwrap();
    assert_eq!(header.date(), "July 21, 2024");
}
