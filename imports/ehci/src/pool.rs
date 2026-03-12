//! Very simple memory pool implementation with intrusive flagging of whether the
//! item is used or not.
use core::{
    marker::PhantomData,
    ops::{Deref, DerefMut, Range},
    sync::atomic::{AtomicBool, Ordering},
};

use crate::{
    error::{EhciError, Result},
    registers::ListElementPointer,
};

pub(crate) struct PoolElementHandle<T> {
    virtual_address: *mut PoolElement<T>,
    physical_address: usize,
}

/// A pool item wrapper
pub struct PoolElement<T> {
    item: T,
    used: AtomicBool,
}

/// Memory pool that allows allocating elements in a given memory range
pub struct Pool<T> {
    pool: Range<*mut PoolElement<T>>,
    cursor: *mut PoolElement<T>,
    physical_address: usize,
}

impl<T> Pool<T> {
    /// Create a new memory pool from a given range
    ///
    /// `pool` is the pool's start and end addresses in virtual memory space
    /// `physical_address` is the pool's start address in physical memory space
    ///
    /// # Safety
    /// The memory range must be valid as long as the Pool object lives.
    pub unsafe fn new(
        pool: Range<*mut PoolElement<T>>,
        virt_to_phys: impl Fn(*const u8) -> usize,
    ) -> Self {
        Self {
            cursor: pool.start,
            physical_address: virt_to_phys(pool.start as *const u8),
            pool,
        }
    }

    pub(crate) fn alloc(&mut self, value: T) -> Result<PoolElementHandle<T>> {
        let original_cursor = self.cursor;
        loop {
            self.cursor = self.cursor.wrapping_add(1);
            if !self.pool.contains(&self.cursor) {
                self.cursor = self.pool.start;
            }
            if self.cursor == original_cursor {
                return Err(EhciError::OutOfPoolItems);
            }
            let element_was_used = unsafe { &*self.cursor }.used.swap(true, Ordering::SeqCst);
            if !element_was_used {
                unsafe { core::ptr::write_volatile(&mut (*self.cursor).item, value) }
                return Ok(PoolElementHandle {
                    physical_address: self.physical_address.saturating_add_signed(unsafe {
                        self.cursor
                            .cast::<u8>()
                            .offset_from(self.pool.start.cast::<u8>())
                    }),
                    virtual_address: self.cursor,
                });
            }
        }
    }
}

impl<T> PoolElementHandle<T> {
    pub fn to_controller_ptr(&self) -> ListElementPointer<T> {
        ListElementPointer {
            ptr: self.physical_address as u32,
            _phantom_data: PhantomData,
        }
    }
}

impl<T> Drop for PoolElementHandle<T> {
    fn drop(&mut self) {
        unsafe { &*self.virtual_address }
            .used
            .store(false, Ordering::SeqCst)
    }
}

impl<T> Deref for PoolElementHandle<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &(*self.virtual_address).item }
    }
}

impl<T> DerefMut for PoolElementHandle<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut (*self.virtual_address).item }
    }
}
