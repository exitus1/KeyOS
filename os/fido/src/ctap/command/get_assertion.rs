// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{
    AuthData, ClientDataHash, Error, Extension, ExtensionOutput, ExtensionOutputs,
    PublicKeyCredentialDescriptor, PublicKeyCredentialUserEntity,
};

#[derive(Debug, Default)]
pub struct Options {
    /// user presence: Instructs the authenticator to require user consent to complete the operation.
    pub up: Option<bool>,

    /// user verification: If true, instructs the authenticator to require a user-verifying gesture in order
    /// to complete the request. Examples of such gestures are fingerprint scan or a PIN.
    /// NOTE: Use of this "uv" option key is deprecated in CTAP2.1. Instead, platforms SHOULD create a
    /// pinUvAuthParam by obtaining pinUvAuthToken via getPinUvAuthTokenUsingUvWithPermissions or
    /// getPinUvAuthTokenUsingPinWithPermissions, as appropriate.
    /// Platforms MUST NOT include the "uv" option key if the authenticator does not support built-in user
    /// verification.
    /// Platforms MUST NOT include both the "uv" option key and the pinUvAuthParam parameter in the same
    /// request.
    pub uv: Option<bool>,

    pub rk: Option<bool>,
}
impl<'b, C> minicbor::Decode<'b, C> for Options {
    fn decode(d: &mut minicbor::Decoder<'b>, _ctx: &mut C) -> Result<Self, minicbor::decode::Error> {
        let mut len = d.map()?;
        let mut opts = Options::default();
        while len.map(|l| l > 0).unwrap_or(false) {
            len = len.map(|l| l - 1);
            match d.str()? {
                "up" => opts.up = Some(d.bool()?),
                "uv" => opts.uv = Some(d.bool()?),
                _ => d.skip()?,
            }
        }
        Ok(opts)
    }
}

#[derive(Debug, minicbor::Decode)]
#[cbor(map)]
pub struct GetAssertionRequest {
    /// relying party identifier. See [WebAuthn].
    #[n(0x01)]
    pub rp_id: String,

    /// Hash of the serialized client data collected by the host. See [WebAuthn].
    #[n(0x02)]
    pub client_data_hash: ClientDataHash,

    /// An array of PublicKeyCredentialDescriptor structures, each denoting a credential, as specified in
    /// [WebAuthn]. A platform MUST NOT send an empty allowList—if it would be empty it MUST be omitted. If
    /// this parameter is present the authenticator MUST only generate an assertion using one of the denoted
    /// credentials.
    #[n(0x03)]
    pub allow_list: Option<Vec<PublicKeyCredentialDescriptor>>,

    #[n(0x04)]
    /// Parameters to influence authenticator operation. These parameters might be authenticator specific.
    pub extensions: Option<Vec<Extension>>,

    #[n(0x05)]
    /// Parameters to influence authenticator operation.
    pub options: Option<Options>,

    #[n(0x06)]
    /// Result of calling authenticate(pinUvAuthToken, clientDataHash)
    pub pin_uv_auth_param: Option<Vec<u8>>,

    #[n(0x07)]
    /// PIN/UV protocol version chosen by the platform
    pub pin_uv_auth_protocol: Option<usize>,
}
impl GetAssertionRequest {
    pub fn from_cbor(data: &[u8]) -> Result<GetAssertionRequest, Error> {
        Ok(minicbor::decode::<GetAssertionRequest>(&data)?)
    }
}

#[derive(Debug, minicbor::Encode)]
#[cbor(map)]
pub struct GetAssertionResponse {
    #[n(0x01)]
    /// PublicKeyCredentialDescriptor structure containing the credential identifier whose private key was
    /// used to generate the assertion.
    pub credential: Option<PublicKeyCredentialDescriptor>,

    #[n(0x02)]
    /// The signed-over contextual bindings made by the authenticator, as specified in [WebAuthn].
    pub auth_data: AuthData,

    #[cbor(n(3), with = "minicbor::bytes")]
    /// The assertion signature produced by the authenticator, as specified in [WebAuthn].
    pub signature: Vec<u8>,

    #[n(0x04)]
    /// PublicKeyCredentialUserEntity structure containing the user account information. User identifiable
    /// information (name, DisplayName, icon) MUST NOT be returned if user verification is not done by the
    /// authenticator.
    /// U2F Devices: For U2F devices, this parameter is not returned as this user information is not present
    /// for U2F credentials.
    /// FIDO Devices - server-side credentials: For server-side credentials on FIDO devices, this parameter
    /// is OPTIONAL as server-side credentials behave the same as U2F credentials where they are discovered
    /// given the user information on the RP. Authenticators MAY store user information inside the credential
    /// ID.
    /// FIDO Devices - discoverable credentials: For discoverable credentials on FIDO devices, at least user
    /// "id" is mandatory.
    /// For single account per RP case, authenticator returns "id" field to the platform which will be
    /// returned to the [WebAuthn] layer.
    /// For multiple accounts per RP case, where the authenticator does not have a display, authenticator
    /// returns "id" as well as other fields to the platform. Platform will use this information to show the
    /// account selection UX to the user and for the user selected account, it will ONLY return "id" back to
    /// the [WebAuthn] layer and discard other user details.
    pub user: Option<PublicKeyCredentialUserEntity>,

    #[n(0x05)]
    /// Total number of account credentials for the RP. Optional; defaults to one. This member is required
    /// when more than one credential is found for an RP, and the authenticator does not have a display or
    /// the UV & UP flags are false. Omitted when returned for the authenticatorGetNextAssertion method.
    pub number_of_credentials: Option<usize>,

    #[n(0x06)]
    /// Indicates that a credential was selected by the user via interaction directly with the authenticator,
    /// and thus the platform does not need to confirm the credential. Optional; defaults to false. MUST NOT
    /// be present in response to a request where an allowList was given, where numberOfCredentials is
    /// greater than one, nor in response to an authenticatorGetNextAssertion request.
    pub user_selected: Option<bool>,

    #[n(0x07)]
    /// The contents of the associated largeBlobKey if present for the asserted credential, and if
    /// largeBlobKey was true in the extensions input.
    pub large_blob_key: Option<Vec<u8>>,

    #[n(0x08)]
    ///  A map, keyed by extension identifiers, to unsigned outputs of extensions, if any. Authenticators
    /// SHOULD omit this field if no processed extensions define unsigned outputs. Clients MUST treat an
    /// empty map the same as an omitted field.
    pub unsigned_extension_outputs: Option<ExtensionOutputs>,
}
impl GetAssertionResponse {
    pub fn new(rp_id: &str) -> GetAssertionResponse {
        GetAssertionResponse {
            credential: None,
            auth_data: AuthData::new(rp_id),
            signature: Vec::new(),
            user: None,
            number_of_credentials: None,
            user_selected: None,
            large_blob_key: None,
            unsigned_extension_outputs: None,
        }
    }

    pub fn set_credential(&mut self, public_key: &[u8]) {
        self.credential = Some(PublicKeyCredentialDescriptor::with_public_key(public_key));
    }

    pub fn set_user(&mut self, mut user: PublicKeyCredentialUserEntity) {
        // User identifiable information (name, DisplayName, icon) inside the publicKeyCredentialUserEntity
        // MUST NOT be returned if user verification is not done by the authenticator.
        if self.auth_data.flags.uv {
            user.name = None;
            user.display_name = None;
        }
        self.user = Some(user);
    }

    pub fn set_number_of_credentials(&mut self, number_of_credentials: Option<usize>) {
        if let Some(number_of_credentials) = number_of_credentials {
            if number_of_credentials > 1 {
                self.number_of_credentials = Some(number_of_credentials);
            }
        }
    }

    pub fn add_unsigned_extention_output_data(&mut self, ext_output: ExtensionOutput) {
        if let Some(unsigned_extension_outputs) = &mut self.unsigned_extension_outputs {
            unsigned_extension_outputs.0.push(ext_output);
        } else {
            let mut unsigned_extension_outputs = ExtensionOutputs::new();
            unsigned_extension_outputs.0.push(ext_output);
            self.unsigned_extension_outputs = Some(unsigned_extension_outputs);
        }
        self.auth_data.flags.ed = true;
    }

    pub fn to_vec_cbor(&self) -> Vec<u8> { minicbor::to_vec(self).unwrap() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_get_assertion_request() {
        let data_get_assertion_request = [
            0xa4, 0x01, 0x6a, 0x67, 0x69, 0x74, 0x6c, 0x61, 0x62, 0x2e, 0x63, 0x6f, 0x6d, 0x02, 0x58, 0x20,
            0xc2, 0x95, 0x2d, 0x58, 0x04, 0xc6, 0xdb, 0xb5, 0xae, 0x9f, 0x07, 0xef, 0xea, 0xd1, 0x7f, 0x1c,
            0xda, 0xae, 0xfa, 0x28, 0x24, 0x08, 0x90, 0x23, 0xf7, 0xac, 0x7e, 0x21, 0x7b, 0x0c, 0xec, 0x73,
            0x03, 0x81, 0xa2, 0x62, 0x69, 0x64, 0x58, 0x40, 0x57, 0x02, 0x7b, 0xf4, 0x94, 0x97, 0xb6, 0x84,
            0x2a, 0x9b, 0x65, 0x8e, 0xe8, 0x9f, 0x6e, 0x71, 0x8e, 0x2b, 0xac, 0x63, 0xe7, 0xd7, 0xc8, 0xc7,
            0x3f, 0x85, 0xe3, 0x20, 0xcc, 0xca, 0xd6, 0xa1, 0x38, 0x50, 0x56, 0x06, 0x20, 0xd1, 0x60, 0x87,
            0x4d, 0x46, 0x93, 0xaf, 0xb1, 0x23, 0xfd, 0x0b, 0x2f, 0x87, 0xe6, 0xf7, 0x6e, 0xd3, 0xad, 0x95,
            0xc1, 0x2d, 0x2d, 0x72, 0x4b, 0x60, 0x4c, 0x86, 0x64, 0x74, 0x79, 0x70, 0x65, 0x6a, 0x70, 0x75,
            0x62, 0x6c, 0x69, 0x63, 0x2d, 0x6b, 0x65, 0x79, 0x05, 0xa1, 0x62, 0x75, 0x70, 0xf4,
        ];
        let get_assertion_request =
            minicbor::decode::<GetAssertionRequest>(&data_get_assertion_request).unwrap();
        println!("{:02x?}", get_assertion_request);
    }

    #[test]
    fn encode_get_assertion_response() {
        let mut get_assertion_response = GetAssertionResponse::new("gitlab.com");
        get_assertion_response.set_credential(&[
            0x57, 0x02, 0x7b, 0xf4, 0x94, 0x97, 0xb6, 0x84, 0x2a, 0x9b, 0x65, 0x8e, 0xe8, 0x9f, 0x6e, 0x71,
            0x8e, 0x2b, 0xac, 0x63, 0xe7, 0xd7, 0xc8, 0xc7, 0x3f, 0x85, 0xe3, 0x20, 0xcc, 0xca, 0xd6, 0xa1,
            0x38, 0x50, 0x56, 0x06, 0x20, 0xd1, 0x60, 0x87, 0x4d, 0x46, 0x93, 0xaf, 0xb1, 0x23, 0xfd, 0x0b,
            0x2f, 0x87, 0xe6, 0xf7, 0x6e, 0xd3, 0xad, 0x95, 0xc1, 0x2d, 0x2d, 0x72, 0x4b, 0x60, 0x4c, 0x86,
        ]);
        get_assertion_response.auth_data.sign_count = 2;
        get_assertion_response.signature = [
            0x30, 0x46, 0x02, 0x21, 0x00, 0xdd, 0xc8, 0xc0, 0x2e, 0xb7, 0x57, 0x77, 0x50, 0x96, 0x49, 0x20,
            0x3c, 0x96, 0xad, 0x94, 0x48, 0xea, 0xea, 0xd5, 0xa4, 0x53, 0xcc, 0xac, 0x83, 0x5e, 0x7b, 0xb8,
            0xff, 0xb1, 0x6f, 0xaf, 0x73, 0x02, 0x21, 0x00, 0x94, 0x8b, 0x77, 0xb9, 0x17, 0xc7, 0x78, 0xdd,
            0x17, 0x99, 0x70, 0xb6, 0xc2, 0x92, 0xc0, 0xfd, 0x40, 0xf1, 0x47, 0x8d, 0x3c, 0xdd, 0x03, 0xc9,
            0x36, 0x1d, 0x20, 0xab, 0x53, 0x68, 0x99, 0x5b,
        ]
        .to_vec();
        let data_get_assertion_response = vec![
            0xa3, 0x01, 0xa2, 0x62, 0x69, 0x64, 0x58, 0x40, 0x57, 0x02, 0x7b, 0xf4, 0x94, 0x97, 0xb6, 0x84,
            0x2a, 0x9b, 0x65, 0x8e, 0xe8, 0x9f, 0x6e, 0x71, 0x8e, 0x2b, 0xac, 0x63, 0xe7, 0xd7, 0xc8, 0xc7,
            0x3f, 0x85, 0xe3, 0x20, 0xcc, 0xca, 0xd6, 0xa1, 0x38, 0x50, 0x56, 0x06, 0x20, 0xd1, 0x60, 0x87,
            0x4d, 0x46, 0x93, 0xaf, 0xb1, 0x23, 0xfd, 0x0b, 0x2f, 0x87, 0xe6, 0xf7, 0x6e, 0xd3, 0xad, 0x95,
            0xc1, 0x2d, 0x2d, 0x72, 0x4b, 0x60, 0x4c, 0x86, 0x64, 0x74, 0x79, 0x70, 0x65, 0x6a, 0x70, 0x75,
            0x62, 0x6c, 0x69, 0x63, 0x2d, 0x6b, 0x65, 0x79, 0x02, 0x58, 0x25, 0x20, 0xa8, 0x3b, 0x42, 0xcd,
            0x54, 0x0c, 0x2f, 0xcd, 0xee, 0x61, 0xe4, 0xf1, 0x47, 0x93, 0xed, 0xe7, 0x74, 0xfa, 0x82, 0x58,
            0x83, 0x56, 0x83, 0xa7, 0xc8, 0xe8, 0x85, 0xa8, 0xc2, 0x70, 0xc2, 0x00, 0x00, 0x00, 0x00, 0x02,
            0x03, 0x58, 0x48, 0x30, 0x46, 0x02, 0x21, 0x00, 0xdd, 0xc8, 0xc0, 0x2e, 0xb7, 0x57, 0x77, 0x50,
            0x96, 0x49, 0x20, 0x3c, 0x96, 0xad, 0x94, 0x48, 0xea, 0xea, 0xd5, 0xa4, 0x53, 0xcc, 0xac, 0x83,
            0x5e, 0x7b, 0xb8, 0xff, 0xb1, 0x6f, 0xaf, 0x73, 0x02, 0x21, 0x00, 0x94, 0x8b, 0x77, 0xb9, 0x17,
            0xc7, 0x78, 0xdd, 0x17, 0x99, 0x70, 0xb6, 0xc2, 0x92, 0xc0, 0xfd, 0x40, 0xf1, 0x47, 0x8d, 0x3c,
            0xdd, 0x03, 0xc9, 0x36, 0x1d, 0x20, 0xab, 0x53, 0x68, 0x99, 0x5b,
        ];
        let cbor_get_assertion_response = get_assertion_response.to_vec_cbor();
        println!("cbor_get_assertion_response: {:02x?}", cbor_get_assertion_response);
        println!("data_get_assertion_response: {:02x?}", data_get_assertion_response);
        assert_eq!(data_get_assertion_response, cbor_get_assertion_response);
    }
}
