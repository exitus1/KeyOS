// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use core::ops::{Deref, DerefMut};

use crate::{unmap_memory, MemoryRange};

/// Deallocates a memory range when it goes out of scope.
#[derive(Debug)]
pub struct DropDeallocate {
    range: MemoryRange,
    should_drop: bool,
}

impl DropDeallocate {
    pub fn new(range: MemoryRange) -> Self { Self { range, should_drop: true } }

    pub fn leak(mut self) -> MemoryRange {
        self.should_drop = false;
        self.range
    }
}

impl From<MemoryRange> for DropDeallocate {
    fn from(value: MemoryRange) -> Self { Self::new(value) }
}

impl Deref for DropDeallocate {
    type Target = MemoryRange;

    fn deref(&self) -> &Self::Target { &self.range }
}

impl DerefMut for DropDeallocate {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.range }
}

impl Drop for DropDeallocate {
    fn drop(&mut self) {
        if self.should_drop {
            unmap_memory(self.range).ok();
            self.should_drop = false;
        }
    }
}
