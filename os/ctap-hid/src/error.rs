// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use server::{AsScalar, FromScalar};

use crate::{
    command::Command,
    header::{CmdSeq, Header},
};

/// The command in the request is invalid
const ERR_INVALID_CMD: u8 = 0x01;
/// The parameter(s) in the request is invalid
const ERR_INVALID_PAR: u8 = 0x02;
/// The length field (BCNT) is invalid for the request
const ERR_INVALID_LEN: u8 = 0x03;
/// The sequence does not match expected value
const ERR_INVALID_SEQ: u8 = 0x04;
/// The message has timed out
const ERR_MSG_TIMEOUT: u8 = 0x05;
/// The device is busy for the requesting channel. The client SHOULD retry the request after a short delay.
/// Note that the client MAY abort the transaction if the command is no longer relevant.
const ERR_CHANNEL_BUSY: u8 = 0x06;
/// Command requires channel lock
const ERR_LOCK_REQUIRED: u8 = 0x0A;
/// CID is not valid
const ERR_INVALID_CHANNEL: u8 = 0x0B;
/// Unspecified error
const ERR_OTHER: u8 = 0x7f;

#[derive(Debug, Clone, Copy, thiserror::Error, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub enum CtapHidError {
    #[error("OS error: {:?}", xous::Error::from_usize(*.0))]
    Xous(usize),
    #[error("Usb error: {0:?}")]
    Usb(usb::error::UsbError),
    #[error("Invalid command")]
    InvalidCommand,
    #[error("Invalid parameter")]
    InvalidParam,
    #[error("Invalid payload length")]
    InvalidPayloadLen,
    #[error("Invalid sequence")]
    InvalidSequence,
    #[error("Message timed out")]
    MsgTimeout,
    #[error("Channel busy")]
    BusyChannel,
    #[error("Lock required")]
    LockRequired,
    #[error("Invalid channel")]
    InvalidChannel,
    #[error("Other error")]
    Other,
}

impl From<xous::Error> for CtapHidError {
    fn from(value: xous::Error) -> Self { CtapHidError::Xous(value.to_usize()) }
}

impl From<usb::error::UsbError> for CtapHidError {
    fn from(value: usb::error::UsbError) -> Self { Self::Usb(value) }
}

impl From<CtapHidError> for u8 {
    fn from(e: CtapHidError) -> u8 {
        match e {
            CtapHidError::InvalidCommand => ERR_INVALID_CMD,
            CtapHidError::InvalidParam => ERR_INVALID_PAR,
            CtapHidError::InvalidPayloadLen => ERR_INVALID_LEN,
            CtapHidError::InvalidSequence => ERR_INVALID_SEQ,
            CtapHidError::MsgTimeout => ERR_MSG_TIMEOUT,
            CtapHidError::BusyChannel => ERR_CHANNEL_BUSY,
            CtapHidError::LockRequired => ERR_LOCK_REQUIRED,
            CtapHidError::InvalidChannel => ERR_INVALID_CHANNEL,
            _ => ERR_OTHER,
        }
    }
}

impl CtapHidError {
    pub fn to_cmd_payload(self) -> (Command, Vec<u8>) { (Command::Error, vec![self.into()]) }

    pub fn to_msg(self, cid: u32) -> (Header, Vec<u8>) {
        let (cmd, payload) = self.to_cmd_payload();
        (Header::new(cid, CmdSeq::Cmd { cmd, payload_len: payload.len() as u16 }), payload)
    }
}

impl AsScalar<3> for CtapHidError {
    fn as_scalar(&self) -> [u32; 3] {
        match self {
            CtapHidError::Xous(e) => [1, *e as u32, 0],
            CtapHidError::Usb(e) => [2, AsScalar::<2>::as_scalar(e)[0], AsScalar::<2>::as_scalar(e)[1]],
            CtapHidError::InvalidCommand => [3, 0, 0],
            CtapHidError::InvalidParam => [4, 0, 0],
            CtapHidError::InvalidPayloadLen => [5, 0, 0],
            CtapHidError::InvalidSequence => [6, 0, 0],
            CtapHidError::MsgTimeout => [7, 0, 0],
            CtapHidError::BusyChannel => [8, 0, 0],
            CtapHidError::LockRequired => [9, 0, 0],
            CtapHidError::InvalidChannel => [10, 0, 0],
            CtapHidError::Other => [11, 0, 0],
        }
    }
}

impl FromScalar<3> for CtapHidError {
    fn from_scalar(value: [u32; 3]) -> Self {
        match value[0] {
            1 => CtapHidError::Xous(value[1] as usize),
            2 => CtapHidError::Usb(usb::error::UsbError::from_scalar([value[1], value[2]])),
            3 => CtapHidError::InvalidCommand,
            4 => CtapHidError::InvalidParam,
            5 => CtapHidError::InvalidPayloadLen,
            6 => CtapHidError::InvalidSequence,
            7 => CtapHidError::MsgTimeout,
            8 => CtapHidError::BusyChannel,
            9 => CtapHidError::LockRequired,
            10 => CtapHidError::InvalidChannel,
            _ => CtapHidError::Other,
        }
    }
}

impl From<usize> for CtapHidError {
    fn from(value: usize) -> Self { Self::from_scalar([value as u32, 0, 0]) }
}

impl From<CtapHidError> for usize {
    fn from(value: CtapHidError) -> Self { server::AsScalar::<3>::as_scalar(&value)[0] as usize }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_to_msg() {
        assert_eq!(
            CtapHidError::InvalidCommand.to_msg(0),
            (Header::new(0, CmdSeq::Cmd { cmd: Command::Error, payload_len: 1 }), vec![0x01])
        );
        assert_eq!(
            CtapHidError::InvalidParam.to_msg(0),
            (Header::new(0, CmdSeq::Cmd { cmd: Command::Error, payload_len: 1 }), vec![0x02])
        );
        assert_eq!(
            CtapHidError::InvalidPayloadLen.to_msg(0),
            (Header::new(0, CmdSeq::Cmd { cmd: Command::Error, payload_len: 1 }), vec![0x03])
        );
        assert_eq!(
            CtapHidError::InvalidSequence.to_msg(0),
            (Header::new(0, CmdSeq::Cmd { cmd: Command::Error, payload_len: 1 }), vec![0x04])
        );
        assert_eq!(
            CtapHidError::MsgTimeout.to_msg(0),
            (Header::new(0, CmdSeq::Cmd { cmd: Command::Error, payload_len: 1 }), vec![0x05])
        );
        assert_eq!(
            CtapHidError::BusyChannel.to_msg(0),
            (Header::new(0, CmdSeq::Cmd { cmd: Command::Error, payload_len: 1 }), vec![0x06])
        );
        assert_eq!(
            CtapHidError::LockRequired.to_msg(0),
            (Header::new(0, CmdSeq::Cmd { cmd: Command::Error, payload_len: 1 }), vec![0x0a])
        );
        assert_eq!(
            CtapHidError::InvalidChannel.to_msg(0),
            (Header::new(0, CmdSeq::Cmd { cmd: Command::Error, payload_len: 1 }), vec![0x0b])
        );
        assert_eq!(
            CtapHidError::Other.to_msg(0),
            (Header::new(0, CmdSeq::Cmd { cmd: Command::Error, payload_len: 1 }), vec![0x7f])
        );
    }
}
