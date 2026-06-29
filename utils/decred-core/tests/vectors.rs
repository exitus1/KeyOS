// SPDX-License-Identifier: Apache-2.0
//
//! Reference vectors lifted verbatim from dcrd source. These are the oracle:
//! if decred-core disagrees with any of them, decred-core is wrong, because
//! these exact strings/bytes are what the live network produced and validated.
//!
//! Sources (paths are within the dcrd repo):
//!   - hdkeychain/extendedkey_test.go  (BIP32 master + child dprv strings)
//!   - txscript/standard_test.go / stdaddr  (P2PKH address + payScript)
//!   - crypto/blake256                 (BLAKE-256 KATs)
//!
//! `cargo test -p decred-core` runs these on the host before anything touches
//! a device.

use decred_core::address::{decode_p2pkh, p2pkh_script};
use decred_core::blake256;
use decred_core::hashing::check_encode;
use decred_core::hd::{ExtPrivKey, HARDENED};
use secp256k1::Secp256k1;

// ---------------------------------------------------------------------------
// BLAKE-256 — Decred's universal hash. NOT BLAKE2/BLAKE3.
// ---------------------------------------------------------------------------

#[test]
fn blake256_empty_kat() {
    // dcrd: blake256.Sum256("")
    let got = blake256::sum256(b"");
    assert_eq!(
        hex::encode(got),
        "716f6e863f744b9ac22c97ec7b76ea5f5908bc5b2f67c61510bfc4751384ea7a"
    );
}

#[test]
fn blake256_single_zero_kat() {
    // dcrd: blake256.Sum256(0x00)
    let got = blake256::sum256(&[0x00]);
    assert_eq!(
        hex::encode(got),
        "0ce8d4ef4dd7cd8d62dfded9d4edb0a774ae6a41929a74da23109e8f11139c87"
    );
}

// ---------------------------------------------------------------------------
// BIP32 over Decred version bytes (dprv). Master + a hardened/normal chain.
// HMAC master key is "Bitcoin seed" for every coin; Decred differs only in the
// dprv/dpub version prefixes and the double-BLAKE256 base58 checksum.
// ---------------------------------------------------------------------------

const BIP32_VEC1_SEED: &str = "000102030405060708090a0b0c0d0e0f";

#[test]
fn bip32_master_dprv() {
    let seed = hex::decode(BIP32_VEC1_SEED).unwrap();
    let master = ExtPrivKey::master_from_seed(&seed).unwrap();
    // dcrd extendedkey_test.go: "test vector 1 chain m"
    assert_eq!(
        master.to_dprv(),
        "dprv3hCznBesA6jBtmoyVFPfyMSZ1qYZ3WdjdebquvkEfmRfxC9VFEFi2YDaJqHnx7uGe75eGSa3Mn3oHK11hBW7KZUrPxwbCPBmuCi1nwm182s"
    );
}

#[test]
fn bip32_hardened_child_m_0h() {
    let secp = Secp256k1::new();
    let seed = hex::decode(BIP32_VEC1_SEED).unwrap();
    let master = ExtPrivKey::master_from_seed(&seed).unwrap();
    let child = master.derive_child(&secp, HARDENED).unwrap(); // m/0'
    // dcrd: "test vector 1 chain m/0H"
    assert_eq!(
        child.to_dprv(),
        "dprv3kUQDBztdyjKuwnaL3hfKYpT7W6X2huYH5d61YSWFBebSYwEBHAXJkCpQ7rvMAxPzKqxVCGLvBqWvGxXjAyMJsV1XwKkfnQCM9KctC8k8bk"
    );
}

#[test]
fn bip32_mixed_path_m_0h_1() {
    let secp = Secp256k1::new();
    let seed = hex::decode(BIP32_VEC1_SEED).unwrap();
    let master = ExtPrivKey::master_from_seed(&seed).unwrap();
    let child = master.derive_path(&secp, &[HARDENED, 1]).unwrap(); // m/0'/1
    // dcrd: "test vector 1 chain m/0H/1" — exercises hardened then normal.
    assert_eq!(
        child.to_dprv(),
        "dprv3nRtCZ5VAoHW4RUwQgRafSNRPUDFrmsgyY71A5eoZceVfuyL9SbZe2rcbwDW2UwpkEniE4urffgbypegscNchPajWzy9QS4cRxF8QYXsZtq"
    );
}

#[test]
fn bip32_master_dpub() {
    let secp = Secp256k1::new();
    let seed = hex::decode(BIP32_VEC1_SEED).unwrap();
    let master = ExtPrivKey::master_from_seed(&seed).unwrap();
    // dcrd extendedkey_test.go: "test vector 1 chain m" wantPub. Locks down the
    // neutered (watch-only) export path.
    assert_eq!(
        master.to_dpub(&secp),
        "dpubZ9169KDAEUnyoBhjjmT2VaEodr6pUTDoqCEAeqgbfr2JfkB88BbK77jbTYbcYXb2FVz7DKBdW4P618yd51MwF8DjKVopSbS7Lkgi6bowX5w"
    );
}

// --------------------------------------------------------------------------- (base58check w/ double-BLAKE256 checksum, "Ds" prefix)
// and the canonical payScript: DUP HASH160 <20> EQUALVERIFY CHECKSIG.
// ---------------------------------------------------------------------------

#[test]
fn p2pkh_address_from_hash160() {
    let h160 = hex::decode("2789d58cfa0957d206f025c2af056fc8a77cebb0").unwrap();
    let mut arr = [0u8; 20];
    arr.copy_from_slice(&h160);
    let addr = check_encode(&arr, decred_core::address::PKH_ADDR_ID_MAINNET);
    assert_eq!(addr, "DsUZxxoHJSty8DCfwfartwTYbuhmVct7tJu");
}

#[test]
fn p2pkh_address_second_vector() {
    let h160 = hex::decode("229ebac30efd6a69eec9c1a48e048b7c975c25f2").unwrap();
    let mut arr = [0u8; 20];
    arr.copy_from_slice(&h160);
    let addr = check_encode(&arr, decred_core::address::PKH_ADDR_ID_MAINNET);
    assert_eq!(addr, "DsU7xcg53nxaKLLcAUSKyRndjG78Z2VZnX9");
}

#[test]
fn p2pkh_address_roundtrip_decode() {
    let h160 = hex::decode("2789d58cfa0957d206f025c2af056fc8a77cebb0").unwrap();
    let decoded = decode_p2pkh("DsUZxxoHJSty8DCfwfartwTYbuhmVct7tJu").unwrap();
    assert_eq!(&decoded[..], &h160[..]);
}

#[test]
fn p2pkh_payscript_layout() {
    let h160 = hex::decode("2789d58cfa0957d206f025c2af056fc8a77cebb0").unwrap();
    let mut arr = [0u8; 20];
    arr.copy_from_slice(&h160);
    let script = p2pkh_script(&arr);
    // dcrd: 76a914<20-byte-hash>88ac
    assert_eq!(
        hex::encode(script),
        "76a9142789d58cfa0957d206f025c2af056fc8a77cebb088ac"
    );
}
