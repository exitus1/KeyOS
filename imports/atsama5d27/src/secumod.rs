// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundationdevices.com>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Secure Module (`SECUMOD`) driver routines.

pub use utralib::HW_SECUMOD_BASE;
use {
    bitflags::bitflags,
    utralib::{
        utra::secumod::{
            BMPR,
            CR_BACKUP,
            CR_KEY,
            CR_NORMAL,
            CR_SWPROT,
            NIDPR,
            NIEPR,
            NIMPR,
            NMPR,
            PIOBU_OUTPUT,
            PIOBU_PIOBU_AFV,
            PIOBU_PIOBU_RFV,
            PIOBU_PULLUP,
            PIOBU_SCHEDULE,
            PIOBU_SWITCH,
            RAMRDY_READY,
            SCR,
            SR,
            SYSR,
            SYSR_ERASE_DONE,
            WKPR,
        },
        CSR,
    },
};

const SECUMOD_KEY: u32 = 0x89CA;

pub struct Secumod {
    base_addr: u32,
}

pub struct SecumodProtectionRegisters {
    base_addr: u32,
}

impl Default for Secumod {
    fn default() -> Self {
        Secumod::new()
    }
}

impl Secumod {
    #[inline]
    pub fn new() -> Secumod {
        Secumod {
            base_addr: HW_SECUMOD_BASE as u32,
        }
    }

    #[inline]
    pub fn with_alt_base_addr(base_addr: u32) -> Secumod {
        Secumod { base_addr }
    }

    #[inline]
    pub fn set_normal_mode(&self) {
        let mut csr = CSR::new(self.base_addr as *mut u32);
        csr.wfo(CR_NORMAL, 1);
    }

    #[inline]
    pub fn set_backup_mode(&self) {
        let mut csr = CSR::new(self.base_addr as *mut u32);
        csr.wfo(CR_BACKUP, 1);
    }

    /// Starts the `BUSRAM4KB` and `BUREG256b` Clear content.
    #[inline]
    pub fn securam_erase(&self) {
        let mut csr = CSR::new(self.base_addr as *mut u32);
        csr.wfo(CR_SWPROT, 1);
    }

    /// When going out of CPU idle, system reset or Backup mode, this flag must be read
    /// high before accessing the secure memories.
    /// The flag remains low until any ongoing process stops.
    #[inline]
    pub fn is_ram_ready(&self) -> bool {
        let csr = CSR::new(self.base_addr as *mut u32);
        csr.rf(RAMRDY_READY) == 1
    }

    // TODO: SFT-4186
    // /// Returns the current system status of the `SECUMOD` module.
    // pub fn auxiliary_status(&self) -> SecumodAuxiliaryStatus {
    //     let csr = CSR::new(self.base_addr as *mut u32);
    //     SecumodAuxiliaryStatus::from_bits_truncate(csr.r(ASR))
    // }

    /// Returns the current system status of the `SECUMOD` module.
    #[inline]
    pub fn system_status(&self) -> SystemStatus {
        let csr = CSR::new(self.base_addr as *mut u32);
        SystemStatus::from_bits_truncate(csr.r(SYSR))
    }

    /// Returns the status of the alarms for enabled protections.
    #[inline]
    pub fn protections_status(&self) -> Protections {
        let csr = CSR::new(self.base_addr as *mut u32);
        Protections::from_bits_truncate(csr.r(SR))
    }

    /// Clears the `ERASE_DONE` status.
    #[inline]
    pub fn erase_done_clear(&self) {
        let mut csr = CSR::new(self.base_addr as *mut u32);
        csr.wfo(SYSR_ERASE_DONE, 1);
    }

    #[inline]
    pub fn with_protection_registers(&self, f: impl FnOnce(&SecumodProtectionRegisters)) {
        let mut csr = CSR::new(self.base_addr as *mut u32);
        csr.wfo(CR_KEY, SECUMOD_KEY);
        f(&SecumodProtectionRegisters {
            base_addr: self.base_addr,
        });
        csr.wfo(CR_KEY, SECUMOD_KEY);
    }

    fn configure_pio(&self, pio_num: u32, settings: PioSettings) {
        let csr = CSR::new(self.base_addr as *mut u32);
        const SECUMOD_PIOBUX_OFFSET: u32 = 0x18;
        const PIOBU_PIOBU_DYNSTAT_BIT: u32 = 20; // TODO: SFT-4186 this bit is absent in utralib
        const PIOBU_PIOBU_FILTER_BIT: u32 = 21; // TODO: SFT-4186 this bit is absent in utralib

        let piox_register = self.base_addr + SECUMOD_PIOBUX_OFFSET + pio_num * 0x04;
        let piox_register = piox_register as *mut u32;

        let mut reg_val = 0;
        reg_val |= csr.ms(PIOBU_PIOBU_AFV, settings.afv);
        reg_val |= csr.ms(PIOBU_PIOBU_RFV, settings.rfv);
        reg_val |= csr.ms(PIOBU_SWITCH, settings.switch_high as u32);
        reg_val |= csr.ms(PIOBU_SCHEDULE, settings.schedule as u32);
        reg_val |= csr.ms(PIOBU_PULLUP, settings.pull_up_setting as u32);
        reg_val |= csr.ms(PIOBU_OUTPUT, settings.is_output as u32);

        // FILTER3_5 and DYNSTAT fields exist only for even PIOBUs
        if pio_num % 2 == 0 {
            reg_val |= if settings.is_dynamic { 1 } else { 0 } << PIOBU_PIOBU_DYNSTAT_BIT;
            if let Some(filter_type) = settings.filter_type {
                reg_val |= if matches!(filter_type, FilterType::Majority5) {
                    1
                } else {
                    0
                } << PIOBU_PIOBU_FILTER_BIT;
            }
        }

        unsafe {
            piox_register.write_volatile(reg_val);
        }
    }

    /// Configures the PIOBUx pair intrusion detection
    #[inline]
    pub fn configure_protection(&self, pair: PioPair, protection: PioPairProtection) {
        match protection {
            PioPairProtection::Static(StaticProtectionSettings {
                afv,
                rfv,
                switch_high,
            }) => {
                self.configure_pio(
                    pair.0,
                    PioSettings::new_static(
                        afv.into(),
                        rfv.into(),
                        false,
                        true,
                        switch_high,
                        PullUpSetting::NoPull,
                    ),
                );
                self.configure_pio(
                    pair.0 + 1,
                    PioSettings::new_static(
                        afv.into(),
                        rfv.into(),
                        false,
                        false,
                        switch_high,
                        PullUpSetting::NoPull,
                    ),
                );
            }
            PioPairProtection::Dynamic(DynamicProtectionSettings { filter_type }) => {
                self.configure_pio(
                    pair.0,
                    PioSettings::new_dynamic(
                        false,
                        true,
                        false,
                        PullUpSetting::NoPull,
                        filter_type,
                    ),
                );
                self.configure_pio(
                    pair.0 + 1,
                    PioSettings::new_dynamic(
                        false,
                        false,
                        false,
                        PullUpSetting::NoPull,
                        filter_type,
                    ),
                );
            }
            PioPairProtection::NoProtection => {
                self.configure_pio(
                    pair.0,
                    PioSettings::new_static(0, 0, false, false, false, PullUpSetting::NoPull),
                );
                self.configure_pio(
                    pair.0 + 1,
                    PioSettings::new_static(0, 0, false, false, false, PullUpSetting::NoPull),
                );
            }
        }
    }

    /// Acknowledges the signals from provided protections.
    #[inline]
    pub fn clear_protections(&self, protections: Protections) {
        let mut csr = CSR::new(self.base_addr as *mut u32);
        csr.wo(SCR, protections.bits());
    }
}
impl SecumodProtectionRegisters {
    #[inline]
    pub fn set_normal_mode_protections(&self, protections: Protections) {
        let mut csr = CSR::new(self.base_addr as *mut u32);
        csr.wo(NMPR, protections.bits());
    }

    #[inline]
    pub fn set_backup_mode_protections(&self, protections: Protections) {
        let mut csr = CSR::new(self.base_addr as *mut u32);
        csr.wo(BMPR, protections.bits());
    }
    #[inline]
    pub fn enable_protections_interrupt(&self, protections: Protections) {
        let mut csr = CSR::new(self.base_addr as *mut u32);
        csr.wo(NIEPR, protections.bits());
    }

    #[inline]
    pub fn protections_interrupt_mask(&self) -> Protections {
        let csr = CSR::new(self.base_addr as *mut u32);
        Protections::from_bits_truncate(csr.r(NIMPR))
    }

    #[inline]
    pub fn disable_protections_interrupt(&self, protections: Protections) {
        let mut csr = CSR::new(self.base_addr as *mut u32);
        csr.wo(NIDPR, protections.bits());
    }

    /// Sets the protections that should wake up the system from Backup mode.
    #[inline]
    pub fn set_wakeup_protections(&self, protections: Protections) {
        let mut csr = CSR::new(self.base_addr as *mut u32);
        csr.wo(WKPR, protections.bits());
    }
}

bitflags! {
    #[derive(Debug, Copy, Clone)]
    pub struct SystemStatus: u32 {
        /// `ERASE_DONE`: Erasable Memories State (RW)
        ///
        /// - 0: Secure memories content has not been erased since the last clear.
        /// - 1: Secure memories content has been erased since the last clear.
        /// The user must write 1 into this bit to clear this flag.
        /// Note that not clearing this flag does not prevent the next erase processes.
        /// This flag also activates the `SECURAM` interrupt line as long as it is not cleared.
        const ERASE_DONE = 1 << 0;

        /// `ERASE_ON`: Erase Process Ongoing (RO)
        ///
        /// - 0: Erase automaton is not running.
        /// - 1: Erase automaton is currently running, memories are not accessible.
        /// When `ERASE_ON` returns to 0, `ERASE_DONE` is set after half a period of ICLK.
        const ERASE_ON   = 1 << 1;

        /// `BACKUP`: Backup Mode (RO)
        ///
        /// - 0: Normal mode active
        /// - 1: Backup mode active
        const BACKUP     = 1 << 2;

        /// `SWKUP`: `SWKUP` State (RO)
        ///
        /// - 0: No `SWKUP` signal sent since the last clear.
        /// - 1: `SWKUP` signal has been sent since the last clear.
        const SWKUP      = 1 << 3;

        /// `AUTOBKP`: Automatic Backup Mode Enabled (RO)
        ///
        /// - 0: Disabled
        /// - 1: Enabled
        const AUTOBKP    = 1 << 6;

        /// `SCRAMB`: Scrambling Enabled (RO)
        ///
        /// - 0: Disabled
        /// - 1: Enabled
        const SCRAMB     = 1 << 7;
    }
}

bitflags! {
    #[derive(Debug, Copy, Clone)]
    pub struct Protections: u32 {
        const SHLDM = 1 << 0;
        const DBLFM = 1 << 1;

        /// `TEST` pin monitor
        const TST = 1 << 2;

        /// `JTAG` pins monitor
        const JTAG = 1 << 3;

        const TPML = 1 << 6;
        const TPMH = 1 << 7;

        const VDDBUL = 1 << 10;
        const VDDBUH = 1 << 11;

        /// `DETx`: PIOBUx intrusion detector.
        const DET0 = 1 << 16;
        const DET1 = 1 << 17;
        const DET2 = 1 << 18;
        const DET3 = 1 << 19;
        const DET4 = 1 << 20;
        const DET5 = 1 << 21;
        const DET6 = 1 << 22;
        const DET7 = 1 << 23;
    }
}

/* TODO: SFT-4186
bitflags! {
    #[derive(Debug, Copy, Clone)]
    pub struct SecumodAuxiliaryStatus: u32 {
        /// `JTAGSEL`, `CA5` tap response or `CA5` debug acknowledge is the cause of `JTAG` flag in `SR`.
        const JTAG = 1 << 4;

        /// `TCK`/`TMS` activity detected is the cause of `JTAG` flag in `SR`
        const TCK  = 1 << 5;
    }
}
*/

#[derive(Debug)]
pub enum PinNumber {
    PioBu0 = 0,
    PioBu1 = 1,
    PioBu2 = 2,
    PioBu3 = 3,
    PioBu4 = 4,
    PioBu5 = 5,
    PioBu6 = 6,
    PioBu7 = 7,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum PullUpSetting {
    NoPull = 0b00,
    PullUp = 0b01,
    PullDown = 0b10,
}

#[derive(Debug)]
pub enum IoType {
    Input = 0,
    Output,
}

#[derive(Debug)]
pub enum ProtectionType {
    Static = 0,
    Dynamic,
}

#[derive(Debug, Copy, Clone)]
pub enum FilterType {
    Majority3 = 0,
    Majority5,
}

#[derive(Debug, Copy, Clone)]
pub enum FilterValue {
    NoStaticProtection = 0,
    Max2 = 1,
    Max4 = 2,
    Max8 = 3,
    Max16 = 4,
    Max32 = 5,
    Max64 = 6,
    Max128 = 7,
    Max256 = 8,
    Max512 = 9,
}

impl From<FilterValue> for u32 {
    fn from(value: FilterValue) -> u32 {
        value as u32
    }
}

#[derive(Debug)]
struct PioSettings {
    afv: u32,
    rfv: u32,
    schedule: bool,
    is_dynamic: bool,
    is_output: bool,
    switch_high: bool,
    pull_up_setting: PullUpSetting,
    filter_type: Option<FilterType>,
}

impl PioSettings {
    #[inline]
    pub fn new_static(
        afv: u32,
        rfv: u32,
        schedule: bool,
        is_output: bool,
        switch_high: bool,
        pull_up_setting: PullUpSetting,
    ) -> Self {
        Self {
            afv,
            rfv,
            schedule,
            is_dynamic: false,
            is_output,
            switch_high,
            pull_up_setting,
            filter_type: None,
        }
    }

    #[inline]
    pub fn new_dynamic(
        schedule: bool,
        is_output: bool,
        switch_high: bool,
        pull_up_setting: PullUpSetting,
        filter_type: impl Into<Option<FilterType>>,
    ) -> Self {
        Self {
            afv: 0,
            rfv: 0,
            schedule,
            is_dynamic: true,
            is_output,
            switch_high,
            pull_up_setting,
            filter_type: filter_type.into(),
        }
    }
}

#[derive(Debug)]
pub struct StaticProtectionSettings {
    afv: FilterValue,
    rfv: FilterValue,
    switch_high: bool,
}

impl StaticProtectionSettings {
    #[inline]
    pub fn new(afv: FilterValue, rfv: FilterValue, switch_high: bool) -> Self {
        StaticProtectionSettings {
            afv,
            rfv,
            switch_high,
        }
    }
}

#[derive(Debug)]
pub struct DynamicProtectionSettings {
    filter_type: FilterType,
}

impl DynamicProtectionSettings {
    #[inline]
    pub fn new(filter_type: FilterType) -> Self {
        DynamicProtectionSettings { filter_type }
    }
}

#[derive(Debug)]
pub enum PioPairProtection {
    Static(StaticProtectionSettings),
    Dynamic(DynamicProtectionSettings),
    NoProtection,
}

pub struct PioPair(u32);

impl PioPair {
    /// `PIOBU0` and `PIOBU1`
    #[inline]
    pub fn new_0_1() -> PioPair {
        PioPair(0)
    }

    /// `PIOBU2` and `PIOBU3`
    #[inline]
    pub fn new_2_3() -> PioPair {
        PioPair(2)
    }

    /// `PIOBU4` and `PIOBU5`
    #[inline]
    pub fn new_4_5() -> PioPair {
        PioPair(4)
    }

    /// `PIOBU6` and `PIOBU7`
    #[inline]
    pub fn new_6_7() -> PioPair {
        PioPair(6)
    }
}
