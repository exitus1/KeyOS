// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]
#![cfg(keyos)]

mod bindings;
mod macros;

use {
    bindings::*,
    core::{
        ptr::addr_of,
        sync::atomic::{AtomicBool, Ordering},
    },
    xous::{MemoryRange, PID, TID},
};

const KEYOS_SYSCALL_NUMBER_TO_EVT_ID_OFFSET: u32 = 40;

pub struct SystemView;

static INITIALIZED: AtomicBool = AtomicBool::new(false);

impl SystemView {
    pub const fn new() -> SystemView { SystemView }

    pub fn init(rtt_up_buffer: &mut [u8], rtt_down_buffer: &mut [u8], sys_freq: u32, cpu_freq: u32) {
        let rtt_up_buffer_ptr = rtt_up_buffer.as_mut_ptr() as *mut _;
        let rtt_up_buffer_len = rtt_up_buffer.len() as u32;

        let rtt_down_buffer_ptr = rtt_down_buffer.as_mut_ptr() as *mut _;
        let rtt_down_buffer_len = rtt_down_buffer.len() as u32;

        unsafe {
            SEGGER_SYSVIEW_Init(
                sys_freq,
                cpu_freq,
                addr_of!(OS_API),
                Some(send_system_description),
                rtt_up_buffer_ptr,
                rtt_up_buffer_len,
                rtt_down_buffer_ptr,
                rtt_down_buffer_len,
            );
            INITIALIZED.store(true, Ordering::SeqCst);
        }
    }

    pub fn send_system_description(desc: &str) {
        if !INITIALIZED.load(Ordering::SeqCst) {
            return;
        }

        unsafe {
            SEGGER_SYSVIEW_SendSysDesc(desc.as_ptr());
        }
    }
}

impl SystemView {
    pub fn wait_for_recorder() {
        if !INITIALIZED.load(Ordering::SeqCst) {
            return;
        }

        unsafe {
            while SEGGER_SYSVIEW_IsStarted() == 0 {
                armv7::asm::nop();
            }
        }
    }

    pub fn task_new(id: u32) {
        if !INITIALIZED.load(Ordering::SeqCst) {
            return;
        }

        unsafe {
            SEGGER_SYSVIEW_OnTaskCreate(id);
        }
    }

    pub fn thread_send_info(pid: PID, tid: TID, name: &str, stack: MemoryRange) {
        // Copy into null terminated buffer
        const MAX_LEN: usize = 128;
        let mut name_buf = [0u8; MAX_LEN];
        let name_len = name.len().min(MAX_LEN);
        name_buf[..name_len].copy_from_slice(&name.as_bytes()[..name_len]);
        name_buf[name_len] = 0x00;

        if !INITIALIZED.load(Ordering::SeqCst) {
            return;
        }

        let info = SEGGER_SYSVIEW_TASKINFO {
            TaskID: pid_tid_to_id(pid, tid),
            sName: name_buf.as_ptr(),
            Prio: 0,
            StackBase: stack.as_ptr() as u32,
            StackSize: stack.len() as u32,
        };
        unsafe {
            SEGGER_SYSVIEW_SendTaskInfo(&info);
        }
    }

    pub fn task_terminate(id: u32) {
        if !INITIALIZED.load(Ordering::SeqCst) {
            return;
        }

        unsafe {
            SEGGER_SYSVIEW_OnTaskTerminate(id);
        }
    }

    pub fn task_exec_begin(id: u32) {
        if !INITIALIZED.load(Ordering::SeqCst) {
            return;
        }

        unsafe {
            SEGGER_SYSVIEW_OnTaskStartExec(id);
        }
    }

    pub fn task_exec_end() {
        if !INITIALIZED.load(Ordering::SeqCst) {
            return;
        }

        unsafe {
            SEGGER_SYSVIEW_OnTaskStopExec();
        }
    }

    pub fn task_ready_begin(id: u32) {
        if !INITIALIZED.load(Ordering::SeqCst) {
            return;
        }

        unsafe {
            SEGGER_SYSVIEW_OnTaskStartReady(id);
        }
    }

    pub fn task_ready_end(id: u32, cause: u32) {
        if !INITIALIZED.load(Ordering::SeqCst) {
            return;
        }

        unsafe {
            SEGGER_SYSVIEW_OnTaskStopReady(id, cause);
        }
    }

    pub fn system_idle() {
        if !INITIALIZED.load(Ordering::SeqCst) {
            return;
        }

        unsafe {
            SEGGER_SYSVIEW_OnIdle();
        }
    }

    pub fn isr_enter() {
        if !INITIALIZED.load(Ordering::SeqCst) {
            return;
        }

        unsafe {
            SEGGER_SYSVIEW_RecordEnterISR();
        }
    }

    pub fn isr_exit() {
        if !INITIALIZED.load(Ordering::SeqCst) {
            return;
        }

        unsafe {
            SEGGER_SYSVIEW_RecordExitISR();
        }
    }

    pub fn isr_exit_to_scheduler() {
        if !INITIALIZED.load(Ordering::SeqCst) {
            return;
        }

        unsafe {
            SEGGER_SYSVIEW_RecordExitISRToScheduler();
        }
    }

    pub fn marker(id: u32) {
        if !INITIALIZED.load(Ordering::SeqCst) {
            return;
        }

        unsafe {
            SEGGER_SYSVIEW_Mark(id);
        }
    }

    pub fn marker_begin(id: u32) {
        if !INITIALIZED.load(Ordering::SeqCst) {
            return;
        }

        unsafe {
            SEGGER_SYSVIEW_MarkStart(id);
        }
    }

    pub fn marker_end(id: u32) {
        if !INITIALIZED.load(Ordering::SeqCst) {
            return;
        }

        unsafe {
            SEGGER_SYSVIEW_MarkStop(id);
        }
    }

    pub fn trace_syscall(args: &[usize; 8]) {
        if !INITIALIZED.load(Ordering::SeqCst) {
            return;
        }

        let id = KEYOS_SYSCALL_NUMBER_TO_EVT_ID_OFFSET + args[0] as u32;
        let args = &args[1..8];

        unsafe {
            SEGGER_SYSVIEW_RecordU32x7(
                id,
                args[0] as u32,
                args[1] as u32,
                args[2] as u32,
                args[3] as u32,
                args[4] as u32,
                args[5] as u32,
                args[6] as u32,
            );
        }
    }

    pub fn trace_syscall_result(args: &[usize; 8], res: usize) {
        if !INITIALIZED.load(Ordering::SeqCst) {
            return;
        }

        let id = KEYOS_SYSCALL_NUMBER_TO_EVT_ID_OFFSET + args[0] as u32;

        unsafe {
            SEGGER_SYSVIEW_RecordEndCallU32(id, res as u32);
        }
    }

    pub fn trace_internal_api(api_fn: InternalApiFn, args: &[usize; 8]) {
        if !INITIALIZED.load(Ordering::SeqCst) {
            return;
        }

        let id = api_fn as u32;

        unsafe {
            SEGGER_SYSVIEW_RecordU32x8(
                id,
                args[0] as u32,
                args[1] as u32,
                args[2] as u32,
                args[3] as u32,
                args[4] as u32,
                args[5] as u32,
                args[6] as u32,
                args[7] as u32,
            );
        }
    }

    pub fn trace_internal_api_result(api_fn: InternalApiFn, res: xous::Error) {
        if !INITIALIZED.load(Ordering::SeqCst) {
            return;
        }

        let id = api_fn as u32;

        unsafe {
            SEGGER_SYSVIEW_RecordEndCallU32(id, res as u32);
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum InternalApiFn {
    AllocPage = 256,
}

extern "C" {
    fn send_system_description();
    fn send_task_list();
    fn os_get_time() -> u64;
    fn get_timestamp() -> u32;
    fn get_current_isr() -> u32;
}

static mut OS_API: SEGGER_SYSVIEW_OS_API =
    SEGGER_SYSVIEW_OS_API { pfGetTime: Some(os_get_time), pfSendTaskList: Some(send_task_list) };

#[no_mangle]
unsafe extern "C" fn SEGGER_SYSVIEW_X_GetTimestamp() -> u32 { get_timestamp() }

#[no_mangle]
unsafe extern "C" fn SEGGER_SYSVIEW_X_GetInterruptId() -> u32 { get_current_isr() }

pub fn pid_tid_to_id(pid: PID, tid: TID) -> u32 { (pid.get() as u32) << 8 | tid as u32 }
