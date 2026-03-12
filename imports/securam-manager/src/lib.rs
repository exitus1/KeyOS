#![no_std]

use core::ffi::CStr;
use core::fmt::{Display, Formatter};

use crcxx::crc32::*;
use zeroize::Zeroize;

/// The actual in-memory structure of the SECURAM
/// See [`SecuramManager`] for documentation of the fields
#[repr(C)]
struct SecuramFields {
    magic: u32,
    otp_key: [u8; 72],
    io_protection_secret: [u8; 32],
    security_check_secret: [u8; 32],
    bluetooth_challenge_secret: [u8; 32],
    bluetooth_challenge_secret_sent: u8,
    pin_entry_mode: u8,

    // Reserved bytes.
    // Fields above this should be generally preserved
    // Fields below are rewritten between boots.
    _zeros: [u8; 1738],

    disk_encryption_keys: ([u8; 32], [u8; 32]),
    aes_keys: [AesKey; NUM_SECURAM_AES_KEYS],
    kernel_panic_message: KernelPanicMessage,
    os_arguments: OsArguments,
    _padding2: u32, // OsArguments is aligned to 8 bytes
    checksum: u32,
}

#[repr(C)]
pub struct AesKey {
    len: u8,
    bytes: [u8; 32],
}

pub const SECURAM_SIZE: usize = 0x1000;
pub const NUM_SECURAM_AES_KEYS: usize = 32;
const SECURAM_MAGIC: u32 = 0x32636573; // "sec2"
const _: () = assert!(core::mem::size_of::<SecuramFields>() == SECURAM_SIZE);

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum OsArguments {
    Unknown([u8; 31]) = 0,
    RecoveryMode { bootloader_version: [u8; 8], bootloader_build_date: u64 },
    SystemInfoMode { bootloader_version: [u8; 8], bootloader_build_date: u64 },
    NormalMode { bootloader_version: [u8; 8], keyos_version: [u8; 20], _padding: [u8; 3] },
}

const _: () = assert!(core::mem::size_of::<OsArguments>() == 32);

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct KernelPanicMessage(pub [u8; Self::MAX_MSG_LENGTH]);

pub struct SecuramManager {
    fields: &'static mut SecuramFields,
}

#[cfg(feature = "bootloader")]
const CRC32: Crc<LookupTable32> = Crc::<LookupTable32>::new(&catalog::CRC_32_BZIP2);

#[cfg(not(feature = "bootloader"))]
const CRC32: Crc<LookupTable256xN<16>> = Crc::<LookupTable256xN<16>>::new(&catalog::CRC_32_BZIP2);

impl SecuramManager {
    /// Create a new SecuramManager from the SECURAM page. Checks integrity.
    ///
    /// # Safety
    /// The caller should make sure that address actually points to the SECURAM, at least SECURAM_SIZE
    /// bytes are available, and it's aligned to at least 4 bytes.
    pub unsafe fn new(address: *mut u8) -> Result<Self, Error> {
        let this = Self { fields: (address as *mut SecuramFields).as_mut().unwrap() };

        this.check_magic()?;
        this.check_checksum()?;

        Ok(this)
    }

    /// Create a new SecuramManager from the SECURAM page, clearing all data, including the magic.
    /// The magic should only be set once the fields are in a consistent state.
    ///
    /// # Safety
    /// The caller should make sure that address actually points to the SECURAM, at least SECURAM_SIZE
    /// bytes are available, and it's aligned to at least 4 bytes.
    pub unsafe fn new_clear(address: *mut u8) -> Self {
        let mut this = Self { fields: (address as *mut SecuramFields).as_mut().unwrap() };
        this.clear();
        this
    }

    /// Zero-out the entire SECURAM page, including the magic.
    ///
    /// # Safety
    ///
    /// The caller should make sure that the SECURAM fields address is valid and aligned to at least 4 bytes.
    pub unsafe fn clear(&mut self) {
        unsafe {
            (*(self.fields as *const _ as *mut [u32; SECURAM_SIZE / 4])).zeroize();
        }
    }

    fn get_field<'a, T>(&'a self, f: impl FnOnce(&'a SecuramFields) -> T) -> Result<T, Error> {
        self.check_magic()?;
        Ok(f(self.fields))
    }

    fn set_field(&mut self, f: impl FnOnce(&mut SecuramFields)) -> Result<(), Error> {
        self.check_magic()?;
        self.set_field_no_check_magic(f)
    }

    fn set_field_no_check_magic(&mut self, f: impl FnOnce(&mut SecuramFields)) -> Result<(), Error> {
        f(self.fields);
        self.recalc_checksum()
    }

    fn check_magic(&self) -> Result<(), Error> {
        if self.fields.magic == SECURAM_MAGIC {
            Ok(())
        } else {
            Err(Error::MagicMismatch)
        }
    }

    /// Fixed value, so we can check the strucutre version or if the SECURAM was cleared
    pub fn set_magic(&mut self) -> Result<(), Error> {
        self.set_field_no_check_magic(|fields| fields.magic = SECURAM_MAGIC)
    }

    /// Bytes that XOR the Seed before storing it in the secure element.
    /// Set to random bytes when setting the
    pub fn otp_key(&self) -> Result<&[u8; 72], Error> { self.get_field(|fields| &fields.otp_key) }

    pub fn set_otp_key(&mut self, key: &[u8; 72]) -> Result<(), Error> {
        self.set_field(|fields| fields.otp_key.copy_from_slice(key))
    }

    /// Key to use when communicating with the secure element.
    /// Fixed per device, calculated from the fused entropy.
    pub fn io_protection_secret(&self) -> Result<&[u8; 32], Error> {
        self.get_field(|fields| &fields.io_protection_secret)
    }

    #[cfg(feature = "bootloader")]
    pub fn set_io_protection_secret(&mut self, io_protection_secret: &[u8; 32]) -> Result<(), Error> {
        // No check on the magic, because this field is set before setting the magic.
        self.set_field_no_check_magic(|fields| {
            fields.io_protection_secret.copy_from_slice(io_protection_secret)
        })
    }

    /// Key to use to prove the bootloader was not tampered with.
    /// Fixed, calculated from the "extra entropy" value in the bootloader.
    pub fn security_check_secret(&self) -> Result<&[u8; 32], Error> {
        self.get_field(|fields| &fields.security_check_secret)
    }

    #[cfg(feature = "bootloader")]
    pub fn set_security_check_secret(&mut self, security_check_secret: &[u8; 32]) -> Result<(), Error> {
        // No check on the magic, because this field is set before setting the magic.
        self.set_field_no_check_magic(|fields| {
            fields.security_check_secret.copy_from_slice(security_check_secret)
        })
    }

    /// Key to use to challenge the bluetooth chip regularly to check if it was mass erased.
    /// Fixed per device, calculated from the "extra entropy" value in the bootloader and the fused
    /// entropy.
    pub fn bluetooth_challenge_secret(&self) -> Result<&[u8; 32], Error> {
        self.get_field(|fields| &fields.bluetooth_challenge_secret)
    }

    #[cfg(feature = "bootloader")]
    pub fn set_bluetooth_challenge_secret(
        &mut self,
        bluetooth_challenge_secret: &[u8; 32],
    ) -> Result<(), Error> {
        // No check on the magic, because this field is set before setting the magic.
        self.set_field_no_check_magic(|fields| {
            fields.bluetooth_challenge_secret.copy_from_slice(bluetooth_challenge_secret)
        })
    }

    /// Flag to store if the secret was shared with the BT chip.
    /// Reset to 0 on SECURAM clear, set to 1 once the secret was sent successfully.
    pub fn bluetooth_challenge_secret_sent(&self) -> Result<bool, Error> {
        self.get_field(|fields| fields.bluetooth_challenge_secret_sent != 0)
    }

    pub fn set_bluetooth_challenge_secret_sent(&mut self) -> Result<(), Error> {
        self.set_field(|fields| fields.bluetooth_challenge_secret_sent = 1)
    }

    pub fn pin_entry_mode(&self) -> Result<u8, Error> {
        self.get_field(|fields| fields.pin_entry_mode)
    }

    pub fn set_pin_entry_mode(&mut self, mode: u8) -> Result<(), Error> {
        self.set_field(|fields| fields.pin_entry_mode = mode)
    }

    /// XTS keys used to encrypt/decrypt the encrypted eMMC partition
    #[cfg(keyos)]
    pub fn disk_encryption_keys(
        &self,
    ) -> Result<(atsama5d27::aes::Key<'_>, atsama5d27::aes::Key<'_>), Error> {
        self.get_field(|fields| {
            ((&fields.disk_encryption_keys.0).into(), (&fields.disk_encryption_keys.1).into())
        })
    }

    pub fn set_disk_encryption_keys(&mut self, (dk0, dk1): (&[u8; 32], &[u8; 32])) -> Result<(), Error> {
        self.set_field(|fields| {
            fields.disk_encryption_keys.0 = *dk0;
            fields.disk_encryption_keys.1 = *dk1;
        })
    }

    /// Ephemeral AES key slots, used by the crypto server during runtime to safely store keys.
    ///
    /// Panics if key number is out of range (see [`NUM_SECURAM_AES_KEYS`])
    #[cfg(keyos)]
    pub fn aes_key(&self, key_no: usize) -> Result<atsama5d27::aes::Key<'_>, Error> {
        self.get_field(|fields| {
            let key = &fields.aes_keys[key_no];
            (&key.bytes[..key.len as usize]).try_into().unwrap()
        })
    }

    pub fn set_aes_key(&mut self, key_no: usize, aes_key: &[u8]) -> Result<(), Error> {
        if aes_key.len() != 16 && aes_key.len() != 24 && aes_key.len() != 32 {
            return Err(Error::WrongKeySize);
        }
        self.set_field(|fields| {
            fields.aes_keys[key_no].bytes[..aes_key.len()].copy_from_slice(aes_key);
            fields.aes_keys[key_no].bytes[aes_key.len()..].fill(0);
            fields.aes_keys[key_no].len = aes_key.len() as u8;
        })
    }

    /// Text of the last kernel panic
    pub fn kernel_panic_message(&self) -> Result<&KernelPanicMessage, Error> {
        self.get_field(|fields| &fields.kernel_panic_message)
    }

    pub fn set_kernel_panic_message(
        &mut self,
        kernel_panic_message: &KernelPanicMessage,
    ) -> Result<(), Error> {
        self.set_field(|fields| fields.kernel_panic_message = kernel_panic_message.clone())
    }

    /// Data to pass to the booted OS
    pub fn os_arguments(&self) -> Result<&OsArguments, Error> {
        self.get_field(|fields| &fields.os_arguments)
    }

    #[cfg(feature = "bootloader")]
    pub fn set_os_arguments(&mut self, os_arguments: &OsArguments) -> Result<(), Error> {
        self.set_field(|fields| fields.os_arguments = os_arguments.clone())
    }

    fn check_checksum(&self) -> Result<(), Error> {
        if self.fields.checksum == self.calc_checksum() {
            Ok(())
        } else {
            Err(Error::ChecksumMismatch)
        }
    }

    fn recalc_checksum(&mut self) -> Result<(), Error> {
        let checksum = self.calc_checksum();
        self.fields.checksum = checksum;
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
        Ok(())
    }

    fn calc_checksum(&self) -> u32 {
        CRC32.compute(unsafe { &*(self.fields as *const SecuramFields as *const [u8; SECURAM_SIZE - 4]) })
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Error {
    MagicMismatch,
    ChecksumMismatch,
    WrongKeySize,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::MagicMismatch => f.write_str("SECURAM magic mismatch"),
            Error::ChecksumMismatch => f.write_str("SECURAM checksum mismatch"),
            Error::WrongKeySize => f.write_str("AES key size must be 16, 24 or 32 bytes"),
        }
    }
}

impl KernelPanicMessage {
    pub const MAX_MSG_LENGTH: usize = 1024;

    pub const fn new_empty() -> Self { Self([0u8; Self::MAX_MSG_LENGTH]) }

    pub fn is_empty(&self) -> bool { self.0[0] == 0 }

    pub fn as_str(&self) -> Option<&str> {
        if let Ok(cstr) = CStr::from_bytes_until_nul(&self.0) {
            if let Ok(cstr) = cstr.to_str() {
                return Some(cstr);
            }
        }

        None
    }
}

impl Display for KernelPanicMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        if let Some(str) = self.as_str() {
            f.write_str(str)?
        } else {
            f.write_str("<invalid utf-8>")?
        }

        Ok(())
    }
}

impl From<&str> for KernelPanicMessage {
    fn from(value: &str) -> Self { Self::from(value.as_bytes()) }
}

impl From<&[u8]> for KernelPanicMessage {
    fn from(value: &[u8]) -> Self {
        let mut msg_buf = [0u8; Self::MAX_MSG_LENGTH];
        let max_len = value.len().min(Self::MAX_MSG_LENGTH);
        msg_buf[..max_len].copy_from_slice(value);
        KernelPanicMessage(msg_buf)
    }
}
