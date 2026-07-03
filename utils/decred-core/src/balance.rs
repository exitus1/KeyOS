// SPDX-License-Identifier: Apache-2.0
//! Companion-reported balance package.
//!
//! The device is air-gapped and cannot see the chain, so the online companion
//! (DCR Pulse) reports per-account balances plus the DCR/USD exchange rate.
//! Same transports as the sign request: a `balance.dcr` file on the microSD
//! card, or a single-part `UR:DCR-BALANCE/...` QR (the UR envelope stays
//! app-side, like the sign-request path). The payload is CBOR in this crate's
//! airgap conventions: minicbor positional arrays, fixed byte fields as arrays
//! of integers.
//!
//! Accounts are identified ONLY by their account fingerprint - the first four
//! bytes of hash160 (ripemd160 over BLAKE-256) of the account xpub's
//! compressed public key, the same value `SignRequest::account_fp` carries.
//! Names and indices never cross the airgap. Entries whose fingerprint matches
//! no local account are counted and surfaced, never guessed at.
//!
//! Purely informational: nothing is spent or verified against these figures,
//! and signing re-derives everything. The wallet total is derived by summing
//! entries - it is deliberately not carried in the package. Decoders must
//! accept arrays longer than declared here and ignore the extra elements, so
//! the format evolves by appending fields (exactly how `account_fp` was added
//! to the sign request).

use minicbor::{Decode, Encode};

use crate::Error;

/// Version counter for the binary balance package. (The retired key=value text
/// format was unversioned; this starts fresh at 1.)
pub const BALANCE_FORMAT_VERSION: u8 = 1;

/// One account's balance, keyed by the account fingerprint.
#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq)]
pub struct AccountBalance {
    #[n(0)]
    pub fp: [u8; 4],
    /// Total balance in atoms (1 DCR = 1e8 atoms).
    #[n(1)]
    pub balance_atoms: u64,
}

#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq)]
pub struct BalanceUpdate {
    #[n(0)]
    pub format_version: u8,
    /// Unix seconds at export time on the companion. Render absolute; only a
    /// device that trusts its clock should render "Nh ago".
    #[n(1)]
    pub asof: u64,
    /// USD per 1 DCR scaled by 1e6 (16.5647 USD -> 16_564_700). 0 = the
    /// companion had no rate; hide fiat.
    #[n(2)]
    pub rate_micro_usd: u64,
    #[n(3)]
    pub accounts: Vec<AccountBalance>,
}

impl BalanceUpdate {
    /// Wallet-wide total: the sum of all entries (matched or not).
    pub fn total_atoms(&self) -> u64 {
        self.accounts
            .iter()
            .fold(0u64, |t, a| t.saturating_add(a.balance_atoms))
    }
}

pub fn encode_balance_update(upd: &BalanceUpdate) -> Result<Vec<u8>, Error> {
    let mut buf = Vec::new();
    minicbor::encode(upd, &mut buf).map_err(|_| Error::Encode)?;
    Ok(buf)
}

pub fn decode_balance_update(bytes: &[u8]) -> Result<BalanceUpdate, Error> {
    let upd: BalanceUpdate = minicbor::decode(bytes).map_err(|_| Error::Parse)?;
    if upd.format_version != BALANCE_FORMAT_VERSION {
        return Err(Error::UnsupportedVersion);
    }
    Ok(upd)
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

    /// Golden fixture shared with the companion (DCR Pulse pins the same bytes
    /// in wallet_device_balance_test.go): asof 1751537520, rate $16.5647,
    /// fp 7f3a9c21 = 1200 DCR, fp b44e0d5a = 34.56 DCR.
    const GOLDEN_HEX: &str = "84011a686657701a00fcc1dc828284187f183a189c18211b0000001bf08eb000828418b4184e0d185a1acdfe6000";

    fn golden_update() -> BalanceUpdate {
        BalanceUpdate {
            format_version: 1,
            asof: 1_751_537_520,
            rate_micro_usd: 16_564_700,
            accounts: vec![
                AccountBalance { fp: [0x7f, 0x3a, 0x9c, 0x21], balance_atoms: 120_000_000_000 },
                AccountBalance { fp: [0xb4, 0x4e, 0x0d, 0x5a], balance_atoms: 3_456_000_000 },
            ],
        }
    }

    #[test]
    fn golden_decodes() {
        let upd = decode_balance_update(&unhex(GOLDEN_HEX)).unwrap();
        assert_eq!(upd, golden_update());
        assert_eq!(upd.total_atoms(), 123_456_000_000);
    }

    #[test]
    fn golden_encodes_bit_identical() {
        // Both ends must produce the same bytes for the same values.
        assert_eq!(encode_balance_update(&golden_update()).unwrap(), unhex(GOLDEN_HEX));
    }

    #[test]
    fn empty_accounts_and_no_rate() {
        let bytes = unhex("84011a686657700080");
        let upd = decode_balance_update(&bytes).unwrap();
        assert_eq!(upd.rate_micro_usd, 0);
        assert!(upd.accounts.is_empty());
        assert_eq!(upd.total_atoms(), 0);
    }

    #[test]
    fn tolerates_appended_fields() {
        // Future versions append elements; today's decoder must ignore them.
        // Same golden body under a 5-element head, with a trailing uint.
        let mut bytes = unhex(GOLDEN_HEX);
        bytes[0] = 0x85;
        bytes.push(0x00);
        assert_eq!(decode_balance_update(&bytes).unwrap(), golden_update());
    }

    #[test]
    fn rejects_unknown_version() {
        let mut bytes = unhex(GOLDEN_HEX);
        bytes[1] = 0x02; // format_version 2
        assert_eq!(decode_balance_update(&bytes), Err(Error::UnsupportedVersion));
    }

    #[test]
    fn max_supply_amount() {
        let bytes = unhex("84011a686657701a00fcc1dc8182840018ff1018e01b000775f05a074000");
        let upd = decode_balance_update(&bytes).unwrap();
        assert_eq!(upd.accounts[0].balance_atoms, 2_100_000_000_000_000);
    }
}
