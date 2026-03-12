// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use num_traits::{FromPrimitive, ToPrimitive};

use crate::{command::Command, error::CtapHidError};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CmdSeq {
    Cmd { cmd: Command, payload_len: u16 },
    Seq(u8),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Header {
    pub cid: u32,
    pub cmd_seq: CmdSeq,
}

impl Header {
    pub const LEN: usize = 5;

    pub fn new(cid: u32, cmd_seq: CmdSeq) -> Self { Self { cid, cmd_seq } }

    pub fn len(&self) -> usize {
        match self.cmd_seq {
            CmdSeq::Cmd { cmd: _, payload_len: _ } => Self::LEN + 2,
            CmdSeq::Seq(_) => Self::LEN,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.cid.to_be_bytes());
        match self.cmd_seq {
            CmdSeq::Cmd { cmd, payload_len } => {
                buf.push(0x80 | cmd.to_u8().unwrap());
                buf.extend_from_slice(&payload_len.to_be_bytes());
            }
            CmdSeq::Seq(seq) => buf.push(seq),
        }
        buf
    }

    pub fn deserialize<'a>(buf: &'a [u8]) -> Result<(Self, &'a [u8]), CtapHidError> {
        if buf.len() >= Self::LEN {
            let cid = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
            let cmd_seq = if buf[4] & 0x80 == 0 {
                CmdSeq::Seq(buf[4])
            } else {
                CmdSeq::Cmd {
                    cmd: Command::from_u8(buf[4] & 0x7f).ok_or(CtapHidError::InvalidCommand)?,
                    payload_len: if buf.len() >= Self::LEN + 2 {
                        u16::from_be_bytes([buf[5], buf[6]])
                    } else {
                        return Err(CtapHidError::InvalidParam);
                    },
                }
            };
            let header = Self { cid, cmd_seq };
            Ok((header, &buf[header.len()..]))
        } else {
            Err(CtapHidError::InvalidParam)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_command() {
        let header = Header::new(0x1234, CmdSeq::Cmd { cmd: Command::Ping, payload_len: 0x5678 });
        let serialized = header.serialize();
        assert_eq!(&serialized, &[0x00, 0x00, 0x12, 0x34, 0x81, 0x56, 0x78]);
        let deserialized = Header::deserialize(&serialized).unwrap();
        assert_eq!(deserialized.0, header);
        assert_eq!(deserialized.1.len(), 0);
    }
    #[test]
    fn header_sequence() {
        let header = Header::new(0x1234, CmdSeq::Seq(5));
        let serialized = header.serialize();
        assert_eq!(&serialized, &[0x00, 0x00, 0x12, 0x34, 0x05]);
        let deserialized = Header::deserialize(&serialized).unwrap();
        assert_eq!(deserialized.0, header);
        assert_eq!(deserialized.1.len(), 0);
    }
}
