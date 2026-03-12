// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

mod command;
mod error;

use command::{
    AuthenticateRequest, AuthenticateResponse, Command, KeyHandle, RegisterRequest, RegisterResponse,
    VersionResponse,
};
use error::{Error, Status};
use gui_server_api::navigation::securitykeys::OperationOutcomeOptions;

use crate::{
    implementation::{FidoServer, SELECTION_TIMEOUT_SECONDS},
    messages::Transport,
};

impl FidoServer {
    fn _process_apdu(&mut self, data: &[u8], transport: Transport) -> Result<Vec<u8>, Error> {
        log::debug!("process_apdu({:02x?},{:?})", data, transport);
        let _class = data[0];
        let instruction = data[1];
        let p1 = data[2];
        let p2 = data[3];
        if p2 != 0 {
            return Err(Error::WrongParameter);
        }
        let data = &data[4..];
        match Command::try_from(instruction)? {
            Command::Register => {
                log::info!("Register");
                // gitlab.com/github.com/google.com send a 3 here !!!
                // if p1 != 0 {
                //     return Err(Error::WrongParameter);
                // }

                if !self.has_security_keys() {
                    log::warn!("No security keys available for registration");
                    self.notify_no_keys_warning();
                    return Err(Error::Other);
                }

                let req = RegisterRequest::from_apdu(data)?;
                log::debug!("{req:02x?}");

                // Determine the security key index based on transport type
                let security_key_index = if transport == Transport::Nfc {
                    // NFC requires a pre-selected key (can't delay communication)
                    let (index, _timestamp) = self
                        .state
                        .selected
                        .ok_or(Error::Other)
                        .inspect_err(|_| log::error!("No Key pre-selected for NFC!"))?;
                    index
                } else {
                    // USB: can prompt for selection during user presence check
                    let pre_selected = self.state.selected.map(|(idx, _)| idx);

                    let (confirmed, selected_index) = self
                        .check_user_presence(pre_selected, false, None, None, None, None)
                        .map_err(|_| Error::Other)
                        .inspect_err(|_| log::error!("User Presence error!"))?;
                    log::debug!("confirmed: {confirmed}, selected_index: {selected_index:?}");

                    if !confirmed {
                        log::error!("User Presence canceled!");
                        return Err(Error::Other);
                    }

                    // Use newly selected key if provided, otherwise use pre-selected
                    selected_index.or(pre_selected).ok_or_else(|| {
                        log::error!("No key selected after user presence check!");
                        Error::Other
                    })?
                };
                let (created_resgistered_key_index, user_public_key) =
                    self.create_registered_key_u2f(security_key_index, req.application_parameter)?;
                let key_handle =
                    KeyHandle { security_key_index, registered_key_index: created_resgistered_key_index };
                let mut resp =
                    RegisterResponse::new(user_public_key, key_handle, self.attestation_certificate.clone());
                resp.attest(&req.application_parameter, &req.challenge_parameter)?;
                log::debug!("{resp:02x?}");

                // Notify user of successful registration
                self.notify_operation_outcome(OperationOutcomeOptions::registration_success(
                    security_key_index,
                ));

                Ok(resp.to_vec())
            }
            Command::Authenticate => {
                log::info!("Authenticate");
                let req = AuthenticateRequest::from_apdu(data)?;
                log::debug!("{req:02x?}");
                let security_key_index = req.key_handle.security_key_index;
                let security_key = self.security_key(security_key_index)?;
                if !security_key.live
                    && !self.state.selected.is_some_and(|selected| selected.0 == security_key_index)
                {
                    // Abort if requested Security Key is not Live and not manually selected
                    log::error!("U2F trying to use a not-live and not-selected Key");
                    return Err(Error::WrongData);
                }
                let registered_key_index = req.key_handle.registered_key_index;
                let enforce_user_presence = match p1 {
                    0x07 => {
                        log::debug!("check-only");
                        let registered_key_indexes =
                            security_key.registered_key_indexes(true, &req.application_parameter);
                        if registered_key_indexes.contains(&registered_key_index) {
                            // note that despite the name this signals a success condition
                            return Err(Error::ConditionNotSatified);
                        } else {
                            log::error!("U2F asked for an unknown key handle");
                            return Err(Error::WrongData);
                        }
                    }
                    0x03 => {
                        log::debug!("enforce-user-presence-and-sign");
                        true
                    }
                    0x08 => {
                        log::debug!("dont-enforce-user-presence-and-sign");
                        false
                    }
                    _ => return Err(Error::WrongParameter),
                };
                let user_present = match self.state.selected {
                    Some((selected_security_key_index, selected_timestamp))
                        if selected_security_key_index == security_key_index
                            && crate::implementation::system_time() - selected_timestamp
                                < SELECTION_TIMEOUT_SECONDS =>
                    {
                        true
                    }
                    _ if transport == Transport::Nfc => true,
                    _ => {
                        let (confirmed, _) = self
                            .check_user_presence(Some(security_key_index), true, None, None, None, None)
                            .map_err(|_| Error::ConditionNotSatified)
                            .inspect_err(|_| log::error!("User Presence error!"))?;
                        confirmed
                    }
                };
                if enforce_user_presence && !user_present {
                    log::error!("User Presence canceled !");
                    return Err(Error::ConditionNotSatified);
                }
                let mut resp = AuthenticateResponse::new(user_present);
                let registered_key = security_key.registered_key(registered_key_index)?;
                resp.counter = registered_key.signature_counter();
                let mut signature_base = Vec::new();
                signature_base.extend_from_slice(&req.application_parameter);
                signature_base.push(resp.user_presence);
                signature_base.extend_from_slice(&resp.counter.to_be_bytes());
                signature_base.extend_from_slice(&req.challenge_parameter);
                (resp.signature, _) =
                    self.sign_der(security_key_index, registered_key_index, &signature_base)?;
                log::debug!("{resp:02x?}");

                // Notify user of successful authentication
                self.notify_operation_outcome(OperationOutcomeOptions::authentication_success(
                    security_key_index,
                ));

                Ok(resp.to_vec())
            }
            Command::Version => {
                log::info!("Version");
                if p1 != 0 {
                    return Err(Error::WrongParameter);
                }
                let resp = VersionResponse::prime();
                Ok(resp.to_vec())
            }
        }
    }

    pub fn u2f_process_apdu(&mut self, data: &[u8], transport: Transport) -> Vec<u8> {
        let res = self._process_apdu(data, transport);
        if res.is_err() {
            log::error!("{res:?}");
        }
        if let Err(e) = self.save_states() {
            log::error!("Failed to save FIDO states: {:?}", e);
        }
        Status::from(&res).to_vec(res.unwrap_or_default().as_slice())
    }
}
