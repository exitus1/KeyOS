// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

/// Indicates successful response.
const CTAP1_ERR_SUCCESS: u8 = 0x00;
/// The command is not a valid CTAP command.
const CTAP1_ERR_INVALID_COMMAND: u8 = 0x01;
/// The command included an invalid parameter.
const CTAP1_ERR_INVALID_PARAMETER: u8 = 0x02;
/// Invalid message or item length.
const CTAP1_ERR_INVALID_LENGTH: u8 = 0x03;
/// Invalid message sequencing.
const CTAP1_ERR_INVALID_SEQ: u8 = 0x04;
/// Message timed out.
const CTAP1_ERR_TIMEOUT: u8 = 0x05;
/// Channel busy. Client SHOULD retry the request after a short delay. Note that the client MAY abort the
/// transaction if the command is no longer relevant.
const CTAP1_ERR_CHANNEL_BUSY: u8 = 0x06;
/// Command requires channel lock.
const CTAP1_ERR_LOCK_REQUIRED: u8 = 0x0A;
/// Command not allowed on this cid.
const CTAP1_ERR_INVALID_CHANNEL: u8 = 0x0B;
/// Invalid/unexpected CBOR error.
const CTAP2_ERR_CBOR_UNEXPECTED_TYPE: u8 = 0x11;
/// Error when parsing CBOR.
const CTAP2_ERR_INVALID_CBOR: u8 = 0x12;
/// Missing non-optional parameter.
const CTAP2_ERR_MISSING_PARAMETER: u8 = 0x14;
/// Limit for number of items exceeded.
const CTAP2_ERR_LIMIT_EXCEEDED: u8 = 0x15;
/// Fingerprint data base is full, e.g., during enrollment.
const CTAP2_ERR_FP_DATABASE_FULL: u8 = 0x17;
/// Large blob storage is full. (See § 6.10.3 Large, per-credential blobs.)
const CTAP2_ERR_LARGE_BLOB_STORAGE_FULL: u8 = 0x18;
/// Valid credential found in the exclude list.
const CTAP2_ERR_CREDENTIAL_EXCLUDED: u8 = 0x19;
/// Processing (Lengthy operation is in progress).
const CTAP2_ERR_PROCESSING: u8 = 0x21;
/// Credential not valid for the authenticator.
const CTAP2_ERR_INVALID_CREDENTIAL: u8 = 0x22;
/// Authentication is waiting for user interaction.
const CTAP2_ERR_USER_ACTION_PENDING: u8 = 0x23;
/// Processing, lengthy operation is in progress.
const CTAP2_ERR_OPERATION_PENDING: u8 = 0x24;
/// No request is pending.
const CTAP2_ERR_NO_OPERATIONS: u8 = 0x25;
/// Authenticator does not support requested algorithm.
const CTAP2_ERR_UNSUPPORTED_ALGORITHM: u8 = 0x26;
/// Not authorized for requested operation.
const CTAP2_ERR_OPERATION_DENIED: u8 = 0x27;
/// Internal key storage is full.
const CTAP2_ERR_KEY_STORE_FULL: u8 = 0x28;
/// Unsupported option.
const CTAP2_ERR_UNSUPPORTED_OPTION: u8 = 0x2B;
/// Not a valid option for current operation.
const CTAP2_ERR_INVALID_OPTION: u8 = 0x2C;
/// Pending keep alive was cancelled.
const CTAP2_ERR_KEEPALIVE_CANCEL: u8 = 0x2D;
/// No valid credentials provided.
const CTAP2_ERR_NO_CREDENTIALS: u8 = 0x2E;
/// A user action timeout occurred.
const CTAP2_ERR_USER_ACTION_TIMEOUT: u8 = 0x2F;
/// Continuation command, such as, authenticatorGetNextAssertion not allowed.
const CTAP2_ERR_NOT_ALLOWED: u8 = 0x30;
/// PIN Invalid.
const CTAP2_ERR_PIN_INVALID: u8 = 0x31;
/// PIN Blocked.
const CTAP2_ERR_PIN_BLOCKED: u8 = 0x32;
/// PIN authentication,pinUvAuthParam, verification failed.
const CTAP2_ERR_PIN_AUTH_INVALID: u8 = 0x33;
/// PIN authentication using pinUvAuthToken blocked. Requires power cycle to reset.
const CTAP2_ERR_PIN_AUTH_BLOCKED: u8 = 0x34;
/// No PIN has been set.
const CTAP2_ERR_PIN_NOT_SET: u8 = 0x35;
/// A pinUvAuthToken is required for the selected operation. See also the pinUvAuthToken option ID.
const CTAP2_ERR_PUAT_REQUIRED: u8 = 0x36;
/// PIN policy violation. Minimum PIN length or PIN complexity may trigger this error. The platform should
/// check the minimum PIN length in authenticatorGetInfo to discriminate between the causes of this error.
const CTAP2_ERR_PIN_POLICY_VIOLATION: u8 = 0x37;
/// Authenticator cannot handle this request due to memory constraints.
const CTAP2_ERR_REQUEST_TOO_LARGE: u8 = 0x39;
/// The current operation has timed out.
const CTAP2_ERR_ACTION_TIMEOUT: u8 = 0x3A;
/// User presence is required for the requested operation.
const CTAP2_ERR_UP_REQUIRED: u8 = 0x3B;
/// built-in user verification is disabled.
const CTAP2_ERR_UV_BLOCKED: u8 = 0x3C;
/// A checksum did not match.
const CTAP2_ERR_INTEGRITY_FAILURE: u8 = 0x3D;
/// The requested subcommand is either invalid or not implemented.
const CTAP2_ERR_INVALID_SUBCOMMAND: u8 = 0x3E;
/// built-in user verification unsuccessful. The platform SHOULD retry.
const CTAP2_ERR_UV_INVALID: u8 = 0x3F;
/// The permissions parameter contains an unauthorized permission.
const CTAP2_ERR_UNAUTHORIZED_PERMISSION: u8 = 0x40;
/// Other unspecified error.
const CTAP1_ERR_OTHER: u8 = 0x7F;
// /// CTAP 2 spec last error.
// const CTAP2_ERR_SPEC_LAST: u8 = 0xDF;
// /// Extension specific error.
// const CTAP2_ERR_EXTENSION_FIRST: u8 = 0xE0;
// /// Extension specific error.
// const CTAP2_ERR_EXTENSION_LAST: u8 = 0xEF;
// /// Vendor specific error.
// const CTAP2_ERR_VENDOR_FIRST: u8 = 0xF0;
// /// Vendor specific error.
// const CTAP2_ERR_VENDOR_LAST: u8 = 0xFF;

#[derive(Debug)]
pub enum Error {
    InvalidCommand,
    InvalidParamter,
    InvalidLength,
    InvalidSequence,
    Timeout,
    ChannelBusy,
    LockRequired,
    InvalidChannel,
    CborUnexpectedType,
    CborParsing,
    MissingParamter,
    LimitExceeded,
    FpDatabaseFull,
    LargeBlobStorageFull,
    CredentialExculded,
    Processing,
    InvalidCredential,
    UserActionPending,
    OperationPending,
    NoOperations,
    UnsupportedAlgorithm,
    OperationDenied,
    KeyStoreFull,
    UnsupportedOption,
    InvalidOption,
    KeepAliveCancel,
    NoCredentials,
    UserActionTimeout,
    NotAllowed,
    PinInvalid,
    PinBlocked,
    PinAuthInvalid,
    PinAuthBlocked,
    PinNotSet,
    PuatRequired,
    PinPolicyViolation,
    RequestTooLarge,
    ActionTimeout,
    UpRequired,
    UvBlocked,
    IntegrityFailure,
    InvalidSubcommand,
    UvInvalid,
    UnauthorizedPermission,
    Other,
    Signing,
}

impl From<crate::FidoError> for Error {
    fn from(_e: crate::FidoError) -> Error { Error::Other }
}

impl From<minicbor::decode::Error> for Error {
    fn from(e: minicbor::decode::Error) -> Error {
        if e.is_type_mismatch() {
            Error::CborUnexpectedType
        } else {
            Error::CborParsing
        }
    }
}

pub struct Status(u8);

impl From<Status> for u8 {
    fn from(s: Status) -> u8 { s.0 }
}

impl<T> From<&Result<T, Error>> for Status {
    fn from(r: &Result<T, Error>) -> Status {
        match r {
            Ok(_) => Status(CTAP1_ERR_SUCCESS),
            Err(Error::InvalidCommand) => Status(CTAP1_ERR_INVALID_COMMAND),
            Err(Error::InvalidParamter) => Status(CTAP1_ERR_INVALID_PARAMETER),
            Err(Error::InvalidLength) => Status(CTAP1_ERR_INVALID_LENGTH),
            Err(Error::InvalidSequence) => Status(CTAP1_ERR_INVALID_SEQ),
            Err(Error::Timeout) => Status(CTAP1_ERR_TIMEOUT),
            Err(Error::ChannelBusy) => Status(CTAP1_ERR_CHANNEL_BUSY),
            Err(Error::LockRequired) => Status(CTAP1_ERR_LOCK_REQUIRED),
            Err(Error::InvalidChannel) => Status(CTAP1_ERR_INVALID_CHANNEL),
            Err(Error::CborUnexpectedType) => Status(CTAP2_ERR_CBOR_UNEXPECTED_TYPE),
            Err(Error::CborParsing) => Status(CTAP2_ERR_INVALID_CBOR),
            Err(Error::MissingParamter) => Status(CTAP2_ERR_MISSING_PARAMETER),
            Err(Error::LimitExceeded) => Status(CTAP2_ERR_LIMIT_EXCEEDED),
            Err(Error::FpDatabaseFull) => Status(CTAP2_ERR_FP_DATABASE_FULL),
            Err(Error::LargeBlobStorageFull) => Status(CTAP2_ERR_LARGE_BLOB_STORAGE_FULL),
            Err(Error::CredentialExculded) => Status(CTAP2_ERR_CREDENTIAL_EXCLUDED),
            Err(Error::Processing) => Status(CTAP2_ERR_PROCESSING),
            Err(Error::InvalidCredential) => Status(CTAP2_ERR_INVALID_CREDENTIAL),
            Err(Error::UserActionPending) => Status(CTAP2_ERR_USER_ACTION_PENDING),
            Err(Error::OperationPending) => Status(CTAP2_ERR_OPERATION_PENDING),
            Err(Error::NoOperations) => Status(CTAP2_ERR_NO_OPERATIONS),
            Err(Error::UnsupportedAlgorithm) => Status(CTAP2_ERR_UNSUPPORTED_ALGORITHM),
            Err(Error::OperationDenied) => Status(CTAP2_ERR_OPERATION_DENIED),
            Err(Error::KeyStoreFull) => Status(CTAP2_ERR_KEY_STORE_FULL),
            Err(Error::UnsupportedOption) => Status(CTAP2_ERR_UNSUPPORTED_OPTION),
            Err(Error::InvalidOption) => Status(CTAP2_ERR_INVALID_OPTION),
            Err(Error::KeepAliveCancel) => Status(CTAP2_ERR_KEEPALIVE_CANCEL),
            Err(Error::NoCredentials) => Status(CTAP2_ERR_NO_CREDENTIALS),
            Err(Error::UserActionTimeout) => Status(CTAP2_ERR_USER_ACTION_TIMEOUT),
            Err(Error::NotAllowed) => Status(CTAP2_ERR_NOT_ALLOWED),
            Err(Error::PinInvalid) => Status(CTAP2_ERR_PIN_INVALID),
            Err(Error::PinBlocked) => Status(CTAP2_ERR_PIN_BLOCKED),
            Err(Error::PinAuthInvalid) => Status(CTAP2_ERR_PIN_AUTH_INVALID),
            Err(Error::PinAuthBlocked) => Status(CTAP2_ERR_PIN_AUTH_BLOCKED),
            Err(Error::PinNotSet) => Status(CTAP2_ERR_PIN_NOT_SET),
            Err(Error::PuatRequired) => Status(CTAP2_ERR_PUAT_REQUIRED),
            Err(Error::PinPolicyViolation) => Status(CTAP2_ERR_PIN_POLICY_VIOLATION),
            Err(Error::RequestTooLarge) => Status(CTAP2_ERR_REQUEST_TOO_LARGE),
            Err(Error::ActionTimeout) => Status(CTAP2_ERR_ACTION_TIMEOUT),
            Err(Error::UpRequired) => Status(CTAP2_ERR_UP_REQUIRED),
            Err(Error::UvBlocked) => Status(CTAP2_ERR_UV_BLOCKED),
            Err(Error::IntegrityFailure) => Status(CTAP2_ERR_INTEGRITY_FAILURE),
            Err(Error::InvalidSubcommand) => Status(CTAP2_ERR_INVALID_SUBCOMMAND),
            Err(Error::UvInvalid) => Status(CTAP2_ERR_UV_INVALID),
            Err(Error::UnauthorizedPermission) => Status(CTAP2_ERR_UNAUTHORIZED_PERMISSION),
            Err(Error::Other) => Status(CTAP1_ERR_OTHER),
            Err(Error::Signing) => Status(CTAP1_ERR_OTHER),
        }
    }
}

impl Status {
    pub fn to_vec(&self, payload: &[u8]) -> Vec<u8> {
        let mut v = vec![self.0];
        v.extend_from_slice(payload);
        v
    }
}
