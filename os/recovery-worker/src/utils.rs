// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

pub(crate) fn convert_cosign2_header(
    header: Result<cosign2::Header, fw_utils::hash::HashError>,
) -> ([u8; 32], String, String, String, u32, bool) {
    if let Ok(header) = header {
        #[cfg(feature = "production")]
        let is_trusted = matches!(header.trust(), cosign2::Trust::FullyTrusted);
        #[cfg(not(feature = "production"))]
        let is_trusted = true;

        if !is_trusted {
            log::error!("Firmware not signed by a trusted key: {:?}", header.trust());
            return (
                [0; 32],
                "<untrusted>".to_string(),
                "<untrusted>".to_string(),
                "<untrusted>".to_string(),
                0,
                false,
            );
        }

        let hash_bytes = header.binary_hash();
        let hash = hex::encode(header.binary_hash());
        let version = header.version();
        let build_date = header.date();
        let timestamp = header.timestamp();
        (*hash_bytes, hash, version.to_string(), build_date.to_string(), timestamp, true)
    } else {
        log::error!("Error verifying firmware: {header:?}");
        ([0; 32], "<invalid>".to_string(), "<invalid>".to_string(), "<invalid>".to_string(), 0, false)
    }
}
