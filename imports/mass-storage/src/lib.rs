#![no_std]
#![warn(missing_docs)]
//! This crate contains a simplified implementation of the Bulk-Only
//! Mass Storage protocol.
//!
//! It only contains the bare minimum functionality to read sectors.

mod commands;
mod emulation;
mod error;
mod host;

pub use commands::SenseKey;
pub use emulation::{AllowedAccess, BlockDeviceCommands, Buffer, MassStorageEmulation, UsbEmulationCommands};
pub use error::{BlockDeviceError, MassStorageError, UsbError};
pub use host::{MassStorageHost, UsbHostCommands};
