// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::num::NonZero;

use atsama5d27::securam::HW_SECURAM_BASE;
use securam_manager::{OsArguments, SecuramManager};
use xous::{DropDeallocate, MemoryFlags};

fn with_securam_manager<R>(f: impl FnOnce(&SecuramManager) -> R) -> Result<R, securam_manager::Error> {
    log::debug!("Mapping SECURAM");

    let securam_mem = DropDeallocate::new(
        xous::map_memory(
            Some(NonZero::new(HW_SECURAM_BASE).unwrap()),
            None,
            4096,
            MemoryFlags::W | MemoryFlags::DEV,
        )
        .expect("mapmemory"),
    );
    let securam_addr = securam_mem.as_ptr() as u32;
    log::debug!("SECURAM mapped at 0x{:08x}", securam_addr);
    Ok(f(&unsafe { SecuramManager::new(securam_addr as _) }?))
}

pub(crate) fn os_arguments() -> Result<OsArguments, securam_manager::Error> {
    with_securam_manager(|sm| sm.os_arguments().cloned())?
}
