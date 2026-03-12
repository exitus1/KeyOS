// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{Extension, PublicKeyCredentialParameters};

#[derive(Debug)]
pub struct Options {
    /// platform device: Indicates that the device is attached to the client and therefore can’t be removed
    /// and used on another client.
    pub plat: Option<bool>,

    /// Specifies whether this authenticator can create discoverable credentials, and therefore can satisfy
    /// authenticatorGetAssertion requests with the allowList parameter omitted.
    pub rk: bool,

    /// ClientPIN feature support:
    /// - If present and set to true, it indicates that the device is capable of accepting a PIN from the
    ///   client and PIN has been set.
    /// - If present and set to false, it indicates that the device is capable of accepting a PIN from the
    ///   client and PIN has not been set yet.
    /// - If absent, it indicates that the device is not capable of accepting a PIN from the client.
    /// ClientPIN is one of the overall ways to do user verification, although ClientPIN is not considered a
    /// built-in user verification method.
    pub client_pin: Option<bool>,

    /// user presence: Indicates that the device is capable of testing user presence.
    pub up: bool,

    /// user verification: Indicates that the authenticator supports a built-in user verification method. For
    /// example, devices with UI, biometrics fall into this category.
    /// - If present and set to true, it indicates that the device is capable of built-in user verification
    ///   and its user verification feature is presently configured.
    /// - If present and set to false, it indicates that the authenticator is capable of built-in user
    ///   verification and its user verification feature is not presently configured. For example, an
    ///   authenticator featuring a built-in biometric user verification feature that is not presently
    ///   configured will return this "uv" option id set to false.
    /// - If absent, it indicates that the authenticator does not have a built-in user verification
    ///   capability.
    /// A device that can only do Client PIN will not return the "uv" option id.
    /// If a device is capable of both built-in user verification and Client PIN, the authenticator will
    /// return both the "uv" and the "clientPin" option ids.
    pub uv: Option<bool>,

    /// If pinUvAuthToken is:
    /// - present and set to true: if the clientPin option id is present and set to true, then the
    ///   authenticator supports authenticatorClientPIN’s getPinUvAuthTokenUsingPinWithPermissions
    ///   subcommand. If the uv option id is present and set to true, then the authenticator supports
    ///   authenticatorClientPIN’s getPinUvAuthTokenUsingUvWithPermissions subcommand.
    /// - present and set to false, or absent: the authenticator does not support authenticatorClientPIN’s
    ///   getPinUvAuthTokenUsingPinWithPermissions and getPinUvAuthTokenUsingUvWithPermissions subcommands.
    pub pin_uv_auth_token: Option<bool>,

    /// If this noMcGaPermissionsWithClientPin is:
    /// - present and set to true: A pinUvAuthToken obtained via getPinUvAuthTokenUsingPinWithPermissions (or
    ///   getPinToken) cannot be used for authenticatorMakeCredential or authenticatorGetAssertion commands,
    ///   because it will lack the necessary mc and ga permissions. In this situation, platforms SHOULD NOT
    ///   attempt to use getPinUvAuthTokenUsingPinWithPermissions if using
    ///   getPinUvAuthTokenUsingUvWithPermissions fails.
    /// - present and set to false, or absent: A pinUvAuthToken obtained via
    ///   getPinUvAuthTokenUsingPinWithPermissions (or getPinToken) can be used for
    ///   authenticatorMakeCredential or authenticatorGetAssertion commands.
    /// Note: noMcGaPermissionsWithClientPin MUST only be present if the clientPin option ID is present.
    pub no_mc_ga_permissions_with_client_pin: Option<bool>,

    /// If largeBlobs is:
    /// - present and set to true: the authenticator supports the authenticatorLargeBlobs command.
    /// - present and set to false, or absent: the authenticatorLargeBlobs command is NOT supported.
    /// This option MUST NOT be set to true if the largeBlob extension is supported instead.
    pub large_blobs: Option<bool>,

    /// Enterprise Attestation feature support:
    /// If ep is:
    /// - Present and set to true: The authenticator is enterprise attestation capable, and enterprise
    ///   attestation is enabled.
    /// - Present and set to false: The authenticator is enterprise attestation capable, and enterprise
    ///   attestation is disabled.
    /// - Absent: The Enterprise Attestation feature is NOT supported.
    pub ep: Option<bool>,

    /// If bioEnroll is:
    /// - present and set to true: the authenticator supports the authenticatorBioEnrollment commands, and
    ///   has at least one bio enrollment presently provisioned.
    /// - present and set to false: the authenticator supports the authenticatorBioEnrollment commands, and
    ///   does not yet have any bio enrollments provisioned.
    /// - absent: the authenticatorBioEnrollment commands are NOT supported.
    pub bio_enroll: Option<bool>,

    /// "FIDO_2_1_PRE" Prototype Bio enrollment support:
    /// If userVerificationMgmtPreview is:
    /// - present and set to true: the authenticator supports the Prototype authenticatorBioEnrollment (0x40)
    ///   commands, and has at least one bio enrollment presently provisioned.
    /// - present and set to false: the authenticator supports the Prototype authenticatorBioEnrollment
    ///   (0x40) commands, and does not yet have any bio enrollments provisioned.
    /// - absent: the Prototype authenticatorBioEnrollment (0x40) commands are not supported.
    pub user_verification_mgmt_preview: Option<bool>,

    /// getPinUvAuthTokenUsingUvWithPermissions support for requesting the be permission:
    /// This option ID MUST only be present if bioEnroll is also present.
    /// If uvBioEnroll is:
    /// - present and set to true: requesting the be permission when invoking
    ///   getPinUvAuthTokenUsingUvWithPermissions is supported.
    /// - present and set to false, or absent: requesting the be permission when invoking
    ///   getPinUvAuthTokenUsingUvWithPermissions is NOT supported.
    pub uv_bio_enroll: Option<bool>,

    /// authenticatorConfig command support:
    /// If authnrCfg is:
    /// - present and set to true: the authenticatorConfig command is supported.
    /// - present and set to false, or absent: the authenticatorConfig command is NOT supported.
    pub authnr_cfg: Option<bool>,

    /// getPinUvAuthTokenUsingUvWithPermissions support for requesting the acfg permission:
    /// This option ID MUST only be present if authnrCfg is also present.
    /// If uvAcfg is:
    /// - present and set to true: requesting the acfg permission when invoking
    ///   getPinUvAuthTokenUsingUvWithPermissions is supported.
    /// - present and set to false, or absent: requesting the acfg permission when invoking
    ///   getPinUvAuthTokenUsingUvWithPermissions is NOT supported.
    pub uv_acfg: Option<bool>,

    /// Credential management support:
    /// If credMgmt is:
    /// - present and set to true: the authenticatorCredentialManagement command is supported.
    /// - present and set to false, or absent: the authenticatorCredentialManagement command is NOT
    ///   supported.
    pub cred_mgmt: Option<bool>,

    /// Credential management Read Only support:
    /// If perCredMgmtRO is:
    /// - present and set to true: requesting the pcmr permission when invoking
    ///   getPinUvAuthTokenUsingUvWithPermissions or getPinUvAuthTokenUsingPinWithPermissions is supported.
    /// - present and set to false, or absent: requesting the pcmr permission when invoking
    ///   getPinUvAuthTokenUsingUvWithPermissions or getPinUvAuthTokenUsingPinWithPermissions is NOT
    ///   supported.
    pub per_cred_mgmt_ro: Option<bool>,

    /// "FIDO_2_1_PRE" Prototype Credential management support:
    /// If credentialMgmtPreview is:
    /// - present and set to true: the Prototype authenticatorCredentialManagement (0x41) command is
    ///   supported.
    /// - present and set to false, or absent: the Prototype authenticatorCredentialManagement (0x41) command
    ///   is NOT supported.
    pub credential_mgmt_preview: Option<bool>,

    /// Support for the Set Minimum PIN Length feature.
    /// If setMinPINLength is:
    /// - present and set to true: the setMinPINLength subcommand is supported.
    /// - present and set to false, or absent: the setMinPINLength subcommand is NOT supported.
    /// Note: setMinPINLength MUST only be present if the clientPin option ID is present.
    pub set_min_pin_length: Option<bool>,

    /// Support for making non-discoverable credentials without requiring User Verification.
    /// If makeCredUvNotRqd is:
    /// - present and set to true: the authenticator allows creation of non-discoverable credentials without
    ///   requiring any form of user verification, if the platform requests this behaviour.
    /// - present and set to false, or absent: the authenticator requires some form of user verification for
    ///   creating non-discoverable credentials, regardless of the parameters the platform supplies for the
    ///   authenticatorMakeCredential command.
    /// Authenticators SHOULD include this option with the value true.
    pub make_cred_uv_not_rqd: Option<bool>,

    /// Support for the Always Require User Verification feature:
    /// If alwaysUv is
    /// - present and set to true: the authenticator supports the Always Require User Verification feature
    ///   and it is enabled.
    /// - present and set to false: the authenticator supports the Always Require User Verification feature
    ///   but it is disabled.
    /// - absent: the authenticator does not support the Always Require User Verification feature.
    /// NOTE: If the alwaysUv option ID is present and true the authenticator MUST set the value of
    /// makeCredUvNotRqd to false.
    pub always_uv: Option<bool>,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            plat: None,
            rk: false,
            client_pin: None,
            up: true,
            uv: None,
            pin_uv_auth_token: None,
            no_mc_ga_permissions_with_client_pin: None,
            large_blobs: None,
            ep: None,
            bio_enroll: None,
            user_verification_mgmt_preview: None,
            uv_bio_enroll: None,
            authnr_cfg: None,
            uv_acfg: None,
            cred_mgmt: None,
            per_cred_mgmt_ro: None,
            credential_mgmt_preview: None,
            set_min_pin_length: None,
            make_cred_uv_not_rqd: None,
            always_uv: None,
        }
    }
}
impl Options {
    pub fn prime() -> Self { Self { up: true, ..Default::default() } }
}
impl<C> minicbor::Encode<C> for Options {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        _ctx: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        let mut map_len = 2;
        for op in [
            self.plat,
            self.client_pin,
            self.uv,
            self.pin_uv_auth_token,
            self.no_mc_ga_permissions_with_client_pin,
            self.large_blobs,
            self.ep,
            self.bio_enroll,
            self.user_verification_mgmt_preview,
            self.uv_bio_enroll,
            self.authnr_cfg,
            self.uv_acfg,
            self.cred_mgmt,
            self.per_cred_mgmt_ro,
            self.credential_mgmt_preview,
            self.set_min_pin_length,
            self.make_cred_uv_not_rqd,
            self.always_uv,
        ] {
            if op.is_some() {
                map_len += 1;
            }
        }
        e.map(map_len)?;
        if let Some(ep) = self.ep {
            e.str("ep")?.bool(ep)?;
        }
        e.str("rk")?.bool(self.rk)?;
        e.str("up")?.bool(self.up)?;
        if let Some(uv) = self.uv {
            e.str("uv")?.bool(uv)?;
        }
        if let Some(plat) = self.plat {
            e.str("plat")?.bool(plat)?;
        }
        if let Some(uv_acfg) = self.uv_acfg {
            e.str("uvAcfg")?.bool(uv_acfg)?;
        }
        if let Some(always_uv) = self.always_uv {
            e.str("alwaysUv")?.bool(always_uv)?;
        }
        if let Some(cred_mgmt) = self.cred_mgmt {
            e.str("credMgmt")?.bool(cred_mgmt)?;
        }
        if let Some(authnr_cfg) = self.authnr_cfg {
            e.str("authnrCfg")?.bool(authnr_cfg)?;
        }
        if let Some(bio_enroll) = self.bio_enroll {
            e.str("bioEnroll")?.bool(bio_enroll)?;
        }
        if let Some(client_pin) = self.client_pin {
            e.str("clientPin")?.bool(client_pin)?;
        }
        if let Some(large_blobs) = self.large_blobs {
            e.str("largeBlobs")?.bool(large_blobs)?;
        }
        if let Some(uv_bio_enroll) = self.uv_bio_enroll {
            e.str("uvBioEnroll")?.bool(uv_bio_enroll)?;
        }
        if let Some(per_cred_mgmt_ro) = self.per_cred_mgmt_ro {
            e.str("perCredMgmtRO")?.bool(per_cred_mgmt_ro)?;
        }
        if let Some(pin_uv_auth_token) = self.pin_uv_auth_token {
            e.str("pinUvAuthToken")?.bool(pin_uv_auth_token)?;
        }
        if let Some(set_min_pin_length) = self.set_min_pin_length {
            e.str("setMinPINLength")?.bool(set_min_pin_length)?;
        }
        if let Some(make_cred_uv_not_rqd) = self.make_cred_uv_not_rqd {
            e.str("makeCredUvNotRqd")?.bool(make_cred_uv_not_rqd)?;
        }
        if let Some(credential_mgmt_preview) = self.credential_mgmt_preview {
            e.str("credentialMgmtPreview")?.bool(credential_mgmt_preview)?;
        }
        if let Some(user_verification_mgmt_preview) = self.user_verification_mgmt_preview {
            e.str("userVerificationMgmtPreview")?.bool(user_verification_mgmt_preview)?;
        }
        if let Some(no_mc_ga_permissions_with_client_pin) = self.no_mc_ga_permissions_with_client_pin {
            e.str("noMcGaPermissionsWithClientPin")?.bool(no_mc_ga_permissions_with_client_pin)?;
        }
        e.ok()
    }
}

#[derive(Debug, Default)]
pub struct Certifications {
    pub fips_cmpv2: Option<u8>,
    pub fips_cmpv3: Option<u8>,
    pub fips_cmpv2_phy: Option<u8>,
    pub fips_cmpv3_phy: Option<u8>,
    pub cc_eal: Option<u8>,
    pub fido: Option<u8>,
}
impl<C> minicbor::Encode<C> for Certifications {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        _ctx: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        let mut map_len = 2;
        for op in [
            self.fips_cmpv2,
            self.fips_cmpv3,
            self.fips_cmpv2_phy,
            self.fips_cmpv3_phy,
            self.cc_eal,
            self.fido,
        ] {
            if op.is_some() {
                map_len += 1;
            }
        }
        e.map(map_len)?;
        if let Some(fido) = self.fido {
            e.str("FIDO")?.u8(fido)?;
        }
        if let Some(cc_eal) = self.cc_eal {
            e.str("CC-EAL")?.u8(cc_eal)?;
        }
        if let Some(fips_cmpv2) = self.fips_cmpv2 {
            e.str("FIPS-CMVP-2")?.u8(fips_cmpv2)?;
        }
        if let Some(fips_cmpv3) = self.fips_cmpv3 {
            e.str("FIPS-CMVP-3")?.u8(fips_cmpv3)?;
        }
        if let Some(fips_cmpv2_phy) = self.fips_cmpv2_phy {
            e.str("FIPS-CMVP-2-PHY")?.u8(fips_cmpv2_phy)?;
        }
        if let Some(fips_cmpv3_phy) = self.fips_cmpv3_phy {
            e.str("FIPS-CMVP-3-PHY")?.u8(fips_cmpv3_phy)?;
        }
        e.ok()
    }
}

#[derive(Debug, Default, minicbor::Encode)]
#[cbor(map)]
pub struct GetInfoResponse {
    #[n(0x01)]
    /// List of supported versions. Supported versions are: "FIDO_2_1" for CTAP2.1 / FIDO2 / Web
    /// Authentication authenticators, "FIDO_2_0" for CTAP2.0 / FIDO2 / Web Authentication authenticators,
    /// "FIDO_2_1_PRE" for CTAP2.1 Preview features and "U2F_V2" for CTAP1/U2F authenticators.
    pub versions: Vec<String>,

    #[n(0x02)]
    /// List of supported extensions.
    pub extensions: Option<Vec<Extension>>,

    #[cbor(n(0x03), with = "minicbor::bytes")]
    /// The claimed AAGUID. 16 bytes in length and encoded the same as MakeCredential AuthenticatorData, as
    /// specified in [WebAuthn].
    pub aaguid: [u8; 16],

    #[n(0x04)]
    /// List of supported options.
    pub options: Option<Options>,

    #[n(0x05)]
    /// Maximum message size supported by the authenticator.
    pub max_msg_size: Option<usize>,

    #[n(0x06)]
    /// List of supported PIN/UV auth protocols in order of decreasing authenticator preference. MUST NOT
    /// contain duplicate values nor be empty if present.
    pub pin_uv_auth_protocols: Option<Vec<u8>>,

    #[n(0x07)]
    /// Maximum number of credentials supported in credentialID list at a time by the authenticator. MUST be
    /// greater than zero if present.
    pub max_credential_count_in_list: Option<usize>,

    #[n(0x08)]
    /// Maximum Credential ID Length supported by the authenticator. MUST be greater than zero if present.
    pub max_credential_id_length: Option<usize>,

    #[n(0x09)]
    /// List of supported transports. Values are taken from the AuthenticatorTransport enum in [WebAuthn].
    /// The list MUST NOT include duplicate values nor be empty if present. Platforms MUST tolerate unknown
    /// values.
    pub transports: Option<Vec<String>>,

    #[n(0x0A)]
    /// List of supported algorithms for credential generation, as specified in [WebAuthn]. The array is
    /// ordered from most preferred to least preferred and MUST NOT include duplicate entries nor be empty if
    /// present. PublicKeyCredentialParameters' algorithm identifiers are values that SHOULD be registered in
    /// the IANA COSE Algorithms registry [IANA-COSE-ALGS-REG].
    pub algorithms: Option<Vec<PublicKeyCredentialParameters>>,

    #[n(0x0B)]
    /// The maximum size, in bytes, of the serialized large-blob array that this authenticator can store. If
    /// the authenticatorLargeBlobs command is supported, this MUST be specified. Otherwise it MUST NOT be.
    /// If specified, the value MUST be ≥ 1024. Thus, 1024 bytes is the least amount of storage an
    /// authenticator must make available for per-credential serialized large-blob arrays if it supports the
    /// large, per-credential blobs feature. This value is not specified and not pertinent if the
    /// authenticator implements the largeBlob extension.
    pub max_serialized_large_blob_array: Option<usize>,

    #[n(0x0C)]
    /// If this member is:
    /// - present and set to true: getPinToken and getPinUvAuthTokenUsingPinWithPermissions will return
    ///   errors until after a successful PIN Change.
    /// - present and set to false, or absent: no PIN Change is required.
    pub force_pin_change: Option<bool>,

    #[n(0x0D)]
    /// This specifies the current minimum PIN length, in Unicode code points, the authenticator enforces for
    /// ClientPIN. This is applicable for ClientPIN only: the minPINLength member MUST be absent if the
    /// clientPin option ID is absent; it MUST be present if the authenticator supports
    /// authenticatorClientPIN.
    /// The default pre-configured minimum PIN length is at least 4 Unicode code points. Authenticators MAY
    /// have a pre-configured default minPINLength of more than 4 code points in certain offerings. On reset,
    /// minPINLength reverts to its original pre-configured value. Authenticators MAY also have a
    /// pre-configured list of RP IDs authorized to receive the current minimum PIN length value via the
    /// minPinLength extension.
    pub min_pin_length: Option<usize>,

    #[n(0x0E)]
    /// Indicates the firmware version of the authenticator model identified by AAGUID. Whenever releasing
    /// any code change to the authenticator firmware, authenticator MUST increase the version.
    pub firmware_version: Option<usize>,

    #[n(0x0F)]
    /// Maximum credBlob length in bytes supported by the authenticator. Must be present if, and only if,
    /// credBlob is included in the supported extensions list. If present, this value MUST be at least 32
    /// bytes.
    pub max_cred_blob_length: Option<usize>,

    #[n(0x10)]
    /// This specifies the max number of RP ID that the authenticator will accept via setMinPINLength
    /// subcommand. The platform MUST NOT send more than this number of RP ID to the setMinPINLength
    /// subcommand. This is in addition to pre-configured list authenticator may have. If the authenticator
    /// does not support adding additional RP IDs, its value is 0. This MUST ONLY be present if, and only if,
    /// the authenticator supports the setMinPINLength subcommand.
    pub max_rpids_for_set_min_pin_length: Option<usize>,

    #[n(0x11)]
    /// This specifies the preferred number of invocations of the getPinUvAuthTokenUsingUvWithPermissions
    /// subCommand the platform may attempt before falling back to the
    /// getPinUvAuthTokenUsingPinWithPermissions subCommand or displaying an error. MUST be greater than
    /// zero. If the value is 1 then all uvRetries are internal and the platform MUST only invoke the
    /// getPinUvAuthTokenUsingUvWithPermissions subCommand a single time. If the value is > 1 the
    /// authenticator MUST only decrement uvRetries by 1 for each iteration.
    pub preferred_platform_uv_attempts: Option<usize>,

    #[n(0x12)]
    /// This specifies the user verification modality supported by the authenticator via
    /// authenticatorClientPIN’s getPinUvAuthTokenUsingUvWithPermissions subcommand. This is a hint to help
    /// the platform construct user dialogs. The values are defined in [FIDORegistry] Section 3.1 User
    /// Verification Methods. Combining multiple bit-flags from the [FIDORegistry] is allowed. If clientPin
    /// is supported it MUST NOT be included in the bit-flags, as clientPIN is not a built-in user
    /// verification method.
    pub uv_modality: Option<usize>,

    #[n(0x13)]
    /// This specifies a list of authenticator certifications.
    pub certifications: Option<Certifications>,

    #[n(0x14)]
    /// If this member is present it indicates the estimated number of additional discoverable credentials
    /// that can be stored. If this value is zero then platforms SHOULD create non-discoverable
    /// credentials if possible.
    /// This estimate SHOULD be based on the assumption that all future discoverable credentials will have
    /// maximally-sized fields and SHOULD be zero whenever an attempt to create a discoverable credential may
    /// fail due to lack of space, even if it’s possible that some specific request might succeed. For
    /// example, a specific request might include fields that are smaller than the maximum possible size and
    /// thus succeed, but this value should be zero if a request with maximum-sized fields would fail. Also,
    /// a specific request might have an rp.id and user.id that match an existing discoverable credential and
    /// thus overwrite it, but this value should be set assuming that will not happen.
    pub remaining_discoverable_credentials: Option<usize>,

    #[n(0x15)]
    /// If present the authenticator supports the authenticatorConfig vendorPrototype subcommand, and its
    /// value is a list of authenticatorConfig vendorCommandId values supported, which MAY be empty.
    pub vendor_prototype_config_commands: Option<usize>,

    #[n(0x16)]
    /// List of supported attestation formats. Authenticators that support multiple attestation formats, not
    /// counting "none", MUST set this field. Otherwise it is optional.
    /// Values are taken from the "WebAuthn Attestation Statement Format Identifiers" registry
    /// [IANA-WebAuthn-Registries] established by [RFC8809]. The list MUST NOT include duplicate values nor
    /// be empty if present. Platforms MUST tolerate unknown values. Support for "none" attestation is
    /// implied and MUST be omitted.
    pub attestation_formats: Option<Vec<String>>,

    #[n(0x17)]
    /// If present the number of internal User Verification operations since the last pin entry including all
    /// failed attempts. This allows the platform to periodically prompt the user for PIN on a biometric
    /// device so they don’t forget the PIN. This is optional platform behavior and the interval is at the
    /// discretion of the platform.
    pub uv_count_since_last_pin_entry: Option<usize>,

    #[n(0x18)]
    /// If present the authenticator requires a 10 second touch for reset.
    pub long_touch_for_reset: Option<bool>,

    #[n(0x19)]
    /// The value is a byte value containing iv || ct. Where ct is the AES-128-CBC encryption of (128-bit
    /// device identifier) using HKDF-SHA-256(salt = 32 zero bytes, IKM = persistentPinUvAuthToken, L = 16,
    /// info = "encIdentifier"). The encryption iv must be regenerated for each output of getInfo.
    pub enc_identifier: Option<Vec<u8>>,

    #[n(0x1A)]
    /// List of transports that support the reset command. Values are taken from the AuthenticatorTransport
    /// enum in [WebAuthn]. The list MUST NOT include duplicate values nor be empty if present. Platforms
    /// MUST tolerate unknown values.
    pub transports_for_reset: Option<Vec<String>>,

    #[n(0x1B)]
    /// If present, whether the authenticator is enforcing an additional current PIN complexity policy beyond
    /// minPINLength. PIN complexity policies for authenticators are listed in the FIDO MDS. The
    /// authenticator may have a pre-configured PIN complexity policy value that is applied after a reset.
    pub pin_complexity_policy: Option<bool>,

    #[n(0x1C)]
    /// If present, a URL that the platform can use to provide the user more information about the enforced
    /// PIN policy.
    pub pin_complexity_policy_url: Option<Vec<u8>>,

    #[n(0x1D)]
    /// This specifies the maximum PIN length, in Unicode code points, the authenticator enforces for
    /// ClientPIN. An authenticator setting this value still MUST restrict the PIN to be represented in 63 or
    /// fewer bytes. This is applicable for ClientPIN only: the maxPINLength member MUST be absent if the
    /// clientPin option ID is not supported. If the authenticator supports authenticatorClientPIN and the
    /// maxPINLength member is absent, the effective default maxPINLength is 63 code points.
    /// If specified, the maximum PIN length must be at least 8 Unicode code points. Authenticators MAY have
    /// a pre-configured default maxPINLength of less than 63 code points in certain offerings. UTF-8 encoded
    /// code points may be represented by 1-4 octets, however the maximum length passed in the PIN parameter
    /// MUST always be less than 63 octets.
    pub max_pin_length: Option<usize>,
}
impl GetInfoResponse {
    pub fn prime(aaguid: [u8; 16]) -> Self {
        Self {
            versions: vec![
                "U2F_V2".to_string(), // CTAP1/U2F
            ],
            aaguid,
            ..Default::default()
        }
    }

    pub fn to_vec_cbor(&self) -> Vec<u8> { minicbor::to_vec(self).unwrap() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_options_defaults() {
        assert_eq!(Options::default().up, true);
    }

    #[test]
    fn test_get_info() {
        // From Yubikey5 NFC
        let get_info = GetInfoResponse {
            versions: vec![
                "U2F_V2".to_string(),       // CTAP1/U2F
                "FIDO_2_0".to_string(),     // CTAP2.0 / FIDO2 / Web Authentication
                "FIDO_2_1_PRE".to_string(), // CTAP2.1 Preview features
                "FIDO_2_1".to_string(),     // CTAP2.1 / FIDO2 / Web Authentication
            ],
            extensions: Some(vec![
                Extension::CredProtect,
                Extension::HmacSecret,
                Extension::LargeBlobKey,
                Extension::CredBlob,
                Extension::MinPinLength,
            ]),
            aaguid: [
                0xa2, 0x53, 0x42, 0xc0, 0x3c, 0xdc, 0x44, 0x14, 0x8e, 0x46, 0xf4, 0x80, 0x7f, 0xca, 0x51,
                0x1c,
            ],
            options: Some(Options {
                rk: true,
                plat: Some(false),
                always_uv: Some(false),
                cred_mgmt: Some(true),
                authnr_cfg: Some(true),
                client_pin: Some(false),
                large_blobs: Some(true),
                pin_uv_auth_token: Some(true),
                set_min_pin_length: Some(true),
                make_cred_uv_not_rqd: Some(true),
                credential_mgmt_preview: Some(true),
                ..Default::default()
            }),
            max_msg_size: Some(1280),
            pin_uv_auth_protocols: Some(vec![2, 1]),
            max_credential_count_in_list: Some(8),
            max_credential_id_length: Some(128),
            transports: Some(vec!["nfc".to_string(), "usb".to_string()]),
            algorithms: Some(vec![
                PublicKeyCredentialParameters::es256(),
                PublicKeyCredentialParameters::ed_dsa(),
                PublicKeyCredentialParameters::es384(),
            ]),
            max_serialized_large_blob_array: Some(4096),
            force_pin_change: Some(false),
            min_pin_length: Some(4),
            firmware_version: Some(0x05_0701),
            max_cred_blob_length: Some(32),
            max_rpids_for_set_min_pin_length: Some(1),
            remaining_discoverable_credentials: Some(100),
            ..Default::default()
        };
        let get_info_cbor = get_info.to_vec_cbor();
        let expected = [
            0xb1, 0x01, 0x84, 0x66, 0x55, 0x32, 0x46, 0x5f, 0x56, 0x32, 0x68, 0x46, 0x49, 0x44, 0x4f, 0x5f,
            0x32, 0x5f, 0x30, 0x6c, 0x46, 0x49, 0x44, 0x4f, 0x5f, 0x32, 0x5f, 0x31, 0x5f, 0x50, 0x52, 0x45,
            0x68, 0x46, 0x49, 0x44, 0x4f, 0x5f, 0x32, 0x5f, 0x31, 0x02, 0x85, 0x6b, 0x63, 0x72, 0x65, 0x64,
            0x50, 0x72, 0x6f, 0x74, 0x65, 0x63, 0x74, 0x6b, 0x68, 0x6d, 0x61, 0x63, 0x2d, 0x73, 0x65, 0x63,
            0x72, 0x65, 0x74, 0x6c, 0x6c, 0x61, 0x72, 0x67, 0x65, 0x42, 0x6c, 0x6f, 0x62, 0x4b, 0x65, 0x79,
            0x68, 0x63, 0x72, 0x65, 0x64, 0x42, 0x6c, 0x6f, 0x62, 0x6c, 0x6d, 0x69, 0x6e, 0x50, 0x69, 0x6e,
            0x4c, 0x65, 0x6e, 0x67, 0x74, 0x68, 0x03, 0x50, 0xa2, 0x53, 0x42, 0xc0, 0x3c, 0xdc, 0x44, 0x14,
            0x8e, 0x46, 0xf4, 0x80, 0x7f, 0xca, 0x51, 0x1c, 0x04, 0xac, 0x62, 0x72, 0x6b, 0xf5, 0x62, 0x75,
            0x70, 0xf5, 0x64, 0x70, 0x6c, 0x61, 0x74, 0xf4, 0x68, 0x61, 0x6c, 0x77, 0x61, 0x79, 0x73, 0x55,
            0x76, 0xf4, 0x68, 0x63, 0x72, 0x65, 0x64, 0x4d, 0x67, 0x6d, 0x74, 0xf5, 0x69, 0x61, 0x75, 0x74,
            0x68, 0x6e, 0x72, 0x43, 0x66, 0x67, 0xf5, 0x69, 0x63, 0x6c, 0x69, 0x65, 0x6e, 0x74, 0x50, 0x69,
            0x6e, 0xf4, 0x6a, 0x6c, 0x61, 0x72, 0x67, 0x65, 0x42, 0x6c, 0x6f, 0x62, 0x73, 0xf5, 0x6e, 0x70,
            0x69, 0x6e, 0x55, 0x76, 0x41, 0x75, 0x74, 0x68, 0x54, 0x6f, 0x6b, 0x65, 0x6e, 0xf5, 0x6f, 0x73,
            0x65, 0x74, 0x4d, 0x69, 0x6e, 0x50, 0x49, 0x4e, 0x4c, 0x65, 0x6e, 0x67, 0x74, 0x68, 0xf5, 0x70,
            0x6d, 0x61, 0x6b, 0x65, 0x43, 0x72, 0x65, 0x64, 0x55, 0x76, 0x4e, 0x6f, 0x74, 0x52, 0x71, 0x64,
            0xf5, 0x75, 0x63, 0x72, 0x65, 0x64, 0x65, 0x6e, 0x74, 0x69, 0x61, 0x6c, 0x4d, 0x67, 0x6d, 0x74,
            0x50, 0x72, 0x65, 0x76, 0x69, 0x65, 0x77, 0xf5, 0x05, 0x19, 0x05, 0x00, 0x06, 0x82, 0x02, 0x01,
            0x07, 0x08, 0x08, 0x18, 0x80, 0x09, 0x82, 0x63, 0x6e, 0x66, 0x63, 0x63, 0x75, 0x73, 0x62, 0x0a,
            0x83, 0xa2, 0x63, 0x61, 0x6c, 0x67, 0x26, 0x64, 0x74, 0x79, 0x70, 0x65, 0x6a, 0x70, 0x75, 0x62,
            0x6c, 0x69, 0x63, 0x2d, 0x6b, 0x65, 0x79, 0xa2, 0x63, 0x61, 0x6c, 0x67, 0x27, 0x64, 0x74, 0x79,
            0x70, 0x65, 0x6a, 0x70, 0x75, 0x62, 0x6c, 0x69, 0x63, 0x2d, 0x6b, 0x65, 0x79, 0xa2, 0x63, 0x61,
            0x6c, 0x67, 0x38, 0x22, 0x64, 0x74, 0x79, 0x70, 0x65, 0x6a, 0x70, 0x75, 0x62, 0x6c, 0x69, 0x63,
            0x2d, 0x6b, 0x65, 0x79, 0x0b, 0x19, 0x10, 0x00, 0x0c, 0xf4, 0x0d, 0x04, 0x0e, 0x1a, 0x00, 0x05,
            0x07, 0x01, 0x0f, 0x18, 0x20, 0x10, 0x01, 0x14, 0x18, 0x64,
        ];
        println!("get_info_cbor: {:02x?}", get_info_cbor);
        println!("expected: {:02x?}", expected);
        assert_eq!(&get_info_cbor, &expected);
    }
}
