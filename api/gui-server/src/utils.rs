// SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::error::GuiServerError;
use crate::{DoubleBufferRegistration, VMALocation};

#[cfg(not(keyos))]
pub fn allocate_double_framebuffer(size: usize) -> Result<DoubleBufferRegistration, GuiServerError> {
    let mut disp_buf_id_bytes = [0u8; 32];
    let disp_buf_id = allocate_framebuffer(size, 0)?;
    disp_buf_id_bytes[0..disp_buf_id.len()].copy_from_slice(disp_buf_id.as_bytes());

    let mut work_buf_id_bytes = [0u8; 32];
    let work_buf_id = allocate_framebuffer(size, 0)?;
    work_buf_id_bytes[0..work_buf_id.len()].copy_from_slice(work_buf_id.as_bytes());

    Ok(DoubleBufferRegistration { disp_buf_id: disp_buf_id_bytes, work_buf_id: work_buf_id_bytes, size })
}

#[cfg(keyos)]
pub fn allocate_double_framebuffer(size: usize) -> Result<DoubleBufferRegistration, GuiServerError> {
    let size = size.next_multiple_of(0x1000);
    Ok(DoubleBufferRegistration {
        disp_buf_id: xous::map_memory(
            None,
            None,
            size,
            xous::MemoryFlags::W | xous::MemoryFlags::POPULATE | xous::MemoryFlags::PLAINTEXT,
        )?
        .as_ptr() as _,
        work_buf_id: xous::map_memory(
            None,
            None,
            size,
            xous::MemoryFlags::W | xous::MemoryFlags::POPULATE | xous::MemoryFlags::PLAINTEXT,
        )?
        .as_ptr() as _,
        size,
    })
}

#[cfg(not(keyos))]
fn allocate_framebuffer(size: usize, fill_color: u32) -> Result<String, GuiServerError> {
    let shmem =
        shared_memory::ShmemConf::new().size(size).create().map_err(|_| GuiServerError::InternalError)?;

    // Fill the buffer with provided fill color
    let slice = unsafe { core::slice::from_raw_parts_mut(shmem.as_ptr() as *mut u32, shmem.len() / 4) };
    slice.fill(fill_color);

    let id = shmem.get_os_id().to_string();
    core::mem::forget(shmem); // Otherwise the shared memory gets destroyed

    Ok(id)
}

#[cfg(not(keyos))]
pub fn fb_id_to_addr(id: &str, size: usize) -> Result<usize, GuiServerError> {
    let shmem = shared_memory::ShmemConf::new()
        .os_id(id)
        .size(size)
        .open()
        .map_err(|_| GuiServerError::InternalError)?;
    let addr = shmem.as_ptr() as usize;
    core::mem::forget(shmem); // Otherwise the shared memory gets destroyed

    Ok(addr)
}

#[cfg(keyos)]
pub fn to_vma(virt_addr: usize) -> Result<VMALocation, GuiServerError> {
    let phys_addr = xous::virt_to_phys(virt_addr)?;
    Ok(VMALocation::new(virt_addr, phys_addr))
}

#[cfg(not(keyos))]
pub fn to_vma(virt_addr: usize) -> Result<VMALocation, GuiServerError> {
    Ok(VMALocation::new(virt_addr, virt_addr))
}

#[cfg(not(keyos))]
pub fn str_from_u8_nul_utf8(utf8_src: &[u8]) -> Result<&str, GuiServerError> {
    let nul_range_end = utf8_src.iter().position(|&c| c == b'\0').unwrap_or(utf8_src.len()); // default to length if no `\0` present
    std::str::from_utf8(&utf8_src[0..nul_range_end]).map_err(|_| GuiServerError::InternalError)
}
