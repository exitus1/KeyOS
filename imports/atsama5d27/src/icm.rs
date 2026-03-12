// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundationdevices.com>
// SPDX-License-Identifier: MIT OR Apache-2.0

//! ICM (Integrity Check Monitor) driver

use {
    bitflags::bitflags,
    utralib::{utra::icm, CSR, HW_ICM_BASE},
};

/// Hardware alignment constraints implied by DSCR/HASH masking.
pub const ICM_DSCR_ALIGN: u32 = 64; // DSCR requires 64B alignment
pub const ICM_HASH_ALIGN: u32 = 128; // HASH requires 128B alignment
pub const ICM_BLOCK_BYTES: u32 = 64; // RCTRL is in 512-bit blocks (64 bytes)

bitflags! {
    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub struct Region: u8 {
        const R0 = 1 << 0;
        const R1 = 1 << 1;
        const R2 = 1 << 2;
        const R3 = 1 << 3;
    }

    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub struct Interrupts: u32 {
        /// Undefined Register Access Detection Interrupt
        const URAD = 1 << 0;
        /// Region Status Updated Interrupt
        const RSU  = 1 << 1;
        /// Region End bit Condition Detected Interrupt
        const REC  = 1 << 2;
        /// Region Wrap Condition Detected Interrupt
        const RWC  = 1 << 3;
        /// Region Bus Error Detected Interrupt
        const RBE  = 1 << 4;
        /// Region Digest Mismatch Interrupt
        const RDM  = 1 << 5;
        /// Region Hash Completed Interrupt
        const RHC  = 1 << 6;
    }
}

#[derive(Debug)]
pub enum UradStatus {
    /// Unspecified structure member set to one detected when the descriptor is loaded
    UnspecStructMember = 0,

    /// ICM_CFG modified during active monitoring
    IcmCfgModified = 1,

    /// ICM_DSCR modified during active monitoring
    IcmDscrModified = 2,

    /// ICM_HASH modified during active monitoring
    IcmHashModified = 3,

    /// Write-only register read access
    ReadAccess = 4,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum IcmError {
    DescPhysUnaligned,
    HashPhysUnaligned,
    RegionPhysUnaligned,
    LengthNotMultipleOf64,
    ZeroLength,
    InvalidRegionId,
}

/// Region index 0..3 (maps to descriptor slot / RID).
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum RegionId {
    R0 = 0,
    R1 = 1,
    R2 = 2,
    R3 = 3,
}

impl RegionId {
    #[inline]
    pub const fn mask(self) -> Region {
        match self {
            RegionId::R0 => Region::R0,
            RegionId::R1 => Region::R1,
            RegionId::R2 => Region::R2,
            RegionId::R3 => Region::R3,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Algorithm {
    Sha1 = 0,
    Sha256 = 1,
    Sha224 = 4,
}

/// One descriptor is 16 bytes.
#[derive(Debug, Copy, Clone, Default)]
#[repr(C)]
pub struct IcmRegionDescriptor {
    pub raddr: u32,
    pub rcfg: u32,
    pub rctrl: u32,
    pub rnext: u32,
}

/// Main list: 4 descriptors, base must be 64B aligned for DSCR.
#[repr(C, align(64))]
pub struct IcmDescriptorList {
    pub desc: [IcmRegionDescriptor; 4],
}

#[derive(Debug)]
pub struct IcmStatus {
    pub rmdis: Region,
    pub raw_rmdis: Region,
    pub enabled: bool,
}

pub struct Icm {
    base_addr: u32,
}

impl Icm {
    pub fn new() -> Self {
        Self {
            base_addr: HW_ICM_BASE as u32,
        }
    }

    pub fn with_alt_base_addr(base_addr: u32) -> Self {
        Self { base_addr }
    }

    fn csr(&self) -> CSR<u32> {
        CSR::new(self.base_addr as *mut u32)
    }

    pub fn reset(&self) {
        let mut csr = self.csr();
        csr.wfo(icm::CTRL_SWRST, 1);
    }

    /// Enables or disables the whole ICM block.
    pub fn set_enable(&self, enable: bool) {
        let mut csr = self.csr();
        if enable {
            csr.wfo(icm::CTRL_ENABLE, 1);
        } else {
            csr.wfo(icm::CTRL_DISABLE, 1);
        }
    }

    pub fn set_automatic_monitoring_mode(&self, enable: bool) {
        let mut csr = self.csr();
        csr.rmwf(icm::CFG_ASCD, enable as u32);
    }

    pub fn set_double_buffering(&self, enable: bool) {
        let mut csr = self.csr();
        csr.rmwf(icm::CFG_DUALBUFF, enable as u32);
    }

    pub fn set_eom_disabled(&self, disabled: bool) {
        let mut csr = self.csr();
        csr.rmwf(icm::CFG_EOMDIS, disabled as u32);
    }

    /// Enables monitoring for provided region(s).
    pub fn monitoring_enable(&self, region: Region) {
        let mut csr = self.csr();
        csr.wfo(icm::CTRL_RMEN, region.bits() as u32);
    }

    /// Disables monitoring for provided region(s).
    pub fn monitoring_disable(&self, region: Region) {
        let mut csr = self.csr();
        csr.wfo(icm::CTRL_RMDIS, region.bits() as u32);
    }

    /// Recompute digest of provided region(s).
    /// Only available if monitoring is disabled.
    pub fn rehash(&self, region: Region) {
        let mut csr = self.csr();
        csr.wfo(icm::CTRL_REHASH, region.bits() as u32);
    }

    pub fn status(&self) -> IcmStatus {
        let csr = self.csr();

        IcmStatus {
            rmdis: Region::from_bits_truncate(csr.rf(icm::SR_RMDIS) as u8),
            raw_rmdis: Region::from_bits_truncate(csr.rf(icm::SR_RAWRMDIS) as u8),
            enabled: csr.rf(icm::SR_ENABLE) != 0,
        }
    }

    pub fn enable_interrupts(&self, region: Region, interrupts: Interrupts) {
        let mut csr = self.csr();

        if interrupts.contains(Interrupts::URAD) {
            csr.wfo(icm::IER_URAD, 1);
        }
        if interrupts.contains(Interrupts::RSU) {
            csr.wfo(icm::IER_RSU, region.bits() as u32);
        }
        if interrupts.contains(Interrupts::REC) {
            csr.wfo(icm::IER_REC, region.bits() as u32);
        }
        if interrupts.contains(Interrupts::RWC) {
            csr.wfo(icm::IER_RWC, region.bits() as u32);
        }
        if interrupts.contains(Interrupts::RBE) {
            csr.wfo(icm::IER_RBE, region.bits() as u32);
        }
        if interrupts.contains(Interrupts::RDM) {
            csr.wfo(icm::IER_RDM, region.bits() as u32);
        }
        if interrupts.contains(Interrupts::RHC) {
            csr.wfo(icm::IER_RHC, region.bits() as u32);
        }
    }

    pub fn disable_interrupts(&self, region: Region, interrupts: Interrupts) {
        let mut csr = self.csr();

        if interrupts.contains(Interrupts::URAD) {
            csr.wfo(icm::IDR_URAD, 1);
        }
        if interrupts.contains(Interrupts::RSU) {
            csr.wfo(icm::IDR_RSU, region.bits() as u32);
        }
        if interrupts.contains(Interrupts::REC) {
            csr.wfo(icm::IDR_REC, region.bits() as u32);
        }
        if interrupts.contains(Interrupts::RWC) {
            csr.wfo(icm::IDR_RWC, region.bits() as u32);
        }
        if interrupts.contains(Interrupts::RBE) {
            csr.wfo(icm::IDR_RBE, region.bits() as u32);
        }
        if interrupts.contains(Interrupts::RDM) {
            csr.wfo(icm::IDR_RDM, region.bits() as u32);
        }
        if interrupts.contains(Interrupts::RHC) {
            csr.wfo(icm::IDR_RHC, region.bits() as u32);
        }
    }

    /// Returns the status of the ICM interrupts for a given region.
    pub fn interrupt_status(&self, region: Region) -> Interrupts {
        let csr = self.csr();
        let isr = csr.r(icm::ISR);
        let r = region.bits() as u32;

        let mut out = Interrupts::empty();
        if (isr >> 0) & r != 0 {
            out |= Interrupts::RHC;
        }
        if (isr >> 4) & r != 0 {
            out |= Interrupts::RDM;
        }
        if (isr >> 8) & r != 0 {
            out |= Interrupts::RBE;
        }
        if (isr >> 12) & r != 0 {
            out |= Interrupts::RWC;
        }
        if (isr >> 16) & r != 0 {
            out |= Interrupts::REC;
        }
        if (isr >> 20) & r != 0 {
            out |= Interrupts::RSU;
        }
        if (isr >> 24) & 0x1 != 0 {
            out |= Interrupts::URAD;
        }
        out
    }

    pub fn set_descriptor_area_address(&self, addr_phys: u32) -> Result<(), IcmError> {
        if addr_phys & (ICM_DSCR_ALIGN - 1) != 0 {
            return Err(IcmError::DescPhysUnaligned);
        }
        let mut csr = self.csr();
        csr.wo(icm::DSCR, addr_phys);
        Ok(())
    }

    pub fn set_hash_area_address(&self, addr_phys: u32) -> Result<(), IcmError> {
        if addr_phys & (ICM_HASH_ALIGN - 1) != 0 {
            return Err(IcmError::HashPhysUnaligned);
        }
        let mut csr = self.csr();
        csr.wo(icm::HASH, addr_phys);
        Ok(())
    }

    pub fn set_secondary_list_branching_disable(&self, disable: bool) {
        let mut csr = self.csr();
        csr.rmwf(icm::CFG_SLBDIS, disable as u32);
    }

    pub fn set_algorithm(&self, algorithm: Algorithm) {
        let mut csr = self.csr();
        csr.rmwf(icm::CFG_UALGO, algorithm as u32);
    }

    /// Configure ONE region descriptor for automatic monitoring of a contiguous physical
    /// range.
    ///
    /// Arguments:
    /// - `desc_addr`: CPU-mapped virtual address of a `IcmDescriptorList`
    /// - `hash_addr`: physical HASH base (must be 128B aligned)
    /// - `region_phys`: physical start of the region to monitor
    /// - `len_bytes`: size of a region (must be multiple of 64 bytes)
    /// - `v2p`: virtual->physical for `desc_addr`
    /// - `cache_maintenance`: clean descriptor writes
    ///
    /// After calling this, ICM is enabled and monitoring for that RID is enabled.
    pub fn start_monitoring_contiguous_region(
        &self,
        rid: RegionId,
        desc_addr: u32,
        region_phys: u32,
        len_bytes: u32,
        cache_maintenance: impl FnOnce(),
    ) -> Result<(), IcmError> {
        if !len_bytes.is_multiple_of(ICM_BLOCK_BYTES) {
            return Err(IcmError::LengthNotMultipleOf64);
        }
        if region_phys & (ICM_BLOCK_BYTES - 1) != 0 {
            return Err(IcmError::RegionPhysUnaligned);
        }

        let blocks = len_bytes / ICM_BLOCK_BYTES;

        const RCFG_WRAP: u32 = 1 << 1;

        const RCFG_RHIEN_DIS: u32 = 1 << 4;
        const RCFG_WCIEN_DIS: u32 = 1 << 7;
        const RCFG_ECIEN_DIS: u32 = 1 << 8;
        const RCFG_SUIEN_DIS: u32 = 1 << 9;

        let rcfg = ((Algorithm::Sha256 as u32) << 12)
            | RCFG_WRAP
            | RCFG_RHIEN_DIS
            | RCFG_WCIEN_DIS
            | RCFG_ECIEN_DIS
            | RCFG_SUIEN_DIS;

        let list = unsafe { &mut *(desc_addr as *mut IcmDescriptorList) };
        let idx = rid as usize;

        list.desc[idx] = IcmRegionDescriptor {
            raddr: region_phys,
            rcfg,
            rctrl: blocks, // TRSIZE in 64-byte blocks
            rnext: 0,
        };

        cache_maintenance();

        self.monitoring_enable(rid.mask());

        Ok(())
    }

    /// Should be called before modifying the data under monitoring.
    pub fn stop_monitoring_region(&self, rid: RegionId) {
        self.monitoring_disable(rid.mask());
    }

    /// Should be called after modifying the data under monitoring.
    pub fn resume_monitoring_region(&self, rid: RegionId) {
        // REHASH is only valid when monitoring is disabled, so keep this ordering.
        self.rehash(rid.mask());
        self.monitoring_enable(rid.mask());
    }

    /// Set bus burden control, the higher the value, the bigger the delay between the
    /// block transfers. 2^bcc (up to 2^16 = 32768) cycles are inserted between two
    /// consecutive block transfers.
    pub fn set_bus_burden(&self, bbc: u32) {
        let mut csr = CSR::new(self.base_addr as *mut u32);
        csr.rmwf(icm::CFG_BBC, bbc & 0xf);
    }

    pub fn urad_status(&self) -> UradStatus {
        match self.csr().rf(icm::UASR_URAT) {
            0 => UradStatus::UnspecStructMember,
            1 => UradStatus::IcmCfgModified,
            2 => UradStatus::IcmDscrModified,
            3 => UradStatus::IcmHashModified,
            4 => UradStatus::ReadAccess,
            _ => unreachable!(),
        }
    }
}
