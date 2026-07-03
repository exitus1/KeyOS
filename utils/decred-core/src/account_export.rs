// SPDX-License-Identifier: Apache-2.0
//! Account dpub export package.
//!
//! The companion (DCR Pulse) has no camera, so scanning the dpub-export QR is
//! not always possible. This package carries account xpubs the other way over
//! the microSD card: the device writes `accounts.dcr` to the card root, the
//! companion's watch-only import reads it. CBOR in this crate's airgap
//! conventions (minicbor positional arrays; strings as CBOR text strings).
//!
//! Per entry: the BIP44 account index (authoritative from the device - the
//! companion derives offline-signing requests against it, so a wrong index
//! means a clean on-device refusal later), the dpub as its base58check string
//! (self-checksummed; both ends already parse it, hence no fingerprint field -
//! the companion derives the fingerprint itself for display), and the local
//! account name as a naming suggestion. No network field: the dpub version
//! bytes encode the network. Decoders must accept arrays longer than declared
//! and ignore the extra elements (append-only evolution, like the sign
//! request's account_fp).
//!
//! Only PUBLIC key material leaves the device in this package.

use minicbor::{Decode, Encode};

use crate::Error;

pub const ACCOUNT_EXPORT_FORMAT_VERSION: u8 = 1;

/// One exported account: index, dpub string, and the local name suggestion.
#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq)]
pub struct ExportedAccount {
    /// BIP44 account index (m/44'/42'/account').
    #[n(0)]
    pub account: u32,
    /// Neutered account extended public key, base58check ("dpub...").
    #[n(1)]
    pub dpub: String,
    /// The device-local account name; a suggestion only, may be empty.
    #[n(2)]
    pub name: String,
}

#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq)]
pub struct AccountExport {
    #[n(0)]
    pub format_version: u8,
    #[n(1)]
    pub accounts: Vec<ExportedAccount>,
}

pub fn encode_account_export(exp: &AccountExport) -> Result<Vec<u8>, Error> {
    let mut buf = Vec::new();
    minicbor::encode(exp, &mut buf).map_err(|_| Error::Encode)?;
    Ok(buf)
}

pub fn decode_account_export(bytes: &[u8]) -> Result<AccountExport, Error> {
    let exp: AccountExport = minicbor::decode(bytes).map_err(|_| Error::Parse)?;
    if exp.format_version != ACCOUNT_EXPORT_FORMAT_VERSION {
        return Err(Error::UnsupportedVersion);
    }
    Ok(exp)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unhex(s: &str) -> Vec<u8> {
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
            .collect()
    }

    const DPUB0: &str = "dpubZG4J1GEeSSFc4VVqYDGtPGEubfkmLGbfrpqQCv8HLYdNNMR8tJBmaethbn3myAF5NhLLpTmK2RECPNRtqZTkmkx6NA2tvPrgU2wGLyynuSX";
    const DPUB1: &str = "dpubZG4J1GEeSSFc5hRkUWGVZwHJCFwdiWy67sPNXL9MUpt2XvsWPHB1xHfyyRxqKiNy7X1TLJHfPsRNoT5L9avFv39xaUGKZmKorL7j1L7EnZq";

    /// Golden fixture shared with the companion (DCR Pulse pins the same bytes):
    /// two accounts, index 0 "Main" and index 1 "acc2".
    const GOLDEN_TWO_HEX: &str = "8201828300786f647075625a47344a314745655353466334565671594447745047457562666b6d4c47626672707151437638484c59644e4e4d5238744a426d61657468626e336d794146354e684c4c70546d4b32524543504e5274715a546b6d6b78364e41327476507267553277474c79796e755358644d61696e8301786f647075625a47344a31474565535346633568526b555747565a77484a43467764695779363773504e584c394d55707432587673575048423178486679795278714b694e79375831544c4a48665073524e6f54354c39617646763339786155474b5a6d4b6f724c376a314c37456e5a716461636332";

    fn golden_two() -> AccountExport {
        AccountExport {
            format_version: 1,
            accounts: vec![
                ExportedAccount { account: 0, dpub: DPUB0.into(), name: "Main".into() },
                ExportedAccount { account: 1, dpub: DPUB1.into(), name: "acc2".into() },
            ],
        }
    }

    #[test]
    fn golden_decodes() {
        assert_eq!(decode_account_export(&unhex(GOLDEN_TWO_HEX)).unwrap(), golden_two());
    }

    #[test]
    fn golden_encodes_bit_identical() {
        assert_eq!(encode_account_export(&golden_two()).unwrap(), unhex(GOLDEN_TWO_HEX));
    }

    #[test]
    fn single_and_empty_round_trip() {
        for exp in [
            AccountExport {
                format_version: 1,
                accounts: vec![ExportedAccount { account: 7, dpub: DPUB1.into(), name: String::new() }],
            },
            AccountExport { format_version: 1, accounts: vec![] },
        ] {
            let bytes = encode_account_export(&exp).unwrap();
            assert_eq!(decode_account_export(&bytes).unwrap(), exp);
        }
    }

    #[test]
    fn tolerates_appended_fields() {
        // Future versions append elements; today's decoder must ignore them.
        let mut bytes = unhex(GOLDEN_TWO_HEX);
        bytes[0] = 0x83; // array 2 -> 3 at the top level
        bytes.push(0x00);
        assert_eq!(decode_account_export(&bytes).unwrap(), golden_two());
    }

    #[test]
    fn rejects_unknown_version() {
        let mut bytes = unhex(GOLDEN_TWO_HEX);
        bytes[1] = 0x02;
        assert_eq!(decode_account_export(&bytes), Err(Error::UnsupportedVersion));
    }
}
