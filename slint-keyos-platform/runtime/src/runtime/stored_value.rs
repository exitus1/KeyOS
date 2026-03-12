// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::marker::PhantomData;

use super::core::{try_with_runtime, InnerStoredValue, MainThreadMarker, StoredValueKey};

/// A global mutable value stored on the main thread.
///
/// Similar to RefCell, borrows should be dropped as soon as possible to avoid borrow errors.
/// Prefer using [`StoredValue::with()`] for mutations as it automatically drops the borrow.
///
/// # Example
/// ```
/// # slint_keyos_platform::Runtime::unsafe_init(|| ());
/// let stored_value = slint_keyos_platform::StoredValue::new(42);
/// stored_value.with(|value| {
///     *value += 1;
/// });
/// assert_eq!(stored_value.get(), 43);
/// ```
pub struct StoredValue<T> {
    key: StoredValueKey,
    ty: PhantomData<T>,
    #[allow(unused)]
    marker: MainThreadMarker,
}

impl<T> Copy for StoredValue<T> {}
impl<T> Clone for StoredValue<T> {
    fn clone(&self) -> Self { *self }
}

impl<T> StoredValue<T>
where
    T: 'static,
{
    /// Creates a new `StoredValue` containing the given value.
    ///
    /// Must be created on the main thread.
    ///
    /// # Example
    /// ```
    /// # slint_keyos_platform::Runtime::unsafe_init(|| ());
    /// let counter = slint_keyos_platform::StoredValue::new(0);
    /// ```
    #[track_caller]
    pub fn new(value: T) -> Self {
        let key = InnerStoredValue::new(value);

        StoredValue { key, ty: Default::default(), marker: Default::default() }
    }

    /// Provides mutable access to the stored value through a closure.
    ///
    /// Preferred way to mutate values as it automatically drops the borrow when done.
    ///
    /// # Example
    /// ```
    /// # slint_keyos_platform::Runtime::unsafe_init(|| ());
    /// # let counter = slint_keyos_platform::StoredValue::new(0);
    /// counter.with(|value| *value += 1);
    /// ```
    #[inline]
    #[track_caller]
    pub fn with<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        match self.try_with(f) {
            Ok(r) => r,
            Err(e) => panic!("{e}"),
        }
    }

    /// Attempts to provide mutable access to the stored value through a closure.
    ///
    /// Returns a Result instead of panicking on failure.
    ///
    /// # Example
    /// ```
    /// # slint_keyos_platform::Runtime::unsafe_init(|| ());
    /// # let counter = slint_keyos_platform::StoredValue::new(0);
    /// if let Ok(_) = counter.try_with(|value| *value += 1) {
    ///     println!("Value updated");
    /// }
    /// ```
    #[inline]
    pub fn try_with<R>(&self, f: impl FnOnce(&mut T) -> R) -> Result<R, StoredValueError> {
        let mut value = self.try_borrow_mut()?;
        let result = f(&mut value);
        Ok(result)
    }

    /// Borrows the value mutably.
    ///
    /// Warning: Drop the returned [`StoredRefMut`] as soon as possible.
    /// Consider using [`StoredValue::with()`] instead.
    ///
    /// # Example
    /// ```
    /// # slint_keyos_platform::Runtime::unsafe_init(|| ());
    /// # let counter = slint_keyos_platform::StoredValue::new(0);
    /// {
    ///     let mut value = counter.borrow_mut();
    ///     *value += 1;
    /// } // borrow is dropped here
    /// ```
    #[inline]
    #[track_caller]
    pub fn borrow_mut(&self) -> StoredRefMut<'_, T> {
        self.try_borrow_mut().expect("stored value borrow mut")
    }

    /// Attempts to borrow the value mutably.
    ///
    /// Warning: If successful, drop the returned [`StoredRefMut`] as soon as possible.
    ///
    /// # Example
    /// ```
    /// # slint_keyos_platform::Runtime::unsafe_init(|| ());
    /// # let counter = slint_keyos_platform::StoredValue::new(0);
    /// if let Ok(mut value) = counter.try_borrow_mut() {
    ///     *value += 1;
    /// }
    /// ```
    #[inline]
    pub fn try_borrow_mut(&self) -> Result<StoredRefMut<'_, T>, StoredValueError> {
        StoredRefMut::new(self.key)
    }

    /// Borrows the value immutably.
    ///
    /// Warning: Drop the returned [`StoredRef`] as soon as possible.
    ///
    /// # Example
    /// ```
    /// # slint_keyos_platform::Runtime::unsafe_init(|| ());
    /// # let counter = slint_keyos_platform::StoredValue::new(42);
    /// {
    ///     let value = counter.borrow();
    ///     println!("Value: {}", *value);
    /// } // borrow is dropped here
    /// ```
    #[inline]
    #[track_caller]
    pub fn borrow(&self) -> StoredRef<'_, T> { self.try_borrow().expect("stored value borrow") }

    /// Attempts to borrow the value immutably.
    ///
    /// Warning: If successful, drop the returned [`StoredRef`] as soon as possible.
    ///
    /// # Example
    /// ```
    /// # slint_keyos_platform::Runtime::unsafe_init(|| ());
    /// # let counter = slint_keyos_platform::StoredValue::new(42);
    /// if let Ok(value) = counter.try_borrow() {
    ///     println!("Value: {}", *value);
    /// }
    /// ```
    #[inline]
    pub fn try_borrow(&self) -> Result<StoredRef<'_, T>, StoredValueError> { StoredRef::new(self.key) }

    /// Sets the stored value to a new value.
    ///
    /// Uses `.with()` internally, so it automatically handles proper borrowing.
    ///
    /// # Example
    /// ```
    /// # slint_keyos_platform::Runtime::unsafe_init(|| ());
    /// # let counter = slint_keyos_platform::StoredValue::new(42);
    /// counter.set(0); // Reset counter
    /// assert_eq!(counter.get(), 0);
    /// ```
    #[inline]
    #[track_caller]
    pub fn set(&self, value: T) { self.with(|v| *v = value); }
}

impl<T: Clone + 'static> StoredValue<T> {
    /// Gets a clone of the stored value.
    ///
    /// Requires `T` to implement `Clone`.
    ///
    /// # Example
    /// ```
    /// # slint_keyos_platform::Runtime::unsafe_init(|| ());
    /// # let counter = slint_keyos_platform::StoredValue::new(42);
    /// let current = counter.get();
    /// assert_eq!(current, 42);
    /// ```
    #[inline]
    #[track_caller]
    pub fn get(&self) -> T { self.borrow().clone() }
}

impl<T> StoredValue<T>
where
    T: Default + 'static,
{
    /// Takes the value, leaving `Default::default()` in its place.
    ///
    /// # Example
    /// ```
    /// # slint_keyos_platform::Runtime::unsafe_init(|| ());
    /// # let counter = slint_keyos_platform::StoredValue::new(42);
    /// let old_value = counter.take();
    /// assert_eq!(old_value, 42);
    /// assert_eq!(counter.get(), 0); // Default for i32 is 0
    /// ```
    #[inline]
    #[track_caller]
    pub fn take(&self) -> T { self.with(|value| std::mem::take(value)) }
}

impl<T> std::fmt::Debug for StoredValue<T>
where
    T: std::fmt::Debug + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut dbg = f.debug_struct("StoredValue");
        dbg.field("ty", &self.ty);
        self.with(|v| dbg.field("value", v));
        dbg.finish()
    }
}

pub use stored_ref::*;

mod stored_ref {
    use std::cell::{Ref, RefMut};

    use super::*;

    /// An immutable reference to a stored value.
    ///
    /// Warning: Drop as soon as possible to allow other code to borrow the value.
    ///
    /// # Example
    /// ```
    /// # slint_keyos_platform::Runtime::unsafe_init(|| ());
    /// # let counter = slint_keyos_platform::StoredValue::new(42);
    /// let value = counter.borrow();
    /// assert_eq!(*value, 42);
    /// drop(value); // Explicitly drop when done
    /// ```
    #[must_not_suspend = "holding a StoredRef across suspend points can cause BorrowErrors"]
    pub struct StoredRef<'a, T: 'static> {
        inner: Ref<'static, T>,
        marker: PhantomData<&'a ()>,
    }

    impl<T> StoredRef<'_, T> {
        pub(crate) fn new(key: StoredValueKey) -> Result<Self, StoredValueError> {
            let inner = try_with_runtime(|runtime| {
                let stored = runtime.stored.borrow();
                let stored_value = stored.get(key).ok_or(StoredValueError::NotFound)?;
                let value = stored_value.downcast::<T>().ok_or(StoredValueError::AlreadyBorrowed)?;
                Ok(value)
            })
            .ok_or(StoredValueError::RuntimeNotFound)??;

            Ok(StoredRef { inner, marker: Default::default() })
        }
    }

    impl<'a, T> StoredRef<'a, T> {
        #[inline]
        pub fn map<U, F>(self, f: F) -> StoredRef<'a, U>
        where
            F: FnOnce(&T) -> &U,
        {
            let inner = Ref::map(self.inner, f);
            StoredRef { inner, marker: Default::default() }
        }

        #[inline]
        pub fn map_split<U, V, F>(self, f: F) -> (StoredRef<'a, U>, StoredRef<'a, V>)
        where
            F: FnOnce(&T) -> (&U, &V),
        {
            let (left, right) = Ref::map_split(self.inner, f);
            (
                StoredRef { inner: left, marker: Default::default() },
                StoredRef { inner: right, marker: Default::default() },
            )
        }

        #[inline]
        pub fn filter_map<U, F>(self, f: F) -> Result<StoredRef<'a, U>, Self>
        where
            F: FnOnce(&T) -> Option<&U>,
        {
            match Ref::filter_map(self.inner, f) {
                Ok(inner) => Ok(StoredRef { inner, marker: Default::default() }),
                Err(inner) => Err(StoredRef { inner, marker: Default::default() }),
            }
        }
    }

    /// A mutable reference to a stored value.
    ///
    /// Warning: Drop as soon as possible to allow other code to borrow the value.
    /// Consider using [`StoredValue::with()`] instead.
    ///
    /// # Example
    /// ```
    /// # slint_keyos_platform::Runtime::unsafe_init(|| ());
    /// # let counter = slint_keyos_platform::StoredValue::new(42);
    /// {
    ///     let mut value = counter.borrow_mut();
    ///     *value = 100;
    /// } // borrow is dropped at end of scope
    /// assert_eq!(counter.get(), 100);
    /// ```
    #[must_not_suspend = "holding a StoredRefMut across suspend points can cause BorrowErrors"]
    pub struct StoredRefMut<'a, T: 'static> {
        inner: std::cell::RefMut<'static, T>,
        marker: PhantomData<&'a ()>,
    }

    impl<T> StoredRefMut<'_, T> {
        pub(crate) fn new(key: StoredValueKey) -> Result<Self, StoredValueError> {
            let inner = try_with_runtime(|runtime| {
                let stored = runtime.stored.borrow();
                let stored_value = stored.get(key).ok_or(StoredValueError::NotFound)?;
                let ref_mut = stored_value.downcast_mut::<T>().ok_or(StoredValueError::AlreadyBorrowed)?;
                Ok(ref_mut)
            })
            .ok_or(StoredValueError::RuntimeNotFound)??;

            Ok(StoredRefMut { inner, marker: Default::default() })
        }
    }

    impl<'a, T> StoredRefMut<'a, T> {
        #[inline]
        pub fn map<U, F>(self, f: F) -> StoredRefMut<'a, U>
        where
            F: FnOnce(&mut T) -> &mut U,
        {
            let inner = RefMut::map(self.inner, f);
            StoredRefMut { inner, marker: Default::default() }
        }

        #[inline]
        pub fn map_split<U, V, F>(self, f: F) -> (StoredRefMut<'a, U>, StoredRefMut<'a, V>)
        where
            F: FnOnce(&mut T) -> (&mut U, &mut V),
        {
            let (left, right) = RefMut::map_split(self.inner, f);
            (
                StoredRefMut { inner: left, marker: Default::default() },
                StoredRefMut { inner: right, marker: Default::default() },
            )
        }

        #[inline]
        pub fn filter_map<U, F>(self, f: F) -> Result<StoredRefMut<'a, U>, Self>
        where
            F: FnOnce(&mut T) -> Option<&mut U>,
        {
            match RefMut::filter_map(self.inner, f) {
                Ok(inner) => Ok(StoredRefMut { inner, marker: Default::default() }),
                Err(inner) => Err(StoredRefMut { inner, marker: Default::default() }),
            }
        }
    }

    impl<T: std::fmt::Debug> std::fmt::Debug for StoredRef<'_, T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { self.inner.fmt(f) }
    }

    impl<T: std::fmt::Debug> std::fmt::Debug for StoredRefMut<'_, T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { self.inner.fmt(f) }
    }

    impl<T> std::ops::Deref for StoredRef<'_, T> {
        type Target = T;

        fn deref(&self) -> &Self::Target { self.inner.deref() }
    }

    impl<T> std::ops::Deref for StoredRefMut<'_, T> {
        type Target = T;

        fn deref(&self) -> &Self::Target { self.inner.deref() }
    }

    impl<T> std::ops::DerefMut for StoredRefMut<'_, T> {
        fn deref_mut(&mut self) -> &mut Self::Target { self.inner.deref_mut() }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum StoredValueError {
    #[error("runtime not found")]
    RuntimeNotFound,
    #[error("stored value not available")]
    NotFound,
    #[error("stored value already borrowed")]
    AlreadyBorrowed,
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "not of expected type `u32`. found `alloc::string::String`")]
fn test_error_msg() {
    let _ = super::core::Runtime::try_init(|| {});
    super::core::Runtime::unsafe_run();

    let stored = StoredValue::new("Stored String".to_owned());

    let stored_wrong_type: StoredValue<u32> =
        StoredValue { key: stored.key, ty: Default::default(), marker: Default::default() };

    stored_wrong_type.get();
}
