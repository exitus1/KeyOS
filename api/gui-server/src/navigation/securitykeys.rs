// SPDX-FileCopyrightText: 2024-2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Security Keys navigation request and response formats.
//!
//! This module contains types for:
//! - User presence verification requests
//! - Operation outcome notifications (registration/authentication success/failure)

/// Unified navigation request enum for the Security Keys app.
///
/// This enum wraps all possible navigation request types, providing explicit
/// type discrimination during deserialization. This avoids ambiguity that could
/// arise from deserializing raw bytes as multiple different struct types.
#[derive(Debug, rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)]
pub enum SecurityKeysNavRequest {
    UserPresence(UserPresenceOptions),
    OperationOutcome(OperationOutcomeOptions),
    /// Fire-and-forget notification when no security keys are available for registration.
    NoKeysWarning,
}

impl SecurityKeysNavRequest {
    pub fn from_slice(data: &[u8]) -> Option<Self> {
        let Ok(archived) = rkyv::access::<ArchivedSecurityKeysNavRequest, rkyv::rancor::Error>(data) else {
            return None;
        };
        rkyv::deserialize::<Self, rkyv::rancor::Error>(archived).ok()
    }

    pub fn serialize(&self) -> Vec<u8> {
        rkyv::to_bytes::<rkyv::rancor::Error>(self).map(|b| b.to_vec()).unwrap_or_default()
    }
}

/// Options for the User Presence navigation request.
///
/// ```rust,ignore
/// # use gui_server_api::navigation::securitykeys::{UserPresenceOptions};
/// let options = UserPresenceOptions::authentication(Some(0)).with_rp_id("foundation.xyz".to_string());
/// ```
#[derive(Debug, rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)]
pub struct UserPresenceOptions {
    /// The security key index to use, or None to allow the user to select one.
    pub security_key_index: Option<usize>,
    pub authentication: bool,
    pub rp_id: Option<String>,
    pub rp_name: Option<String>,
    pub user_name: Option<String>,
    pub user_display_name: Option<String>,
}

impl UserPresenceOptions {
    pub fn registration(security_key_index: Option<usize>) -> Self {
        Self {
            security_key_index,
            authentication: false,
            rp_id: None,
            rp_name: None,
            user_name: None,
            user_display_name: None,
        }
    }

    pub fn authentication(security_key_index: Option<usize>) -> Self {
        Self {
            security_key_index,
            authentication: true,
            rp_id: None,
            rp_name: None,
            user_name: None,
            user_display_name: None,
        }
    }

    pub fn with_rp_id(self, rp_id: String) -> Self { Self { rp_id: Some(rp_id), ..self } }

    pub fn with_rp_name(self, rp_name: String) -> Self { Self { rp_name: Some(rp_name), ..self } }

    pub fn with_user_name(self, user_name: String) -> Self { Self { user_name: Some(user_name), ..self } }

    pub fn with_user_display_name(self, user_display_name: String) -> Self {
        Self { user_display_name: Some(user_display_name), ..self }
    }

    pub fn from_slice(data: &[u8]) -> Option<Self> {
        let Ok(archived) = rkyv::access::<ArchivedUserPresenceOptions, rkyv::rancor::Error>(data) else {
            return None;
        };
        rkyv::deserialize::<Self, rkyv::rancor::Error>(archived).ok()
    }

    pub fn serialize(&self) -> Vec<u8> { rkyv::to_bytes::<rkyv::rancor::Error>(self).unwrap().to_vec() }
}

#[derive(Debug, rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)]
pub struct UserPresenceResult {
    present: bool,
    /// The selected key index when user confirms presence.
    /// This is used when the initial security_key_index was None, allowing
    /// the user to select a key during the user presence check.
    selected_key_index: Option<usize>,
}

impl UserPresenceResult {
    pub fn new_checked(selected_key_index: Option<usize>) -> Self {
        UserPresenceResult { present: true, selected_key_index }
    }

    pub fn new_cancelled() -> Self { UserPresenceResult { present: false, selected_key_index: None } }

    pub fn present(&self) -> bool { self.present }

    pub fn selected_key_index(&self) -> Option<usize> { self.selected_key_index }

    pub fn from_slice(data: &[u8]) -> Option<Self> {
        let Ok(archived) = rkyv::access::<ArchivedUserPresenceResult, rkyv::rancor::Error>(data) else {
            return None;
        };
        rkyv::deserialize::<Self, rkyv::rancor::Error>(archived).ok()
    }

    pub fn serialize(&self) -> Vec<u8> { rkyv::to_bytes::<rkyv::rancor::Error>(self).unwrap().to_vec() }
}

/// The type of FIDO operation that completed.
#[derive(Debug, Clone, Copy, rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)]
pub enum OperationType {
    Registration,
    Authentication,
}

/// Options for the Operation Outcome navigation request.
///
/// Used to inform the user about the result of a registration or authentication operation.
/// This is a "fire and forget" notification - the FIDO server does not wait for user acknowledgment.
///
/// ```rust,ignore
/// # use gui_server_api::navigation::securitykeys::{OperationOutcomeOptions, OperationType};
/// let options = OperationOutcomeOptions::registration_success(0)
///     .with_rp_id("example.com".to_string())
///     .with_rp_name("Example Site".to_string());
/// ```
#[derive(Debug, Clone, rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)]
pub struct OperationOutcomeOptions {
    pub security_key_index: usize,
    pub operation: OperationType,
    pub success: bool,
    pub rp_id: Option<String>,
    pub rp_name: Option<String>,
    pub user_name: Option<String>,
    pub user_display_name: Option<String>,
    /// Optional error message when success is false.
    pub error_message: Option<String>,
}

impl OperationOutcomeOptions {
    fn new(security_key_index: usize, operation: OperationType, success: bool) -> Self {
        Self {
            security_key_index,
            operation,
            success,
            rp_id: None,
            rp_name: None,
            user_name: None,
            user_display_name: None,
            error_message: None,
        }
    }

    /// Creates options for a successful registration outcome.
    pub fn registration_success(security_key_index: usize) -> Self {
        Self::new(security_key_index, OperationType::Registration, true)
    }

    /// Creates options for a failed registration outcome.
    pub fn registration_failure(security_key_index: usize) -> Self {
        Self::new(security_key_index, OperationType::Registration, false)
    }

    /// Creates options for a successful authentication outcome.
    pub fn authentication_success(security_key_index: usize) -> Self {
        Self::new(security_key_index, OperationType::Authentication, true)
    }

    /// Creates options for a failed authentication outcome.
    pub fn authentication_failure(security_key_index: usize) -> Self {
        Self::new(security_key_index, OperationType::Authentication, false)
    }

    pub fn with_rp_id(self, rp_id: String) -> Self { Self { rp_id: Some(rp_id), ..self } }

    pub fn with_rp_name(self, rp_name: String) -> Self { Self { rp_name: Some(rp_name), ..self } }

    pub fn with_user_name(self, user_name: String) -> Self { Self { user_name: Some(user_name), ..self } }

    pub fn with_user_display_name(self, user_display_name: String) -> Self {
        Self { user_display_name: Some(user_display_name), ..self }
    }

    pub fn with_error_message(self, error_message: String) -> Self {
        Self { error_message: Some(error_message), ..self }
    }

    pub fn from_slice(data: &[u8]) -> Option<Self> {
        let Ok(archived) = rkyv::access::<ArchivedOperationOutcomeOptions, rkyv::rancor::Error>(data) else {
            return None;
        };
        rkyv::deserialize::<Self, rkyv::rancor::Error>(archived).ok()
    }

    pub fn serialize(&self) -> Vec<u8> { rkyv::to_bytes::<rkyv::rancor::Error>(self).unwrap().to_vec() }
}
