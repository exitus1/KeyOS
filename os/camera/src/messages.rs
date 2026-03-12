// SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

#[derive(Debug, server::Message)]
#[response(bool)]
pub struct IsReady;

#[derive(Debug, server::Message)]
#[response(Option<xous::MemoryRange>)]
pub struct GetFrameMemoryMirror;

#[derive(Debug, server::Message)]
pub struct FrameCaptured;

#[derive(Debug, server::Message)]
pub struct SetEnabled(pub bool);

#[derive(Debug, server::Message)]
pub struct NotifyVisible(pub bool);

#[derive(Debug, server::Message)]
#[response(bool)]
pub struct IsEnabled;

#[derive(Debug, server::Message)]
#[response(bool)]
pub struct IsInUse;

#[derive(Debug, server::Message, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[response([u8; 32])]
pub struct GetFrameBufId;
