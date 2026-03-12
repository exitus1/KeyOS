// SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: MIT OR Apache-2.0

crypto::use_api!();

fn main() {
    log_server::init_wait(env!("CARGO_CRATE_NAME")).unwrap();
    log::set_max_level(log::LevelFilter::Info);
    log::info!("running shamir test");
    let crypto = CryptoApi::default();
    let shares = crypto.split_secret(b"hello world hello world0".to_vec(), 3, 2).unwrap();
    log::info!("split shares: {:?}", shares);
    let secret = crypto.recover_secret(vec![0, 2], vec![shares[0].clone(), shares[2].clone()]).unwrap();
    log::info!("original secret: {:?}", String::from_utf8(secret));
}
