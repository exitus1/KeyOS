// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

mod path;
mod query;
mod segment;

pub use path::{serialize_path, serialize_path_partial};
pub use query::{serialize_query, serialize_query_partial};

macro_rules! unsupported {
    ($ty:ty) => {
        Err(Error::Unsupported(stringify!($ty)))
    };
    ($($ty:ty => $meth:ident,)*) => {
        $(
            fn $meth(self, _v: $ty) -> Result<Self::Ok, Self::Error> {
                Err(Error::Unsupported(stringify!($ty)))
            }
        )*
    };
}

pub(crate) use unsupported;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Custom string-based error
    #[error("{0}")]
    Custom(String),

    #[error("Unsupported Type {0}")]
    Unsupported(&'static str),

    #[error("Invalid UTF-8")]
    InvalidUtf8,

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl serde::ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Error::Custom(msg.to_string())
    }
}
