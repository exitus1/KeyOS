// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

//! defer execution until drop
//!
//! use [`defer()`] when the cleanup function only needs to capture references from the surrounding scope.
//! useful for resource cleanup, logging, and ensuring code runs on early returns.
//!
//! use [`defer_with()`] when the cleanup function needs to consume an owned value.
//! useful when the value needs to live longer than the current scope, or when the cleanup function needs
//! ownership.

#![no_std]

use core::mem::ManuallyDrop;
use core::ops::{Deref, DerefMut};
use core::ptr::read;

/// run a cleanup function on drop
pub fn defer<F: FnOnce()>(f: F) -> Defer<impl FnOnce(())> {
    Defer { value: ManuallyDrop::new(()), dropfn: ManuallyDrop::new(|_| f()) }
}

/// store a value and pass ownership to a cleanup function on drop
/// can access value via [`Deref`] and [`DerefMut`]
pub fn defer_with<T, F: FnOnce(T)>(t: T, f: F) -> Defer<F, T> {
    Defer { value: ManuallyDrop::new(t), dropfn: ManuallyDrop::new(f) }
}

/// guard that runs a function on drop
#[must_use = "defer guard is dropped immediately if unused"]
pub struct Defer<F, T = ()>
where
    F: FnOnce(T),
{
    pub value: ManuallyDrop<T>,
    pub dropfn: ManuallyDrop<F>,
}

impl<F, T> Defer<F, T>
where
    F: FnOnce(T),
{
    /// cancel the deferred function and return the value
    pub fn cancel(self) -> T {
        let mut guard = ManuallyDrop::new(self);
        unsafe {
            let value = read(&*guard.value);
            ManuallyDrop::drop(&mut guard.dropfn);
            value
        }
    }
}

impl<F, T> Drop for Defer<F, T>
where
    F: FnOnce(T),
{
    fn drop(&mut self) {
        let (value, dropfn) = unsafe { (read(&*self.value), read(&*self.dropfn)) };
        dropfn(value);
    }
}

impl<F, T> Deref for Defer<F, T>
where
    F: FnOnce(T),
{
    type Target = T;

    fn deref(&self) -> &Self::Target { &self.value }
}

impl<F, T> DerefMut for Defer<F, T>
where
    F: FnOnce(T),
{
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.value }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate std;
    use std::cell::Cell;

    #[test]
    fn defer_executes_on_drop() {
        let executed = Cell::new(false);
        {
            let _guard = defer(|| executed.set(true));
            assert!(!executed.get());
        }
        assert!(executed.get());
    }

    #[test]
    fn defer_cancel() {
        let executed = Cell::new(false);
        {
            let guard = defer(|| executed.set(true));
            guard.cancel();
        }
        assert!(!executed.get());
    }

    #[test]
    fn defer_with_value() {
        let executed = Cell::new(0);
        {
            let _g = defer_with(42, |val| executed.set(val));
            assert_eq!(executed.get(), 0);
        }
        assert_eq!(executed.get(), 42);
    }

    #[test]
    fn defer_with_cancel() {
        let executed = Cell::new(0);
        let value = {
            let g = defer_with(42, |val| executed.set(val));
            g.cancel()
        };
        assert_eq!(value, 42);
        assert_eq!(executed.get(), 0);
    }
}
