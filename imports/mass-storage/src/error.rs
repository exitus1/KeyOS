use crate::commands::{
    SenseKey, INVALID_FIELD_IN_CBD, LOGICAL_BLOCK_ADDRESS_OUT_OF_RANGE, LOGICAL_UNIT_FAILURE,
};

/// General errors coming from the crate
#[derive(Debug)]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
pub enum MassStorageError {
    /// A irrecoverable USB error happened
    Usb(UsbError),
    /// A irrecoverable Block Device error happened
    BlockDevice(BlockDeviceError),
    /// A wrong length field was received during transfer (e.g. not 13 in case of CSW)
    InvalidLength,
    /// The CSW header was wrong
    InvalidCswSignature,
    /// The received tag of the CSW was not in sznc with the CBW
    InvalidCswTag,
    /// The command itself was indicated as failed in the CSW
    CommandFailed,
    /// The SCSI command failed
    SenseError(SenseKey),
    /// The device reported a Phase Error and needs to reset
    PhaseError,
    /// The device is not a Direct Access Mass storeage device.
    /// (But e.g. a CDROM drive)
    NotDirectAccess,
    /// The function was called with a wrong parameter
    InvalidArgument,
    /// Other, unknown error happened
    OtherError,
    /// The CBW itself was not valid or not meaningful
    InvalidCbw,
    /// The Command in the CBW could not be parsed
    InvalidCbwCommand,
}

/// Usb error returned by the actual USB implementation
#[derive(Debug)]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
pub enum UsbError {
    /// The endpoint stalled while transfering
    Stalled,
    /// The device was disconnected
    Disconnected,
    /// Other, unknown issue.
    Other,
}

/// Block Device error returned by the actual Block Device implementation
#[derive(Debug)]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
pub enum BlockDeviceError {
    /// Too many blocks were requested in a single operation
    TooManyBlocks,
    /// Block index was out range
    OutOfRange,
    /// Other, unknown issue.
    Other,
}

pub type Result<T> = core::result::Result<T, MassStorageError>;

impl From<UsbError> for MassStorageError {
    fn from(value: UsbError) -> Self { Self::Usb(value) }
}

impl From<BlockDeviceError> for MassStorageError {
    fn from(value: BlockDeviceError) -> Self { Self::BlockDevice(value) }
}

impl BlockDeviceError {
    pub(crate) fn sense_key(&self) -> SenseKey {
        match self {
            BlockDeviceError::TooManyBlocks => SenseKey::IllegalRequest,
            BlockDeviceError::OutOfRange => SenseKey::IllegalRequest,
            BlockDeviceError::Other => SenseKey::HardwareError,
        }
    }

    pub(crate) fn sense_code(&self) -> (u8, u8) {
        match self {
            BlockDeviceError::TooManyBlocks => INVALID_FIELD_IN_CBD,
            BlockDeviceError::OutOfRange => LOGICAL_BLOCK_ADDRESS_OUT_OF_RANGE,
            BlockDeviceError::Other => LOGICAL_UNIT_FAILURE,
        }
    }
}
