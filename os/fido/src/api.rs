// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use server::{CheckedConn, CheckedPermissions, MessageAllowed};

#[cfg(feature = "test-app")]
use crate::messages::ResetState;
use crate::{
    error::FidoError,
    messages::{
        CreateSecurityKey, CtapProcessCbor, GetSelectedSecurityKey, IsLive, NextSecurityKeyIndex,
        SelectSecurityKey, SetLive, Transport, U2fProcessApdu,
    },
};

#[macro_export]
macro_rules! use_api {
    () => {
        mod fido_permissions {
            use fido::messages::*;
            #[derive(Debug, Clone, Default, server::Permissions)]
            #[server_name = "os/fido"]
            pub struct FidoPermissions;
        }
        type FidoApi = fido::api::FidoApi<fido_permissions::FidoPermissions>;
    };
}

#[derive(Debug, Default)]
pub struct FidoApi<P: CheckedPermissions>(CheckedConn<P>);

impl<P: CheckedPermissions> FidoApi<P> {
    /* API for gui-app-security-keys application */

    // Get the Liveness of a given Security Key
    pub fn is_live(&self, index: usize) -> Result<bool, FidoError>
    where
        P: MessageAllowed<IsLive>,
    {
        self.0.try_send_blocking_scalar(IsLive(index))?
    }

    // Get the next Security Key index (without creating it)
    pub fn next_security_key_index(&self) -> Result<usize, FidoError>
    where
        P: MessageAllowed<NextSecurityKeyIndex>,
    {
        Ok(self.0.try_send_blocking_scalar(NextSecurityKeyIndex)?)
    }

    // Get the index of the selected Security Key if any
    pub fn selected_security_key_index(&self) -> Result<Option<usize>, FidoError>
    where
        P: MessageAllowed<GetSelectedSecurityKey>,
    {
        Ok(self.0.try_send_blocking_scalar(GetSelectedSecurityKey)?)
    }

    /// Select/Deselect a Security Key for Registration (fire-and-forget).
    pub fn select_security_key(&self, index: Option<usize>)
    where
        P: MessageAllowed<SelectSecurityKey>,
    {
        self.0.try_send_scalar(SelectSecurityKey(index)).ok();
    }

    /// Create a new Security Key (fire-and-forget).
    pub fn create_security_key(&self)
    where
        P: MessageAllowed<CreateSecurityKey>,
    {
        self.0.try_send_scalar(CreateSecurityKey).ok();
    }

    /// Set the Liveness of a given Security Key (fire-and-forget).
    pub fn set_live(&self, index: usize, live: bool)
    where
        P: MessageAllowed<SetLive>,
    {
        self.0.try_send_scalar(SetLive { index, live }).ok();
    }

    /* API for ctap-hid/nfc server */

    pub fn u2f_process_apdu(&self, msg: Vec<u8>, transport: Transport) -> Vec<u8>
    where
        P: MessageAllowed<U2fProcessApdu>,
    {
        self.0.send_archive(U2fProcessApdu { msg, transport })
    }

    pub fn ctap_process_cbor(&self, cmd: u8, raw: Vec<u8>) -> Vec<u8>
    where
        P: MessageAllowed<CtapProcessCbor>,
    {
        self.0.send_archive(CtapProcessCbor { cmd, raw })
    }

    /* API for Test Apps only */

    #[cfg(feature = "test-app")]
    pub fn reset_state(&mut self) -> Result<(), FidoError>
    where
        P: MessageAllowed<ResetState>,
    {
        self.0.try_send_blocking_scalar(ResetState)?
    }
}
