// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

//! High-level API messages definitions.

use crate::error::BluetoothError;
use crate::{AdvChannel, BleVersionInfo, BtAddr, State};

#[derive(Debug, server::Message, rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)]
#[response(Result<BtAddr, BluetoothError>)]
pub struct GetBtAddr;

#[derive(Debug, server::Message)]
#[response(Result<(), BluetoothError>)]
pub struct EnableBle;

#[derive(Debug, server::Message)]
#[response(Result<(), BluetoothError>)]
pub struct DisableBle;

#[derive(Debug, server::Message)]
#[response(Result<(), BluetoothError>)]
pub struct Reset;

#[derive(Debug, server::Message)]
#[response(Result<(), BluetoothError>)]
pub struct Disconnect;

#[derive(Debug, server::Message)]
#[response(Result<State, BluetoothError>)]
pub struct GetState;

#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct BlePacket(pub Vec<u8>);

#[derive(Debug, server::Message, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[event(BlePacket)]
pub struct SubscribeBle;

#[derive(Debug, server::Message, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[event(State)]
pub struct SubscribeBleState;

#[derive(Debug, server::Message, rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)]
#[response(Result<(), BluetoothError>)]
pub struct SendBle(pub Vec<u8>);

#[derive(Debug, server::Message)]
pub struct Poll;

#[derive(Debug, server::Message)]
#[response(Result<(), BluetoothError>)]
pub struct DisableAdvChannels(pub AdvChannel);

#[derive(Debug, server::Message, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[response(Option<BleVersionInfo>)]
pub struct GetBleVersionInfo;

#[derive(Debug, server::Message, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[response(Result<(), BluetoothError>)]
pub struct TestEcho {
    pub size: usize,
    pub character: u8,
}
