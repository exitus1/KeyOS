//! Special function registers backup (SFRBU).

use utralib::{utra::sfrbu::*, HW_SFRBU_BASE, *};

const KEY_PSW_MODE: u32 = 0x4BD20C << 8;

pub enum PowerSwitchBackupSource {
    Auto,
    VddBu,
    VddAna,
}

pub struct Sfrbu {
    base_addr: u32,
}

impl Default for Sfrbu {
    fn default() -> Self {
        Sfrbu::new()
    }
}

impl Sfrbu {
    #[inline]
    pub fn new() -> Self {
        Self {
            base_addr: HW_SFRBU_BASE as u32,
        }
    }

    /// Creates SFRBU instance with a different base address. Used with virtual memory
    #[inline]
    pub fn with_alt_base_addr(base_addr: u32) -> Self {
        Self { base_addr }
    }

    #[inline]
    pub fn set_power_switch_backup_source(
        &mut self,
        src: PowerSwitchBackupSource,
        secumod_auto_select: bool,
    ) {
        let mut sfrbu_csr = CSR::new(self.base_addr as *mut u32);
        let reg = KEY_PSW_MODE
            | match src {
                PowerSwitchBackupSource::Auto => sfrbu_csr.ms(PSWBUCTRL_SCTRL, 0),
                PowerSwitchBackupSource::VddBu => {
                    sfrbu_csr.ms(PSWBUCTRL_SCTRL, 1) | sfrbu_csr.ms(PSWBUCTRL_SSWCTRL, 0)
                }
                PowerSwitchBackupSource::VddAna => {
                    sfrbu_csr.ms(PSWBUCTRL_SCTRL, 1) | sfrbu_csr.ms(PSWBUCTRL_SSWCTRL, 1)
                }
            }
            | sfrbu_csr.ms(PSWBUCTRL_SMCTRL, secumod_auto_select as u32);
        sfrbu_csr.wo(PSWBUCTRL, reg);
    }

    #[inline]
    pub fn power_switch_backup_source(&self) -> PowerSwitchBackupSource {
        let sfr_csr = CSR::new(self.base_addr as *mut u32);
        if sfr_csr.rf(PSWBUCTRL_STATE) == 0 {
            PowerSwitchBackupSource::VddBu
        } else {
            PowerSwitchBackupSource::VddAna
        }
    }
}
