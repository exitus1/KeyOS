// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

//! error origin tracking
//!
//! wraps errors with the source location where they occurred using [`track_caller`].
//! use [`WhenceExt::whence()`] on results to automatically capture their origin.

pub type Result<T, E> = std::result::Result<T, Error<E>>;

#[derive(Copy, Clone)]
pub struct Error<E> {
    pub location: &'static std::panic::Location<'static>,
    pub error: E,
}

impl<E: std::fmt::Debug> std::fmt::Debug for Error<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} [{}:{}:{}]",
            self.error,
            self.location.file(),
            self.location.line(),
            self.location.column()
        )
    }
}

impl<E> Error<E> {
    #[inline]
    pub fn into_inner(self) -> E { self.error }

    #[inline]
    pub fn map<A>(self, f: impl FnOnce(E) -> A) -> Error<A> {
        Error { location: self.location, error: f(self.error) }
    }
}

impl<E: PartialEq> PartialEq for Error<E> {
    fn eq(&self, other: &Self) -> bool { self.error == other.error }
}

impl<E: PartialEq> PartialEq<E> for Error<E> {
    fn eq(&self, other: &E) -> bool { &self.error == other }
}

impl<E: Eq> Eq for Error<E> {}

impl<A> From<A> for Error<A> {
    #[track_caller]
    fn from(error: A) -> Self { Self { location: std::panic::Location::caller(), error: error.into() } }
}

impl<E> std::ops::Deref for Error<E> {
    type Target = E;

    fn deref(&self) -> &Self::Target { &self.error }
}

impl<E> std::fmt::Display for Error<E>
where
    E: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} [{}:{}:{}]",
            self.error,
            self.location.file(),
            self.location.line(),
            self.location.column()
        )
    }
}

/// extension trait to capture error locations
pub trait WhenceExt<T, E> {
    fn whence(self) -> std::result::Result<T, Error<E>>;
}

impl<T, E1, E2> WhenceExt<T, E2> for std::result::Result<T, E1>
where
    E2: From<E1>,
{
    #[inline]
    #[track_caller]
    fn whence(self) -> std::result::Result<T, Error<E2>> {
        match self {
            Ok(r) => Ok(r),
            Err(e) => Err(Error::from(E2::from(e))),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::*;

    #[test]
    fn test_location_capture() {
        fn returns_error() -> std::result::Result<(), String> { Err("test error".to_string()) }
        let result: Result<(), Cow<'static, str>> = returns_error().whence();
        let expected_line = line!() - 1;

        let err = result.unwrap_err();

        assert_eq!(err.location.file(), file!());
        assert_eq!(err.location.line(), expected_line);
        assert_eq!(err.error, "test error");
    }
}
