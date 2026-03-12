// SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use security::{messages::*, Seed, MAX_LOGIN_ATTEMPTS};
use security::{
    AccessDenied, BluetoothChallengeSecret, DeviceId, FirmwareTimestamp, LoginFailed, MasterKeyState,
    ScProof, SecurityWord,
};

use crate::{seed_fingerprint, validate_raw_pin, CryptoApi};

const DATA_FILE_NAME: &str = "hosted_security_data.json";

#[derive(server::Server)]
#[name = "os/security"]
pub struct Server {
    data: SecurityData,
    crypto: CryptoApi,
    logged_in: bool,
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
struct SecurityData {
    raw_pin: String,
    login_attempts: u32,
    factory_reset_counter: u32,
    seed: Vec<u8>,
    seed_fingerprint: Vec<u8>,
    firmware_timestamp: [u8; 4],
    pin_entry_mode: u8,
}

impl SecurityData {
    fn new() -> Self {
        if let Some(data) = Self::try_load() {
            log::info!("Loaded security data");
            data
        } else {
            log::info!("Created default security data");
            Self::default()
        }
    }

    fn try_load() -> Option<Self> { serde_json::from_slice(&std::fs::read(DATA_FILE_NAME).ok()?).ok() }

    fn save(&self) {
        std::fs::write(DATA_FILE_NAME, serde_json::to_vec_pretty(self).unwrap())
            .expect("Could not save security data");
    }
}

impl Default for Server {
    fn default() -> Self {
        Self { data: SecurityData::new(), crypto: CryptoApi::default(), logged_in: false }
    }
}

impl server::ArchiveHandler<SetSeedAndPin> for Server {
    fn handle(
        &mut self,
        mut msg: SetSeedAndPin,
        _sender: server::xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> <SetSeedAndPin as server::Archive>::Response {
        // Validate PIN length before processing
        validate_raw_pin(&msg.pin.0)?;

        self.data.seed = msg.seed.to_vec();
        self.data.seed_fingerprint = seed_fingerprint(&self.crypto, &msg.seed).unwrap().to_vec();
        self.data.raw_pin = std::mem::take(&mut msg.pin.0);
        self.data.pin_entry_mode = msg.pin_entry.into();
        self.data.save();
        std::thread::sleep(std::time::Duration::from_millis(4000));
        Ok(())
    }
}

impl server::ArchiveHandler<ChangePin> for Server {
    fn handle(
        &mut self,
        mut msg: ChangePin,
        _sender: server::xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> <ChangePin as server::Archive>::Response {
        // Validate PIN length before processing
        validate_raw_pin(&msg.pin.0)?;

        if !self.logged_in {
            log::info!("not logged in, request would fail on hardware: ChangePin");
        }

        self.data.raw_pin = std::mem::take(&mut msg.pin.0);
        self.data.pin_entry_mode = msg.pin_entry.into();
        self.data.save();
        Ok(())
    }
}

impl server::ArchiveHandler<IsPinSet> for Server {
    fn handle(
        &mut self,
        _msg: IsPinSet,
        _sender: server::xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> <IsPinSet as server::Archive>::Response {
        Ok(self.data.raw_pin.len() > 0)
    }
}

impl server::ArchiveHandler<Login> for Server {
    fn handle(
        &mut self,
        msg: Login,
        _sender: server::xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> <Login as server::Archive>::Response {
        std::thread::sleep(std::time::Duration::from_millis(500));
        if msg.pin.0 == self.data.raw_pin {
            self.logged_in = true;
            self.data.login_attempts = 0;
            self.data.save();
            Ok(())
        } else {
            log::warn!("Login failed. Got {:?} instead of {:?}", msg.pin.0, self.data.raw_pin);

            self.data.login_attempts += 1;
            self.data.save();
            Err(LoginFailed { attempts_left: MAX_LOGIN_ATTEMPTS - self.data.login_attempts })
        }
    }
}

impl server::ArchiveHandler<GetAttemptsRemaining> for Server {
    fn handle(
        &mut self,
        _msg: GetAttemptsRemaining,
        _sender: server::xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> <GetAttemptsRemaining as server::Archive>::Response {
        Ok(MAX_LOGIN_ATTEMPTS - self.data.login_attempts)
    }
}

impl server::ArchiveHandler<GetFactoryResetCounter> for Server {
    fn handle(
        &mut self,
        _msg: GetFactoryResetCounter,
        _sender: server::xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> <GetFactoryResetCounter as server::Archive>::Response {
        Ok(self.data.factory_reset_counter)
    }
}

impl server::ArchiveHandler<GetSeed> for Server {
    fn handle(
        &mut self,
        _msg: GetSeed,
        _sender: server::xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> <GetSeed as server::Archive>::Response {
        if !self.logged_in {
            log::info!("not logged in, request would fail on hardware: GetSeed");
        }
        if self.data.seed.is_empty() {
            Ok(None)
        } else {
            let seed = Seed::from_bytes(self.data.seed.as_slice());
            Ok(Some(seed))
        }
    }
}

impl server::ArchiveHandler<SetSeed> for Server {
    fn handle(
        &mut self,
        msg: SetSeed,
        _sender: server::xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> <SetSeed as server::Archive>::Response {
        if !self.logged_in {
            log::info!("not logged in, request would fail on hardware: SetSeed");
        }

        self.data.seed = msg.0.to_vec();
        self.data.seed_fingerprint = seed_fingerprint(&self.crypto, &msg.0).unwrap().to_vec();
        self.data.save();
        Ok(())
    }
}

impl server::ArchiveHandler<GetAppSeed> for Server {
    fn handle(
        &mut self,
        _msg: GetAppSeed,
        sender: server::xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> <GetAppSeed as server::Archive>::Response {
        if !self.logged_in {
            log::info!("not logged in, request would fail on hardware: GetAppSeed");
        }

        let sender_app_id =
            xous::get_app_id(sender).map_err(|_| AccessDenied)?.ok_or(AccessDenied)?.0.to_vec();
        let master_seed = self.data.seed.clone();
        let app_seed: [u8; 32] = self
            .crypto
            .hmac256(sender_app_id, master_seed)
            .map_err(|_| AccessDenied)?
            .try_into()
            .expect("incorrect slice length");
        Ok(app_seed)
    }
}

impl server::ArchiveHandler<GetFirmwareTimestamp> for Server {
    fn handle(
        &mut self,
        _msg: GetFirmwareTimestamp,
        _sender: server::xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> <GetFirmwareTimestamp as server::Archive>::Response {
        Ok(FirmwareTimestamp(self.data.firmware_timestamp))
    }
}

impl server::ArchiveHandler<SetFirmwareTimestamp> for Server {
    fn handle(
        &mut self,
        msg: SetFirmwareTimestamp,
        _sender: server::xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> <SetFirmwareTimestamp as server::Archive>::Response {
        if !self.logged_in {
            log::info!("not logged in, request would fail on hardware: SetFirmwareTimestamp");
        }

        self.data.firmware_timestamp = msg.0 .0;
        self.data.save();
        Ok(())
    }
}

impl server::BlockingScalarHandler<Logout> for Server {
    fn handle(
        &mut self,
        _msg: Logout,
        _sender: server::xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> <Logout as server::BlockingScalar>::Response {
        self.logged_in = false;
    }
}

impl server::BlockingScalarHandler<LoggedIn> for Server {
    fn handle(
        &mut self,
        _msg: LoggedIn,
        _sender: server::xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> <LoggedIn as server::BlockingScalar>::Response {
        self.logged_in
    }
}

#[cfg(not(keyos))]
impl server::ArchiveHandler<GetPin> for Server {
    fn handle(
        &mut self,
        _msg: GetPin,
        _sender: server::xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> <GetPin as server::Archive>::Response {
        self.data.raw_pin.clone()
    }
}

impl server::ArchiveHandler<Lockout> for Server {
    fn handle(
        &mut self,
        _msg: Lockout,
        _sender: server::xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> <Lockout as server::Archive>::Response {
        if !self.logged_in {
            log::info!("not logged in, request would fail on hardware: Lockout");
        }

        self.data.factory_reset_counter = self.data.factory_reset_counter + 1;
        self.data.seed = vec![0; 32];
        self.data.seed_fingerprint = vec![0; 32];
        self.data.save();
        Ok(())
    }
}

impl server::ArchiveHandler<SignWithSecurityCheckKey> for Server {
    fn handle(
        &mut self,
        _msg: SignWithSecurityCheckKey,
        _sender: server::xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> <SignWithSecurityCheckKey as server::Archive>::Response {
        if !self.logged_in {
            log::info!("not logged in, request would fail on hardware: SignWithSecurityCheckKey");
        }

        Ok([1; 64])
    }
}

impl server::ArchiveHandler<SignWithFidoKey> for Server {
    fn handle(
        &mut self,
        _msg: SignWithFidoKey,
        _sender: server::xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> <SignWithFidoKey as server::Archive>::Response {
        if !self.logged_in {
            log::info!("not logged in, request would fail on hardware: SignWithFidoKey");
        }

        Ok([1; 64])
    }
}

impl server::ArchiveHandler<GetFidoPubkey> for Server {
    fn handle(
        &mut self,
        _msg: GetFidoPubkey,
        _sender: server::xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> <GetFidoPubkey as server::Archive>::Response {
        if !self.logged_in {
            log::info!("not logged in, request would fail on hardware: GetFidoPubkey");
        }

        Ok([1; 64])
    }
}

impl server::ArchiveHandler<GetSecurityWords> for Server {
    fn handle(
        &mut self,
        _msg: GetSecurityWords,
        _sender: server::xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> <GetSecurityWords as server::Archive>::Response {
        Ok([SecurityWord(0), SecurityWord(1)])
    }
}

impl server::ArchiveHandler<SetAttempts> for Server {
    fn handle(
        &mut self,
        msg: SetAttempts,
        _sender: server::xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> <SetAttempts as server::Archive>::Response {
        self.data.login_attempts = msg.0;
        self.data.save();
    }
}

impl server::ArchiveHandler<GetSeedFingerprint> for Server {
    fn handle(
        &mut self,
        _msg: GetSeedFingerprint,
        _sender: server::xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> <GetSeedFingerprint as server::Archive>::Response {
        Ok(self.data.seed_fingerprint.as_slice().try_into().unwrap())
    }
}

impl server::ArchiveHandler<ComputeSeedFingerprint> for Server {
    fn handle(
        &mut self,
        msg: ComputeSeedFingerprint,
        _sender: server::xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> <ComputeSeedFingerprint as server::Archive>::Response {
        seed_fingerprint(&self.crypto, &msg.0).map_err(|_| AccessDenied)
    }
}

impl server::ArchiveHandler<GetOsVersionInfo> for Server {
    fn handle(
        &mut self,
        _msg: GetOsVersionInfo,
        _sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> <GetOsVersionInfo as server::Archive>::Response {
        Ok(None)
    }
}

impl server::ArchiveHandler<ScChallenge> for Server {
    fn handle(
        &mut self,
        _msg: ScChallenge,
        _sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> <ScChallenge as server::Archive>::Response {
        Ok(ScProof([0u8; ScProof::SIZE]))
    }
}

impl server::ArchiveHandler<GetDeviceId> for Server {
    fn handle(
        &mut self,
        _msg: GetDeviceId,
        _sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> <GetDeviceId as server::Archive>::Response {
        Ok(DeviceId([0u8; 32]))
    }
}

impl server::ArchiveHandler<KeycardAuthenticityMac> for Server {
    fn handle(
        &mut self,
        _msg: KeycardAuthenticityMac,
        _sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> <KeycardAuthenticityMac as server::Archive>::Response {
        Ok([0u8; 32])
    }
}

impl server::ArchiveHandler<GetBluetoothChallengeSecret> for Server {
    fn handle(
        &mut self,
        _msg: GetBluetoothChallengeSecret,
        _sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> BluetoothChallengeSecret {
        BluetoothChallengeSecret { secret: Default::default(), sent: false }
    }
}

impl server::BlockingScalarHandler<SetBluetoothCheckSecretSent> for Server {
    fn handle(
        &mut self,
        _msg: SetBluetoothCheckSecretSent,
        _sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) {
    }
}

impl server::ArchiveHandler<SetBluetoothDeviceId> for Server {
    fn handle(
        &mut self,
        _msg: SetBluetoothDeviceId,
        _sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) {
    }
}

impl server::ArchiveHandler<GetRandom> for Server {
    fn handle(
        &mut self,
        _msg: GetRandom,
        _sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> <GetRandom as server::Archive>::Response {
        let random_bytes = rand::random::<[u8; 32]>();
        Ok(random_bytes)
    }
}

impl server::ArchiveHandler<GetPinEntryMode> for Server {
    fn handle(
        &mut self,
        _msg: GetPinEntryMode,
        _sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> <GetPinEntryMode as server::Archive>::Response {
        self.data.pin_entry_mode.into()
    }
}

impl server::BlockingScalarHandler<GetMasterKeyState> for Server {
    fn handle(
        &mut self,
        _msg: GetMasterKeyState,
        _sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> MasterKeyState {
        let (seed_empty, pin_empty) = (self.data.seed.is_empty(), self.data.raw_pin.is_empty());

        match (seed_empty, pin_empty) {
            (true, false) => MasterKeyState::Erased,
            (_, true) => MasterKeyState::Onboarding,
            (false, false) => MasterKeyState::Normal,
        }
    }
}

impl server::ArchiveHandler<GetBootloaderBuildDate> for Server {
    fn handle(
        &mut self,
        _msg: GetBootloaderBuildDate,
        _sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> Result<Option<u64>, AccessDenied> {
        Ok(None)
    }
}

impl server::Server for Server {}
