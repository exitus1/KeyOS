// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use file_backed::{FileBacked, JsonBacked, JsonCodec};
use gui_server_api::navigation::securitykeys::{OperationOutcomeOptions, UserPresenceOptions};
use p256::ecdsa::{signature::Signer, Signature, SigningKey, VerifyingKey};
use server::{ArchiveHandler, BlockingScalarHandler, ScalarHandler, Server, ServerContext};

#[cfg(feature = "test-app")]
use crate::messages::ResetState;
use crate::{
    ctap::{PublicKeyCredentialRpEntity, PublicKeyCredentialUserEntity},
    error::FidoError,
    implementation::fs_permissions::FileSystemPermissions,
    messages::{
        CreateSecurityKey, CtapProcessCbor, GetSelectedSecurityKey, IsLive, NextSecurityKeyIndex,
        SelectSecurityKey, SetLive, U2fProcessApdu,
    },
    CryptoApi, RegisteredKey, RegisteredKeyCtap, RegisteredKeyU2f, SecurityKey,
};

fs::use_api!();
gui_server_api::use_api!();
security::use_api!();
settings::use_api!();

const STATE_FILE: &str = "security_keys_v1.json";
pub(crate) const SELECTION_TIMEOUT_SECONDS: u32 = 30;

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct FidoKeysState {
    pub security_keys: Vec<SecurityKey>,
    #[serde(skip)]
    pub selected: Option<(usize, u32)>,
}

impl FidoKeysState {
    fn security_key_mut(&mut self, index: usize) -> Result<&mut SecurityKey, FidoError> {
        self.security_keys.get_mut(index).ok_or(FidoError::InvalidIndex)
    }
}

#[derive(Debug, Default)]
pub struct FidoKey {
    signing_keys: Vec<SigningKey>,
    next: Option<(SigningKey, Vec<u8>)>,
}

/// Official attestation certificate (DER-encoded X.509)
const OFFICIAL_CERTIFICATE: [u8; 353] = [
    0x30, 0x82, 0x01, 0x5d, 0x30, 0x82, 0x01, 0x02, 0xa0, 0x03, 0x02, 0x01, 0x02, 0x02, 0x14, 0x68, 0x44,
    0x90, 0x6b, 0x09, 0x8e, 0x6c, 0x32, 0xe9, 0x4b, 0x03, 0xe5, 0x57, 0x46, 0x89, 0xf2, 0x93, 0xcb, 0xfd,
    0x89, 0x30, 0x0a, 0x06, 0x08, 0x2a, 0x86, 0x48, 0xce, 0x3d, 0x04, 0x03, 0x02, 0x30, 0x2e, 0x31, 0x2c,
    0x30, 0x2a, 0x06, 0x03, 0x55, 0x04, 0x03, 0x0c, 0x23, 0x46, 0x6f, 0x75, 0x6e, 0x64, 0x61, 0x74, 0x69,
    0x6f, 0x6e, 0x20, 0x44, 0x65, 0x76, 0x69, 0x63, 0x65, 0x73, 0x20, 0x46, 0x49, 0x44, 0x4f, 0x20, 0x41,
    0x74, 0x74, 0x65, 0x73, 0x74, 0x61, 0x74, 0x69, 0x6f, 0x6e, 0x30, 0x1e, 0x17, 0x0d, 0x32, 0x36, 0x30,
    0x31, 0x30, 0x31, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x5a, 0x17, 0x0d, 0x33, 0x36, 0x30, 0x31, 0x30,
    0x31, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x5a, 0x30, 0x2e, 0x31, 0x2c, 0x30, 0x2a, 0x06, 0x03, 0x55,
    0x04, 0x03, 0x0c, 0x23, 0x46, 0x6f, 0x75, 0x6e, 0x64, 0x61, 0x74, 0x69, 0x6f, 0x6e, 0x20, 0x44, 0x65,
    0x76, 0x69, 0x63, 0x65, 0x73, 0x20, 0x46, 0x49, 0x44, 0x4f, 0x20, 0x41, 0x74, 0x74, 0x65, 0x73, 0x74,
    0x61, 0x74, 0x69, 0x6f, 0x6e, 0x30, 0x59, 0x30, 0x13, 0x06, 0x07, 0x2a, 0x86, 0x48, 0xce, 0x3d, 0x02,
    0x01, 0x06, 0x08, 0x2a, 0x86, 0x48, 0xce, 0x3d, 0x03, 0x01, 0x07, 0x03, 0x42, 0x00, 0x04, 0xa8, 0x61,
    0xfe, 0xad, 0x21, 0xc2, 0xdc, 0x3e, 0xe9, 0x81, 0xb2, 0xbc, 0x27, 0x91, 0x33, 0x23, 0x83, 0xf0, 0x9e,
    0xe6, 0xce, 0x9f, 0x1e, 0x25, 0x00, 0x34, 0x46, 0x2c, 0xac, 0x12, 0xae, 0xfa, 0x03, 0x26, 0xff, 0xc2,
    0x3d, 0x2a, 0xf0, 0xe2, 0xe8, 0x87, 0xff, 0xf9, 0x05, 0x93, 0x08, 0xa7, 0x7f, 0x10, 0x69, 0x70, 0x5d,
    0xaf, 0x41, 0x4d, 0xb2, 0xb0, 0x6d, 0xcd, 0x35, 0x77, 0xc3, 0x58, 0x30, 0x0a, 0x06, 0x08, 0x2a, 0x86,
    0x48, 0xce, 0x3d, 0x04, 0x03, 0x02, 0x03, 0x49, 0x00, 0x30, 0x46, 0x02, 0x21, 0x00, 0xb2, 0xa4, 0x21,
    0x38, 0xab, 0x3a, 0x42, 0x7e, 0x5a, 0x98, 0xfb, 0x6d, 0x02, 0x46, 0x81, 0xe0, 0xfa, 0x30, 0x38, 0x81,
    0xb5, 0xbc, 0x19, 0xeb, 0x50, 0x30, 0x82, 0x12, 0x30, 0x1c, 0x30, 0x2d, 0x02, 0x21, 0x00, 0xc1, 0x28,
    0xd6, 0xb6, 0xb8, 0xc0, 0x32, 0xeb, 0xb0, 0x7c, 0x11, 0xf4, 0xd3, 0xe6, 0xd4, 0x94, 0x43, 0xa3, 0xfd,
    0x12, 0x92, 0x1b, 0x5d, 0xa8, 0x2c, 0x6d, 0x44, 0x41, 0x5c, 0x8b, 0xa8, 0x49,
];

/// Official attestation pubkey (65 bytes: 0x04 prefix + 64-byte uncompressed point)
const OFFICIAL_PUBKEY: [u8; 65] = [
    0x04, 0xa8, 0x61, 0xfe, 0xad, 0x21, 0xc2, 0xdc, 0x3e, 0xe9, 0x81, 0xb2, 0xbc, 0x27, 0x91, 0x33, 0x23,
    0x83, 0xf0, 0x9e, 0xe6, 0xce, 0x9f, 0x1e, 0x25, 0x00, 0x34, 0x46, 0x2c, 0xac, 0x12, 0xae, 0xfa, 0x03,
    0x26, 0xff, 0xc2, 0x3d, 0x2a, 0xf0, 0xe2, 0xe8, 0x87, 0xff, 0xf9, 0x05, 0x93, 0x08, 0xa7, 0x7f, 0x10,
    0x69, 0x70, 0x5d, 0xaf, 0x41, 0x4d, 0xb2, 0xb0, 0x6d, 0xcd, 0x35, 0x77, 0xc3, 0x58,
];

#[derive(server::Server)]
#[name = "os/fido"]
pub struct FidoServer {
    crypto: CryptoApi,
    gui_api: GuiApiLight,
    pub(crate) aaguid: [u8; 16],
    pub(crate) state: JsonBacked<FidoKeysState, FileSystemPermissions>,
    pub(crate) attestation_certificate: Vec<u8>,
    pub(crate) attestation_pubkey: Vec<u8>,
    seed: Vec<u8>,
    fido_keys: Vec<FidoKey>,
}

impl Server for FidoServer {}

/// wait for:
/// - backup restore to complete
/// - secure element to unlock and give us our app_seed
pub fn wait() -> (Security, [u8; 32]) {
    let settings = SettingsApi::default();
    settings.wait_for_onboarding_complete();

    let security = Security::default();
    let seed = security.app_seed().expect("app seed");

    (security, seed)
}

impl FidoServer {
    pub fn new(security: Security, seed: [u8; 32]) -> Result<Self, FidoError> {
        log::info!("starting fido server");
        let mut state: FileBacked<JsonCodec<FidoKeysState>, _> =
            JsonBacked::new(STATE_FILE, fs::Location::AppData).0;
        state.set_auto_save(false);
        log::debug!("Restored State: {:02x?}", state);

        // Get the SE's FIDO public key (64 bytes without 0x04 prefix)
        let se_pubkey = security
            .get_fido_pubkey()
            .inspect_err(|e| log::error!("security.get_fido_pubkey {e:?}"))
            .map_err(|_| FidoError::Other)?;

        // Check if SE pubkey matches the official pubkey (compare without 0x04 prefix)
        let (attestation_certificate, attestation_pubkey) = if se_pubkey[..] == OFFICIAL_PUBKEY[1..] {
            log::info!("Using official attestation certificate");
            (OFFICIAL_CERTIFICATE.to_vec(), OFFICIAL_PUBKEY.to_vec())
        } else {
            log::info!("Non-official pubkey detected, generating attestation certificate");
            let cert = crate::attestation_cert::build_attestation_certificate(&se_pubkey, |hash| {
                let sig = security.sign_with_fido_key(hash).map_err(|_| FidoError::Other)?;
                sig.try_into().map_err(|_| FidoError::Other)
            })?;

            // Build 65-byte pubkey with 0x04 prefix for non-official case
            let mut pubkey = Vec::with_capacity(65);
            pubkey.push(0x04);
            pubkey.extend_from_slice(&se_pubkey);
            (cert, pubkey)
        };

        let mut fido_server = Self {
            crypto: CryptoApi::default(),
            gui_api: GuiApiLight::default(),
            state,
            aaguid: [
                0x8f, 0x1b, 0xcc, 0xae, 0xeb, 0x8f, 0x12, 0xf8, 0x0b, 0x01, 0x7f, 0x55, 0x77, 0x4e, 0x3c,
                0xf5,
            ],
            attestation_certificate,
            attestation_pubkey,
            seed: seed.to_vec(),
            fido_keys: Vec::new(),
        };
        fido_server.populate_fido_keys()?;
        fido_server.compute_next_signing_keys()?;
        log::debug!("FIDO Keys: {:02x?}", fido_server.fido_keys);
        Ok(fido_server)
    }

    fn populate_fido_keys(&mut self) -> Result<(), FidoError> {
        self.fido_keys = self
            .state
            .security_keys
            .iter()
            .enumerate()
            .map(|(security_key_index, security_key)| -> Result<FidoKey, FidoError> {
                let signing_keys = security_key
                    .registered_keys
                    .iter()
                    .enumerate()
                    .map(|(registered_key_index, _registered_key)| {
                        self.signing_key(security_key_index, registered_key_index)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(FidoKey { signing_keys, next: None })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(())
    }

    fn fido_key(&self, security_key_index: usize) -> Result<&FidoKey, FidoError> {
        self.fido_keys.get(security_key_index).ok_or(FidoError::InvalidIndex)
    }

    fn fido_key_mut(&mut self, security_key_index: usize) -> Result<&mut FidoKey, FidoError> {
        self.fido_keys.get_mut(security_key_index).ok_or(FidoError::InvalidIndex)
    }

    fn compute_next_signing_key_if_needed(&mut self, security_key_index: usize) -> Result<(), FidoError> {
        let needs_generation = self.fido_key(security_key_index)?.next.is_none();
        if needs_generation {
            let signing_keys_len =
                self.fido_key(security_key_index).map(|fido_key| fido_key.signing_keys.len()).unwrap_or(0);
            let next_signing_key = self.signing_key(security_key_index, signing_keys_len)?;
            let next_public_key = VerifyingKey::from(&next_signing_key).to_sec1_bytes().as_ref().to_vec();
            self.fido_key_mut(security_key_index)?.next = Some((next_signing_key, next_public_key));
        }
        Ok(())
    }

    fn compute_next_signing_keys(&mut self) -> Result<(), FidoError> {
        for i in 0..self.fido_keys.len() {
            self.compute_next_signing_key_if_needed(i)?;
        }
        Ok(())
    }

    fn use_next_signing_key(&mut self, security_key_index: usize) -> Result<Vec<u8>, FidoError> {
        self.compute_next_signing_key_if_needed(security_key_index)?;
        let fido_key = self.fido_key_mut(security_key_index)?;
        // below should never panic because the call to compute_next_signing_key_if_needed above
        // guarantee next.is_some()
        let next = fido_key.next.take().ok_or(FidoError::Other)?;
        fido_key.signing_keys.push(next.0);
        Ok(next.1)
    }

    pub(crate) fn save_states(&mut self) -> Result<(), FidoError> {
        self.compute_next_signing_keys()?;
        self.state.save();
        Ok(())
    }

    #[cfg(feature = "test-app")]
    fn reset_state(&mut self) -> Result<(), FidoError> {
        self.fido_keys = Vec::new();
        self.compute_next_signing_keys()?;
        self.state.guard().0 = FidoKeysState::default();
        Ok(())
    }

    fn create_security_key(&mut self) -> Result<usize, FidoError> {
        log::info!("creating new security key");
        self.fido_keys.push(FidoKey::default());
        self.compute_next_signing_keys()?;
        let mut state = self.state.guard();
        let new_index = state.security_keys.len();
        state.security_keys.push(SecurityKey::default());
        Ok(new_index)
    }

    fn select_security_key(&mut self, index: Option<usize>) -> Result<(), FidoError> {
        if let Some(idx) = index {
            if idx >= self.state.security_keys.len() {
                return Err(FidoError::InvalidIndex);
            }
        }
        let system_time = system_time();
        let mut state = self.state.guard();
        state.selected = index.map(|idx| (idx, system_time));
        Ok(())
    }

    /// Checks for user presence and optionally allows the user to select a key.
    ///
    /// # Arguments
    /// * `security_key_index` - The key index to use, or None to allow the user to select.
    /// * `authentication` - Whether this is an authentication (true) or registration (false).
    /// * `rp_id`, `rp_name`, `user_name`, `user_display_name` - Optional metadata to display.
    ///
    /// # Returns
    /// * `Ok((present, selected_key_index))` - Whether user confirmed and the selected key index.
    /// * `Err(())` - If the GUI API call failed.
    pub(crate) fn check_user_presence(
        &self,
        security_key_index: Option<usize>,
        authentication: bool,
        rp_id: Option<String>,
        rp_name: Option<String>,
        user_name: Option<String>,
        user_display_name: Option<String>,
    ) -> Result<(bool, Option<usize>), ()> {
        let mut opts = if authentication {
            UserPresenceOptions::authentication(security_key_index)
        } else {
            UserPresenceOptions::registration(security_key_index)
        };
        if let Some(rp_id) = rp_id {
            opts = opts.with_rp_id(rp_id);
        }
        if let Some(rp_name) = rp_name {
            opts = opts.with_rp_name(rp_name);
        }
        if let Some(user_name) = user_name {
            opts = opts.with_user_name(user_name);
        }
        if let Some(user_display_name) = user_display_name {
            opts = opts.with_user_display_name(user_display_name);
        }
        let result = self.gui_api.check_user_presence(opts).map_err(|_| ())?.ok_or(())?;
        Ok((result.present(), result.selected_key_index()))
    }

    /// Notifies the user about the outcome of a FIDO operation.
    ///
    /// This is a "fire and forget" notification - we don't wait for user acknowledgment.
    /// Currently implemented as a placeholder until the Security Keys app has the outcome modal.
    pub(crate) fn notify_operation_outcome(&self, options: OperationOutcomeOptions) {
        if let Err(e) = self.gui_api.notify_operation_outcome(options) {
            log::warn!("Failed to notify operation outcome: {:?}", e);
        }
    }

    /// Checks if there are any security keys created.
    pub(crate) fn has_security_keys(&self) -> bool { !self.state.security_keys.is_empty() }

    /// Notifies the GUI that no keys are available for registration.
    /// This is a fire-and-forget async notification.
    pub(crate) fn notify_no_keys_warning(&self) {
        if let Err(e) = self.gui_api.notify_no_keys_warning() {
            log::warn!("Failed to send no keys warning: {:?}", e);
        }
    }

    // TODO: only used in CTAP2 process, should be rethinked using Selected/Live attributes of SecurityKey
    pub(crate) fn security_key_index(
        &self,
        force_security_key_index: Option<usize>,
    ) -> Result<usize, FidoError> {
        Ok(force_security_key_index.unwrap_or(self.state.selected.ok_or(FidoError::UnselectedKey)?.0))
    }

    pub(crate) fn security_key(&self, index: usize) -> Result<&SecurityKey, FidoError> {
        self.state.security_keys.get(index).ok_or(FidoError::InvalidIndex)
    }

    pub(crate) fn create_registered_key_u2f(
        &mut self,
        security_key_index: usize,
        application_parameter: [u8; 32],
    ) -> Result<(usize, Vec<u8>), FidoError> {
        let public_key = self.use_next_signing_key(security_key_index)?;
        let registered_timestamp = system_time();
        let mut state = self.state.guard();
        let security_key = state.security_key_mut(security_key_index)?;
        let new_registered_key_index = security_key.registered_keys.len();
        security_key.registered_keys.push(RegisteredKey::U2f(RegisteredKeyU2f {
            application_parameter,
            signature_counter: 0,
            registered_timestamp,
        }));
        Ok((new_registered_key_index, public_key))
    }

    pub(crate) fn create_registered_key_ctap(
        &mut self,
        security_key_index: usize,
        rp: PublicKeyCredentialRpEntity,
        user: PublicKeyCredentialUserEntity,
    ) -> Result<(usize, Vec<u8>), FidoError> {
        let public_key = self.use_next_signing_key(security_key_index)?;
        let registered_timestamp = system_time();
        let mut state = self.state.guard();
        let security_key = state.security_key_mut(security_key_index)?;
        let new_resgistered_key_index = security_key.registered_keys.len();
        security_key.registered_keys.push(RegisteredKey::Ctap(RegisteredKeyCtap {
            rp,
            user,
            signature_counter: 0,
            registered_timestamp,
        }));
        Ok((new_resgistered_key_index, public_key))
    }

    fn signing_key(
        &self,
        security_key_index: usize,
        registered_key_index: usize,
    ) -> Result<SigningKey, FidoError> {
        let derivation_path =
            format!("m/83696968’/1179473391’/{}/{}", security_key_index, registered_key_index);
        // TODO: navigate to `settings` app to input the PIN if needed in order to login to `security` server
        // TODO: if we inputed the PIN, save it as auto UV
        let derivated_seed = self.crypto.hmac256(derivation_path.as_bytes().to_vec(), self.seed.clone())?;
        let signing_key = SigningKey::from_slice(&derivated_seed).map_err(|_| FidoError::Ecdsa)?;
        Ok(signing_key)
    }

    pub(crate) fn verifying_key_sec1(
        &self,
        security_key_index: usize,
        registered_key_index: usize,
    ) -> Result<Vec<u8>, FidoError> {
        let signing_key = self.signing_key(security_key_index, registered_key_index)?;
        let verifying_key = VerifyingKey::from(&signing_key).to_sec1_bytes().as_ref().to_vec();
        Ok(verifying_key)
    }

    pub(crate) fn sign_der(
        &mut self,
        security_key_index: usize,
        registered_key_index: usize,
        data: &[u8],
    ) -> Result<(Vec<u8>, u32), FidoError> {
        let signing_key = self
            .fido_key(security_key_index)?
            .signing_keys
            .get(registered_key_index)
            .ok_or(FidoError::InvalidIndex)?;
        let signature: Signature = signing_key.sign(data);
        let mut state = self.state.guard();
        let security_key = state.security_key_mut(security_key_index)?;
        let registered_key = security_key.registered_key_mut(registered_key_index)?;
        let signature_counter = registered_key.inc_signature_counter();
        Ok((signature.to_der().as_bytes().to_vec(), signature_counter))
    }
}

pub(crate) fn system_time() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs() as u32
}

impl BlockingScalarHandler<IsLive> for FidoServer {
    fn handle(
        &mut self,
        msg: IsLive,
        _sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> Result<bool, FidoError> {
        Ok(self.security_key(msg.0)?.live)
    }
}

impl BlockingScalarHandler<NextSecurityKeyIndex> for FidoServer {
    fn handle(
        &mut self,
        _msg: NextSecurityKeyIndex,
        _sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> usize {
        self.state.security_keys.len()
    }
}

impl BlockingScalarHandler<GetSelectedSecurityKey> for FidoServer {
    fn handle(
        &mut self,
        _msg: GetSelectedSecurityKey,
        _sender: xous::PID,
        _context: &mut ServerContext<Self>,
    ) -> Option<usize> {
        self.state.selected.map(|(index, _)| index).clone()
    }
}

impl ScalarHandler<SelectSecurityKey> for FidoServer {
    fn handle(&mut self, msg: SelectSecurityKey, _sender: xous::PID, _context: &mut ServerContext<Self>) {
        if let Err(e) = self.select_security_key(msg.0) {
            log::warn!("select_security_key failed: {}", e);
        }
    }
}

impl ScalarHandler<CreateSecurityKey> for FidoServer {
    fn handle(&mut self, _msg: CreateSecurityKey, _sender: xous::PID, _context: &mut ServerContext<Self>) {
        if let Err(e) = self.create_security_key() {
            log::warn!("create_security_key failed: {:?}", e);
        }
    }
}

impl ScalarHandler<SetLive> for FidoServer {
    fn handle(&mut self, msg: SetLive, _sender: xous::PID, _context: &mut ServerContext<Self>) {
        let mut state = self.state.guard();
        if let Err(e) = state.security_key_mut(msg.index).map(|k| k.live = msg.live) {
            log::warn!("set_live failed: {:?}", e);
        }
    }
}

impl ArchiveHandler<U2fProcessApdu> for FidoServer {
    fn handle(
        &mut self,
        msg: U2fProcessApdu,
        _sender: xous::PID,
        _context: &mut ServerContext<Self>,
    ) -> <U2fProcessApdu as server::Archive>::Response {
        self.u2f_process_apdu(&msg.msg, msg.transport)
    }
}

impl ArchiveHandler<CtapProcessCbor> for FidoServer {
    fn handle(
        &mut self,
        msg: CtapProcessCbor,
        _sender: xous::PID,
        _context: &mut ServerContext<Self>,
    ) -> <CtapProcessCbor as server::Archive>::Response {
        self.ctap_process_cbor(msg.cmd, &msg.raw)
    }
}

#[cfg(feature = "test-app")]
impl BlockingScalarHandler<ResetState> for FidoServer {
    fn handle(
        &mut self,
        _msg: ResetState,
        _sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> Result<(), FidoError> {
        self.reset_state()?;
        Ok(())
    }
}
