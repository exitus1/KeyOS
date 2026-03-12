// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later
use std::time::Duration;

use backup_shard::Shard;
use security::{DeviceId, Seed};
use server::{ArchiveHandler, BlockingScalarHandler, Server, ServerContext};
use xous::DropDeallocate;

use crate::{
    error::{KeycardError, KeycardIdentifyError},
    messages::{
        CheckBackup, DetectKeycard, FormatKeycard, GenerateShards, IdentifyKeycard, KeycardId,
        LoadShardFromKeycard, LoadedShard, MasterSeedRestored, PopShard, PushShard, ResetShards,
        RestoreMasterSeed, StoreShardToKeycard,
    },
};

crypto::use_api!();
nfc::use_api!();
security::use_api!();

#[derive(server::Server)]
#[name = "os/keycard"]
pub struct KeycardServer {
    security: Security,
    crypto: CryptoApi,
    nfc: NfcApi,
    current_device_id: DeviceId,
    expected_seed_fingerprint: [u8; 32],
    shards: Vec<Shard>,
}

const KEYCARD_NUM_SHARES: usize = 3;
const KEYCARD_THRESHOLD: usize = 2;
const NFC_READ_TIMEOUT: Duration = Duration::from_millis(1000);
const NFC_WRITE_TIMEOUT: Duration = Duration::from_millis(3000);

impl Server for KeycardServer {}

impl KeycardServer {
    pub(crate) fn new() -> Result<Self, KeycardError> {
        let security = security::Security::default();
        let crypto = CryptoApi::default();
        let nfc = NfcApi::default();
        let current_device_id = loop {
            if let Ok(device_id) = security.device_id() {
                break device_id;
            } else {
                // retry later, the BT chip is not ready
                std::thread::sleep(Duration::from_millis(100));
            }
        };

        Ok(Self {
            security,
            crypto,
            nfc,
            shards: Vec::new(),
            current_device_id,
            expected_seed_fingerprint: [0; 32],
        })
    }

    fn reset(&mut self) {
        self.shards.clear();
        log::debug!("Shards in pool: {:02x?}", self.shards);
    }

    fn generate_shards(&mut self, with_magic_backup: bool) -> Result<(), KeycardError> {
        self.reset();
        let Some(seed) = self.security.seed()? else {
            return Err(KeycardError::SeedMissing);
        };
        let seed_fingerprint = self.security.seed_fingerprint()?;
        let seed_shares = self.crypto.split_secret(seed.to_vec(), KEYCARD_NUM_SHARES, KEYCARD_THRESHOLD)?;
        for (seed_shamir_share_index, seed_shamir_share) in seed_shares.into_iter().enumerate() {
            self.shards.push(Shard::new(
                self.current_device_id.0,
                seed_fingerprint,
                seed_shamir_share,
                seed_shamir_share_index,
                with_magic_backup,
            ));
        }
        log::debug!("Generated shards {:02x?}", self.shards);
        Ok(())
    }

    fn pop_shard(&mut self) -> Result<Shard, KeycardError> {
        let shard = self.shards.pop().ok_or(KeycardError::NoShardLeft)?;
        log::debug!("Poped shard: {:02x?}", shard);
        log::debug!("Shards in pool: {:02x?}", self.shards);
        if shard.part_of_magic_backup() {
            Ok(shard)
        } else {
            self.shards.push(shard);
            log::debug!("Shards in pool: {:02x?}", self.shards);
            Err(KeycardError::NotMagicBackupShard)
        }
    }

    fn push_shard(&mut self, shard: Shard, accept_different_device_id: bool) -> Result<(), KeycardError> {
        if !shard.part_of_magic_backup() {
            Err(KeycardError::NotMagicBackupShard)
        } else if shard.seed_fingerprint() != &self.expected_seed_fingerprint {
            Err(KeycardError::DifferentSeedFingerprint)
        } else if !accept_different_device_id && shard.device_id() != &self.current_device_id.0 {
            Err(KeycardError::DifferentDeviceId)
        } else {
            log::debug!("Pushed shard: {:02x?}", shard);
            self.shards.push(shard);
            log::debug!("Shards in pool: {:02x?}", self.shards);
            Ok(())
        }
    }

    fn identify_keycard(&mut self) -> Result<(Vec<u8>, Option<KeycardIdentifyError>), KeycardError> {
        let (uid, raw_msg) = self.nfc.read_ndef_raw_msg(NFC_READ_TIMEOUT)?;
        log::debug!("Read raw message: {:02x?}", raw_msg);
        let Ok(ndef_msg) = ndef::Message::try_from(raw_msg.as_slice()) else {
            return Ok((uid, Some(KeycardIdentifyError::InvalidData)));
        };
        log::debug!("Read NDEF message: {:02x?}", ndef_msg);
        if ndef_msg.records.len() != 1 {
            return Ok((uid, Some(KeycardIdentifyError::InvalidData)));
        }
        if !ndef_msg.records[0].is_type_cbor() {
            return Ok((uid, Some(KeycardIdentifyError::InvalidData)));
        }
        let payload = ndef_msg.records[0].payload();
        let Ok(shard) = Shard::decode(&payload) else {
            return Ok((uid, Some(KeycardIdentifyError::InvalidData)));
        };
        log::debug!("Read shard: {:02x?}", shard);
        if &hmac(&self.security, &shard, &uid)? != shard.hmac() {
            return Ok((uid, Some(KeycardIdentifyError::HmacMismatch)));
        }
        if shard.seed_shamir_share().is_empty() {
            return Ok((uid, None));
        }
        if shard.device_id() != &self.current_device_id.0 {
            return Ok((uid, Some(KeycardIdentifyError::DifferentDeviceId)));
        }
        if shard.seed_fingerprint() != &self.security.seed_fingerprint()? {
            return Ok((uid, Some(KeycardIdentifyError::DifferentSeedFingerprint)));
        }
        Ok((uid, Some(KeycardIdentifyError::ExistingShard)))
    }

    fn store_shard_to_keycard(&mut self, uid: Vec<u8>) -> Result<(), KeycardError> {
        let mut shard = self.shards.pop().ok_or(KeycardError::NoShardLeft)?;
        log::debug!("Poped shard: {:02x?}", shard);
        log::debug!("Shards in pool: {:02x?}", self.shards);
        let original_shard = shard.clone();
        shard.set_hmac(hmac(&self.security, &shard, &uid)?);
        let mut ndef_msg = ndef::Message::default();
        let mut ndef_rec1 = ndef::Record::new(None, ndef::Payload::from_cbor_encodable(&shard));
        ndef_msg.append_record(&mut ndef_rec1);
        log::debug!("Store NDEF message: {:02x?}", ndef_msg);
        match self.nfc.write_ndef_raw_msg(uid, ndef_msg.to_vec(), NFC_WRITE_TIMEOUT) {
            Ok(_) => Ok(()),
            Err(e) => {
                // push shard back on the stack in case of error
                // so we can retry storing the shard to the keycard
                self.shards.push(original_shard);
                Err(e.into())
            }
        }
    }

    fn format_keycard(&mut self, uid: Vec<u8>) -> Result<(), KeycardError> {
        // Write an "empty" shard to the keycard to format it.
        // For a formatted card, only the HMAC must be valid; the rest of the fields can be zeroed and
        // the seed_shamir_share must be empty.
        let mut shard = Shard::default();
        shard.set_hmac(hmac(&self.security, &shard, &uid)?);

        let mut ndef_msg = ndef::Message::default();
        let mut ndef_rec = ndef::Record::new(None, ndef::Payload::from_cbor_encodable(&shard));
        ndef_msg.append_record(&mut ndef_rec);
        log::debug!("Format NDEF message: {:02x?}", ndef_msg);

        match self.nfc.write_ndef_raw_msg(uid, ndef_msg.to_vec(), NFC_WRITE_TIMEOUT) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    fn load_shard_from_keycard(&mut self) -> Result<LoadedShard, KeycardError> {
        let (uid, raw_msg) = self.nfc.read_ndef_raw_msg(NFC_READ_TIMEOUT)?;
        log::debug!("Load raw message: {:02x?}", raw_msg);
        if raw_msg.is_empty() {
            return Err(KeycardError::BlankTag);
        }
        let ndef_msg = ndef::Message::try_from(raw_msg.as_slice())?;
        log::debug!("Load NDEF message: {:02x?}", ndef_msg);
        if ndef_msg.records.len() != 1 {
            return Err(KeycardError::InvalidData);
        }
        if !ndef_msg.records[0].is_type_cbor() {
            return Err(KeycardError::InvalidData);
        }
        let payload = ndef_msg.records[0].payload();
        let shard = Shard::decode(&payload).map_err(|_| KeycardError::InvalidData)?;
        log::debug!("Load shard: {:02x?}", shard);
        if &hmac(&self.security, &shard, &uid)? != shard.hmac() {
            return Err(KeycardError::HmacMismatch);
        }
        // Ignore formatted Keycard with blank Shard
        if shard.seed_shamir_share().is_empty() {
            return Err(KeycardError::BlankShard);
        }
        if self.shards.is_empty() {
            self.expected_seed_fingerprint = *shard.seed_fingerprint();
        } else if &self.expected_seed_fingerprint != shard.seed_fingerprint() {
            return Err(KeycardError::DifferentSeedFingerprint);
        }
        let part_of_magic_backup = shard.part_of_magic_backup();
        // make sure to not add the same shard twice
        if !self.shards.iter().any(|s| s.seed_shamir_share_index() == shard.seed_shamir_share_index()) {
            self.shards.push(shard);
        }
        log::debug!("Shards in pool: {:02x?}", self.shards);
        Ok(LoadedShard {
            id: KeycardId(uid),
            has_magic_backup: part_of_magic_backup,
            seed_fingerprint: self.expected_seed_fingerprint,
        })
    }

    fn reconstruct_seed(&self) -> Result<MasterSeedRestored, KeycardError> {
        let mut indexes = Vec::new();
        let mut shares = Vec::new();
        let mut different_device_id = false;

        for s in &self.shards {
            if s.device_id() != &self.current_device_id.0 {
                different_device_id = true;
            }

            indexes.push(s.seed_shamir_share_index());
            shares.push(s.seed_shamir_share().to_vec());
        }

        let recovered = self.crypto.recover_secret(indexes, shares)?;
        let seed = Seed::from_bytes(&recovered);
        log::debug!("Restored master seed: {:02x?}", seed.bytes());

        let seed_fingerprint = self.security.fingerprint(&seed)?;
        log::debug!("Restored master seed fingerprint: {:02x?}", seed_fingerprint);
        log::debug!("Expected master seed fingerprint: {:02x?}", self.expected_seed_fingerprint);
        if seed_fingerprint != self.expected_seed_fingerprint {
            return Err(KeycardError::DifferentSeedFingerprint);
        }
        Ok(MasterSeedRestored { seed, different_device_id })
    }

    fn check_backup(&mut self) -> Result<(), KeycardError> {
        if self.shards.len() < KEYCARD_NUM_SHARES {
            return Err(KeycardError::NotEnoughShards);
        }

        let _master_seed_restored = self.reconstruct_seed()?;

        self.reset();
        Ok(())
    }

    fn restore_master_seed(&mut self) -> Result<MasterSeedRestored, KeycardError> {
        if self.shards.len() < KEYCARD_THRESHOLD {
            return Err(KeycardError::NotEnoughShards);
        }

        let master_seed_restored = self.reconstruct_seed()?;

        self.reset();
        Ok(master_seed_restored)
    }
}

fn hmac(security: &Security, shard: &Shard, uid: &[u8]) -> Result<[u8; 32], KeycardError> {
    let input = shard.hmac_input(uid);
    let mut page = DropDeallocate::new(
        xous::map_memory(None, None, 4096, xous::MemoryFlags::W | xous::MemoryFlags::NO_CACHE)
            .expect("mapmemory"),
    );
    page.as_slice_mut()[..input.len()].copy_from_slice(&input);
    let input_hash = CryptoApi::default().sha256(*page, 0, input.len())?;
    Ok(security.keycard_authenticity_mac(input_hash)?)
}

impl BlockingScalarHandler<ResetShards> for KeycardServer {
    fn handle(
        &mut self,
        _msg: ResetShards,
        _sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> <ResetShards as server::BlockingScalar>::Response {
        self.reset();
        Ok(())
    }
}

impl BlockingScalarHandler<GenerateShards> for KeycardServer {
    fn handle(
        &mut self,
        GenerateShards { with_magic_backup }: GenerateShards,
        _sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> <GenerateShards as server::BlockingScalar>::Response {
        self.generate_shards(with_magic_backup)
    }
}

impl ArchiveHandler<PopShard> for KeycardServer {
    fn handle(
        &mut self,
        _msg: PopShard,
        _sender: xous::PID,
        _context: &mut ServerContext<Self>,
    ) -> <PopShard as server::Archive>::Response {
        self.pop_shard()
    }
}

impl ArchiveHandler<PushShard> for KeycardServer {
    fn handle(
        &mut self,
        msg: PushShard,
        _sender: xous::PID,
        _context: &mut ServerContext<Self>,
    ) -> <PushShard as server::Archive>::Response {
        self.push_shard(msg.shard, msg.accept_different_device_id)
    }
}

impl ArchiveHandler<FormatKeycard> for KeycardServer {
    fn handle(
        &mut self,
        msg: FormatKeycard,
        _sender: xous::PID,
        _context: &mut ServerContext<Self>,
    ) -> <FormatKeycard as server::Archive>::Response {
        self.format_keycard(msg.0 .0)
    }
}

impl ArchiveHandler<IdentifyKeycard> for KeycardServer {
    fn handle(
        &mut self,
        _msg: IdentifyKeycard,
        _sender: xous::PID,
        _context: &mut ServerContext<Self>,
    ) -> <IdentifyKeycard as server::Archive>::Response {
        self.identify_keycard().map(|(uid, err)| (KeycardId(uid), err))
    }
}

impl ArchiveHandler<StoreShardToKeycard> for KeycardServer {
    fn handle(
        &mut self,
        msg: StoreShardToKeycard,
        _sender: xous::PID,
        _context: &mut ServerContext<Self>,
    ) -> <StoreShardToKeycard as server::Archive>::Response {
        self.store_shard_to_keycard(msg.0 .0)
    }
}

impl ArchiveHandler<LoadShardFromKeycard> for KeycardServer {
    fn handle(
        &mut self,
        _msg: LoadShardFromKeycard,
        _sender: xous::PID,
        _context: &mut ServerContext<Self>,
    ) -> <LoadShardFromKeycard as server::Archive>::Response {
        self.load_shard_from_keycard()
    }
}

impl BlockingScalarHandler<CheckBackup> for KeycardServer {
    fn handle(
        &mut self,
        _msg: CheckBackup,
        _sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> <CheckBackup as server::BlockingScalar>::Response {
        self.check_backup()
    }
}

impl ArchiveHandler<RestoreMasterSeed> for KeycardServer {
    fn handle(
        &mut self,
        _msg: RestoreMasterSeed,
        _sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> <RestoreMasterSeed as server::Archive>::Response {
        self.restore_master_seed()
    }
}

impl ArchiveHandler<DetectKeycard> for KeycardServer {
    fn handle(
        &mut self,
        msg: DetectKeycard,
        _sender: xous::PID,
        _context: &mut ServerContext<Self>,
    ) -> <DetectKeycard as server::Archive>::Response {
        let (uid, _raw_msg) = self.nfc.read_ndef_raw_msg(msg.timeout)?;
        Ok(KeycardId(uid))
    }
}
