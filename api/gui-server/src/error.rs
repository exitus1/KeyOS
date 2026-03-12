// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

#[derive(Debug, Default, thiserror::Error, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub enum NavigationError {
    #[default]
    #[error("Internal error")]
    InternalError,

    #[error("Request buffer is too small for the response")]
    RequestBufferTooSmall,

    #[error("An other PID tried to navigate there at the same time (PID={0})")]
    ConcurrentNavigationRequest(xous::PID),

    #[error("Couldn't find an running app with the given AppId")]
    AppIdNotFound,

    #[error("No pending navigation request")]
    NoNavigationRequest,

    #[error("Modal app exited unexpectedly")]
    ModalExited,

    #[error("The screen is locked")]
    Locked,

    #[error("The navigation was cancelled by the system")]
    CanceledBySystem,

    #[error("The navigation was cancelled by the user")]
    CanceledByUser,
}

#[derive(Debug, thiserror::Error, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub enum GuiServerError {
    #[error("Unknown internal error")]
    InternalError,

    #[error("OS error: {:?}", xous::Error::from_usize(*.0))]
    XousError(usize),

    #[error("navigation error: {0:?}")]
    Navigation(NavigationError),
}

impl From<xous::Error> for GuiServerError {
    fn from(e: xous::Error) -> Self { GuiServerError::XousError(e.to_usize()) }
}
