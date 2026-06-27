// SPDX-License-Identifier: Apache-2.0
//! Air-gapped interchange format between Cake Wallet (online, builds the tx)
//! and the Passport (offline, signs).
//!
//! Decred has no PSBT, so we define a minimal CBOR package. Cake Wallet, as a
//! watch-only wallet, knows every input's prevout script, amount, and the
//! derivation path of the key that owns it — everything the signer needs. The
//! device independently recomputes addresses/amounts for on-screen review, so a
//! malicious or buggy companion cannot redirect funds without the user seeing it.
//!
//! Transport:
//!   * QR  → wrap `encode_sign_request` bytes in UR type `dcr-sign-request`;
//!           return `dcr-signed-tx` (the broadcast-ready full tx).
//!   * SD  → write the same bytes as `unsigned.dcrtx` / `signed.dcrtx`.
//!
//! This is format version 1; bump `FORMAT_VERSION` on any breaking change.

use minicbor::{Decode, Encode};
use secp256k1::Secp256k1;

use crate::address::p2pkh_script;
use crate::hashing::hash160;
use crate::hd::ExtPrivKey;
use crate::sign::sign_p2pkh_input;
use crate::tx::{MsgTx, OutPoint, TxIn, TxOut, NULL_BLOCK_HEIGHT, NULL_BLOCK_INDEX};
use crate::Error;

pub const FORMAT_VERSION: u8 = 1;

/// One input to be signed, with the metadata only an online wallet has.
#[derive(Clone, Debug, Encode, Decode)]
pub struct InputMeta {
    #[n(0)]
    pub prev_hash: [u8; 32],
    #[n(1)]
    pub prev_index: u32,
    #[n(2)]
    pub tree: u8,
    #[n(3)]
    pub sequence: u32,
    #[n(4)]
    pub value_in: i64,
    /// Account-relative path suffix `[branch, index]`; the device prepends
    /// `m/44'/42'/account'`.
    #[n(5)]
    pub branch: u32,
    #[n(6)]
    pub index: u32,
    /// Prevout pkScript. For our keys the device re-derives and verifies this
    /// equals `p2pkh(hash160(pubkey))` before trusting it.
    #[n(7)]
    pub prev_script: Vec<u8>,
}

/// One output, for both the wire tx and on-device display.
#[derive(Clone, Debug, Encode, Decode)]
pub struct OutputMeta {
    #[n(0)]
    pub value: i64,
    #[n(1)]
    pub version: u16,
    #[n(2)]
    pub pk_script: Vec<u8>,
    /// True if this output is change back to our own wallet.
    #[n(3)]
    pub is_change: bool,
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct SignRequest {
    #[n(0)]
    pub format_version: u8,
    #[n(1)]
    pub tx_version: u16,
    #[n(2)]
    pub account: u32,
    #[n(3)]
    pub lock_time: u32,
    #[n(4)]
    pub expiry: u32,
    #[n(5)]
    pub inputs: Vec<InputMeta>,
    #[n(6)]
    pub outputs: Vec<OutputMeta>,
}

pub fn encode_sign_request(req: &SignRequest) -> Result<Vec<u8>, Error> {
    let mut buf = Vec::new();
    minicbor::encode(req, &mut buf).map_err(|_| Error::Encode)?;
    Ok(buf)
}

pub fn decode_sign_request(bytes: &[u8]) -> Result<SignRequest, Error> {
    let req: SignRequest = minicbor::decode(bytes).map_err(|_| Error::Parse)?;
    if req.format_version != FORMAT_VERSION {
        return Err(Error::UnsupportedVersion);
    }
    Ok(req)
}

/// A human-reviewable summary the UI shows before the user approves signing.
pub struct ReviewSummary {
    /// (address, amount) for every non-change output.
    pub recipients: Vec<(String, i64)>,
    pub change_total: i64,
    pub input_total: i64,
    pub fee: i64,
}

impl SignRequest {
    pub fn input_total(&self) -> i64 {
        self.inputs.iter().map(|i| i.value_in).sum()
    }
    pub fn output_total(&self) -> i64 {
        self.outputs.iter().map(|o| o.value).sum()
    }

    /// Build the display summary (recipients, change, fee).
    pub fn review(&self) -> ReviewSummary {
        let mut recipients = Vec::new();
        let mut change_total = 0i64;
        for o in &self.outputs {
            if o.is_change {
                change_total += o.value;
            } else {
                let addr = script_to_address(&o.pk_script)
                    .unwrap_or_else(|| "<non-standard script>".to_string());
                recipients.push((addr, o.value));
            }
        }
        let input_total = self.input_total();
        ReviewSummary {
            recipients,
            change_total,
            input_total,
            fee: input_total - self.output_total(),
        }
    }
}

/// Best-effort: decode a mainnet P2PKH script back to an address for display.
fn script_to_address(script: &[u8]) -> Option<String> {
    if script.len() == 25
        && script[0] == 0x76
        && script[1] == 0xa9
        && script[2] == 0x14
        && script[23] == 0x88
        && script[24] == 0xac
    {
        let mut h = [0u8; 20];
        h.copy_from_slice(&script[3..23]);
        Some(crate::hashing::check_encode(&h, crate::address::PKH_ADDR_ID_MAINNET))
    } else {
        None
    }
}

/// End-to-end: turn a decoded `SignRequest` into a broadcast-ready Decred tx,
/// given the BIP39 entropy (from the secure element) and the optional passphrase.
///
/// For every input the device **re-derives** the owning key, recomputes its
/// P2PKH script, and refuses to sign if it does not match `prev_script` — so
/// the companion cannot trick the device into signing with the wrong key.
///
/// Takes an already-derived BIP32 `master` (from the secure-element seam in
/// the app's keys.rs) rather than raw entropy, so the app keeps ONE place that
/// touches the seed — exactly like the Bitcoin app threads a single MasterKey.
pub fn sign_request(
    secp: &Secp256k1<secp256k1::All>,
    master: &ExtPrivKey,
    req: &SignRequest,
) -> Result<Vec<u8>, Error> {
    let account = master.account_key(secp, req.account)?;

    // Assemble the unsigned tx (sigScripts empty for sighash computation).
    let mut tx = MsgTx {
        version: req.tx_version,
        tx_in: req
            .inputs
            .iter()
            .map(|i| TxIn {
                previous_outpoint: OutPoint {
                    hash: i.prev_hash,
                    index: i.prev_index,
                    tree: i.tree,
                },
                sequence: i.sequence,
                value_in: i.value_in,
                block_height: NULL_BLOCK_HEIGHT,
                block_index: NULL_BLOCK_INDEX,
                signature_script: Vec::new(),
            })
            .collect(),
        tx_out: req
            .outputs
            .iter()
            .map(|o| TxOut {
                value: o.value,
                version: o.version,
                pk_script: o.pk_script.clone(),
            })
            .collect(),
        lock_time: req.lock_time,
        expiry: req.expiry,
    };

    // Sign each input.
    for (idx, meta) in req.inputs.iter().enumerate() {
        let key = account.address_key(secp, meta.branch, meta.index)?;
        let pubkey = key.compressed_pubkey(secp);
        let expected_script = p2pkh_script(&hash160(&pubkey));
        if meta.prev_script != expected_script {
            return Err(Error::ScriptMismatch);
        }
        let sig_script =
            sign_p2pkh_input(secp, &tx, idx, &expected_script, &key.secret, &pubkey)?;
        tx.tx_in[idx].signature_script = sig_script;
    }

    Ok(tx.serialize_full())
}
