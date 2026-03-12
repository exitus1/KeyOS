/// Flags to be passed to the MapMemory struct.
/// Note that it is an error to have memory be
/// writable and not readable.
#[derive(Copy, PartialEq, Eq, Clone, PartialOrd, Ord, Hash, Debug)]
pub struct MemoryFlags {
    bits: usize,
}

// Don't reorder my consts pls, rustfmt.
#[rustfmt::skip]
impl MemoryFlags {
    const FLAGS_ALL: usize = 0b0111_1101;
    /// Immediately allocate this memory.  Otherwise, it will be demand-paged.
    /// Guarantees that the allocated area is physically contiguous.
    /// Only applicable when phys=0
    pub const POPULATE: Self = Self { bits: 0b0000_0001 };

    // No `R` flag because on ARM a page can only be read-only, read-write, or outright no-access,
    // at which point we might as well just not map the page.
    // The old R flag at 0b0010 is currently unused.

    /// Allow the CPU to write to this page.
    pub const W: Self = Self { bits: 0b0000_0100 };
    /// Allow the CPU to execute from this page.
    pub const X: Self = Self { bits: 0b0000_1000 };
    /// Marks the page as the 'device' page for on-chip peripherals.
    /// This implies that no caching is applicable to these pages and all accesses are strongly-ordered.
    pub const DEV: Self = Self { bits: 0b0001_0000 };
    /// Marks the page as not suitable for caching.
    /// Reads and writes go straight to memory, but ordering is not guaranteed, and speculative
    /// accesses are possible. Should be combined with fence operations.
    pub const NO_CACHE: Self = Self { bits: 0b0010_0000 };
    /// Marks the page as non-encrypted.
    /// The physical memory of the page is not encrypted.
    /// Used to support peripheral DMA that can't access AESB pages (LCDC, ISC, SDMMC)
    pub const PLAINTEXT: Self = Self { bits: 0b0100_0000 };

    pub fn bits(&self) -> usize { self.bits }

    pub fn from_bits(raw: usize) -> MemoryFlags {
        MemoryFlags { bits: raw & Self::FLAGS_ALL}
    }

    pub fn is_empty(&self) -> bool { self.bits == 0 }

    pub fn empty() -> MemoryFlags { MemoryFlags { bits: 0 } }

    pub fn is_set(&self, mask: MemoryFlags) -> bool {
        (*self & mask) == mask
    }

    #[allow(dead_code)]
    pub fn print(&self, mut print_fn: impl FnMut(&str)) {
        print_fn("R");
        if !(*self & MemoryFlags::W).is_empty() {
            print_fn("W");
        }
        if !(*self & MemoryFlags::X).is_empty() {
            print_fn("X");
        }
        if !(*self & MemoryFlags::DEV).is_empty() {
            print_fn("D");
        }
        if !(*self & MemoryFlags::NO_CACHE).is_empty() {
            print_fn("nC");
        }
        if !(*self & MemoryFlags::PLAINTEXT).is_empty() {
            print_fn("P");
        }
    }
}

// impl core::fmt::Debug for MemoryFlags {
//     fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
//         // Iterate over the valid flags
//         let mut first = true;
//         for (name, _) in self.iter() {
//             if !first {
//                 f.write_str(" | ")?;
//             }

//             first = false;
//             f.write_str(name)?;
//         }

//         // Append any extra bits that correspond to flags to the end of the format
//         let extra_bits = self.bits & !Self::all().bits();

//         // if extra_bits != <$T as Bits>::EMPTY {
//         //     if !first {
//         //         f.write_str(" | ")?;
//         //     }
//         //     first = false;
//         //     core::write!(f, "{:#x}", extra_bits)?;
//         // }

//         if first {
//             f.write_str("(empty)")?;
//         }

//         core::fmt::Result::Ok(())
//     }
// }

impl core::fmt::Binary for MemoryFlags {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result { core::fmt::Binary::fmt(&self.bits, f) }
}

impl core::fmt::Octal for MemoryFlags {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result { core::fmt::Octal::fmt(&self.bits, f) }
}

impl core::fmt::LowerHex for MemoryFlags {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::fmt::LowerHex::fmt(&self.bits, f)
    }
}

impl core::fmt::UpperHex for MemoryFlags {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::fmt::UpperHex::fmt(&self.bits, f)
    }
}

impl core::ops::BitOr for MemoryFlags {
    type Output = Self;

    /// Returns the union of the two sets of flags.
    #[inline]
    fn bitor(self, other: MemoryFlags) -> Self { Self { bits: self.bits | other.bits } }
}

impl core::ops::BitOrAssign for MemoryFlags {
    /// Adds the set of flags.
    #[inline]
    fn bitor_assign(&mut self, other: Self) { self.bits |= other.bits; }
}

impl core::ops::BitXor for MemoryFlags {
    type Output = Self;

    /// Returns the left flags, but with all the right flags toggled.
    #[inline]
    fn bitxor(self, other: Self) -> Self { Self { bits: self.bits ^ other.bits } }
}

impl core::ops::BitXorAssign for MemoryFlags {
    /// Toggles the set of flags.
    #[inline]
    fn bitxor_assign(&mut self, other: Self) { self.bits ^= other.bits; }
}

impl core::ops::BitAnd for MemoryFlags {
    type Output = Self;

    /// Returns the intersection between the two sets of flags.
    #[inline]
    fn bitand(self, other: Self) -> Self { Self { bits: self.bits & other.bits } }
}

impl core::ops::BitAndAssign for MemoryFlags {
    /// Disables all flags disabled in the set.
    #[inline]
    fn bitand_assign(&mut self, other: Self) { self.bits &= other.bits; }
}

impl core::ops::Sub for MemoryFlags {
    type Output = Self;

    /// Returns the set difference of the two sets of flags.
    #[inline]
    fn sub(self, other: Self) -> Self { Self { bits: self.bits & !other.bits } }
}

impl core::ops::SubAssign for MemoryFlags {
    /// Disables all flags enabled in the set.
    #[inline]
    fn sub_assign(&mut self, other: Self) { self.bits &= !other.bits; }
}

impl core::ops::Not for MemoryFlags {
    type Output = Self;

    /// Returns the complement of this set of flags.
    #[inline]
    fn not(self) -> Self { Self { bits: !self.bits } & MemoryFlags { bits: 15 } }
}
