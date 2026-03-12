// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use bt::messages::BlePacket;
use btp::{Chunk, MasterDechunker};
use server::{CheckedConn, Owned, ServerContext};
use xous::PID;

use crate::{BluetoothApi, InternalPermissions};

#[derive(Debug, server::Message)]
pub struct BtRecvWake;

pub struct BtReceptionBridge {
    dechunker: MasterDechunker<10>,
    ql: CheckedConn<InternalPermissions>,
    tx: std::sync::mpsc::Sender<Vec<u8>>,
}

impl BtReceptionBridge {
    pub fn recv_bridge(tx: std::sync::mpsc::Sender<Vec<u8>>) {
        std::thread::spawn(|| {
            server::listen(Self { dechunker: MasterDechunker::default(), ql: Default::default(), tx })
        });
    }
}

impl server::ServerMessages for BtReceptionBridge {
    const NAME: &'static str = "";

    fn messages() -> &'static [server::MessageDef<Self>] { &[] }
}

impl server::Server for BtReceptionBridge {
    fn on_start(&mut self, context: &mut ServerContext<Self>) {
        xous::set_thread_priority(xous::ThreadPriority::System5).unwrap();
        BluetoothApi::default().subscribe_ble(context);
    }
}

const MAX_MSG_SIZE: usize = 1024 * 256;

impl server::ArchiveEventHandler<BlePacket> for BtReceptionBridge {
    fn handle(&mut self, msg: Owned<BlePacket>, _sender: PID, _context: &mut ServerContext<Self>) {
        let data = msg.0.as_slice();
        let chunk = match Chunk::decode(data) {
            Ok(c) => c,
            Err(e) => {
                log::error!("Failed to decode chunk: {e:?}");
                return;
            }
        };

        let total_size = chunk.header.total_chunks as usize * btp::CHUNK_DATA_SIZE;
        if total_size > MAX_MSG_SIZE {
            log::warn!("incoming message too large: {} kb", total_size / 1024);
            return;
        }

        let _msg_id = chunk.header.message_id;

        let res = match self.dechunker.insert_chunk_raw(chunk) {
            Ok((res, eviction)) => {
                if let Some(eviction) = eviction {
                    log::info!("message was evicted {:?}", eviction.dechunker.info);
                }
                match res {
                    Some(res) => res,
                    None => {
                        return;
                    }
                }
            }
            Err(e) => {
                log::warn!("invalid chunk {e:?}");
                return;
            }
        };

        self.tx.send(res).ok();

        if let Err(e) = self.ql.try_send_scalar(BtRecvWake) {
            log::info!("Could not wake quantum-link server: {e:?}")
        }
    }
}
