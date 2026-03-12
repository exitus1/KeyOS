// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use foundation_api::api::onboarding::OnboardingState;
use foundation_api::backup::RestoreMagicBackupEvent;
use foundation_api::bitcoin::{AccountUpdate, SignPsbt};
use foundation_api::firmware::FirmwareFetchEvent;
use foundation_api::fx::{ExchangeRate, ExchangeRateHistory};
use foundation_api::status::EnvoyStatus;
use quantum_link::{messages::*, ConnectionStatus, PairingEvent, SecurityCheckState};
use server::{ArchiveSubList, ScalarSubList};

#[derive(Default)]
pub struct MessageSubscribers {
    pub exchange_rate: ArchiveSubList<ExchangeRate>,
    pub exchange_rate_history: ArchiveSubList<ExchangeRateHistory>,
    pub firmware_fetch: ArchiveSubList<FirmwareFetchEvent>,
    pub envoy_status: ArchiveSubList<EnvoyStatus>,
    pub sign_psbt: ArchiveSubList<SignPsbt>,
    pub onboarding_state: ArchiveSubList<OnboardingState>,

    pub pairing_event: ArchiveSubList<PairingEvent>,
    pub security_check_state: ArchiveSubList<SecurityCheckState>,

    pub account_update: ArchiveSubList<AccountUpdate>,
    pub published_account_update: ArchiveSubList<SendAccountUpdate>,

    pub restore_magic_backup: ArchiveSubList<RestoreMagicBackupEvent>,

    pub connection_status: ScalarSubList<ConnectionStatus>,
}
