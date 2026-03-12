// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused)]

pub const SEGGER_PRINTF_FLAG_ADJLEFT: u32 = 1;
pub const SEGGER_PRINTF_FLAG_SIGNFORCE: u32 = 2;
pub const SEGGER_PRINTF_FLAG_SIGNSPACE: u32 = 4;
pub const SEGGER_PRINTF_FLAG_PRECEED: u32 = 8;
pub const SEGGER_PRINTF_FLAG_ZEROPAD: u32 = 16;
pub const SEGGER_PRINTF_FLAG_NEGATIVE: u32 = 32;
pub const SEGGER_RTT_MAX_NUM_UP_BUFFERS: u32 = 2;
pub const SEGGER_RTT_MAX_NUM_DOWN_BUFFERS: u32 = 3;
pub const BUFFER_SIZE_UP: u32 = 1024;
pub const BUFFER_SIZE_DOWN: u32 = 16;
pub const SEGGER_RTT_PRINTF_BUFFER_SIZE: u32 = 64;
pub const SEGGER_RTT_MEMCPY_USE_BYTELOOP: u32 = 0;
pub const SEGGER_RTT_MAX_INTERRUPT_PRIORITY: u32 = 32;
pub const SEGGER_SYSVIEW_CORE_OTHER: u32 = 0;
pub const SEGGER_SYSVIEW_CORE_CM0: u32 = 1;
pub const SEGGER_SYSVIEW_CORE_CM3: u32 = 2;
pub const SEGGER_SYSVIEW_CORE_RX: u32 = 3;
pub const SEGGER_SYSVIEW_CORE: u32 = 0;
pub const SEGGER_SYSVIEW_APP_NAME: &[u8; 31] = b"SystemView-enabled Application\0";
pub const SEGGER_SYSVIEW_DEVICE_NAME: &[u8; 17] = b"undefined device\0";
pub const SEGGER_SYSVIEW_TIMESTAMP_BITS: u32 = 20;
pub const SEGGER_SYSVIEW_RTT_CHANNEL: u32 = 0;
pub const SEGGER_SYSVIEW_RTT_BUFFER_SIZE: u32 = 1024;
pub const SEGGER_SYSVIEW_CPU_CACHE_LINE_SIZE: u32 = 0;
pub const SEGGER_SYSVIEW_ID_BASE: u32 = 0;
pub const SEGGER_SYSVIEW_ID_SHIFT: u32 = 0;
pub const SEGGER_SYSVIEW_MAX_ARGUMENTS: u32 = 16;
pub const SEGGER_SYSVIEW_MAX_STRING_LEN: u32 = 128;
pub const SEGGER_SYSVIEW_SUPPORT_LONG_ID: u32 = 1;
pub const SEGGER_SYSVIEW_SUPPORT_LONG_DATA: u32 = 0;
pub const SEGGER_SYSVIEW_PRINTF_IMPLICIT_FORMAT: u32 = 0;
pub const SEGGER_SYSVIEW_USE_INTERNAL_RECORDER: u32 = 0;
pub const SEGGER_SYSVIEW_CAN_RESTART: u32 = 1;
pub const SEGGER_SYSVIEW_START_ON_INIT: u32 = 0;
pub const SEGGER_SYSVIEW_USE_STATIC_BUFFER: u32 = 1;
pub const SEGGER_SYSVIEW_POST_MORTEM_MODE: u32 = 0;
pub const SEGGER_SYSVIEW_SYNC_PERIOD_SHIFT: u32 = 8;
pub const SEGGER_SYSVIEW_MAJOR: u32 = 3;
pub const SEGGER_SYSVIEW_MINOR: u32 = 32;
pub const SEGGER_SYSVIEW_REV: u32 = 0;
pub const SEGGER_SYSVIEW_VERSION: u32 = 33200;
pub const SEGGER_SYSVIEW_INFO_SIZE: u32 = 9;
pub const SEGGER_SYSVIEW_QUANTA_U32: u32 = 5;
pub const SEGGER_SYSVIEW_LOG: u32 = 0;
pub const SEGGER_SYSVIEW_WARNING: u32 = 1;
pub const SEGGER_SYSVIEW_ERROR: u32 = 2;
pub const SEGGER_SYSVIEW_FLAG_APPEND: u32 = 64;
pub const SYSVIEW_EVTID_NOP: u32 = 0;
pub const SYSVIEW_EVTID_OVERFLOW: u32 = 1;
pub const SYSVIEW_EVTID_ISR_ENTER: u32 = 2;
pub const SYSVIEW_EVTID_ISR_EXIT: u32 = 3;
pub const SYSVIEW_EVTID_TASK_START_EXEC: u32 = 4;
pub const SYSVIEW_EVTID_TASK_STOP_EXEC: u32 = 5;
pub const SYSVIEW_EVTID_TASK_START_READY: u32 = 6;
pub const SYSVIEW_EVTID_TASK_STOP_READY: u32 = 7;
pub const SYSVIEW_EVTID_TASK_CREATE: u32 = 8;
pub const SYSVIEW_EVTID_TASK_INFO: u32 = 9;
pub const SYSVIEW_EVTID_TRACE_START: u32 = 10;
pub const SYSVIEW_EVTID_TRACE_STOP: u32 = 11;
pub const SYSVIEW_EVTID_SYSTIME_CYCLES: u32 = 12;
pub const SYSVIEW_EVTID_SYSTIME_US: u32 = 13;
pub const SYSVIEW_EVTID_SYSDESC: u32 = 14;
pub const SYSVIEW_EVTID_MARK_START: u32 = 15;
pub const SYSVIEW_EVTID_MARK_STOP: u32 = 16;
pub const SYSVIEW_EVTID_IDLE: u32 = 17;
pub const SYSVIEW_EVTID_ISR_TO_SCHEDULER: u32 = 18;
pub const SYSVIEW_EVTID_TIMER_ENTER: u32 = 19;
pub const SYSVIEW_EVTID_TIMER_EXIT: u32 = 20;
pub const SYSVIEW_EVTID_STACK_INFO: u32 = 21;
pub const SYSVIEW_EVTID_MODULEDESC: u32 = 22;
pub const SYSVIEW_EVTID_INIT: u32 = 24;
pub const SYSVIEW_EVTID_NAME_RESOURCE: u32 = 25;
pub const SYSVIEW_EVTID_PRINT_FORMATTED: u32 = 26;
pub const SYSVIEW_EVTID_NUMMODULES: u32 = 27;
pub const SYSVIEW_EVTID_END_CALL: u32 = 28;
pub const SYSVIEW_EVTID_TASK_TERMINATE: u32 = 29;
pub const SYSVIEW_EVTID_EX: u32 = 31;
pub const SYSVIEW_EVTID_EX_MARK: u32 = 0;
pub const SYSVIEW_EVTID_EX_NAME_MARKER: u32 = 1;
pub const SYSVIEW_EVTID_EX_HEAP_DEFINE: u32 = 2;
pub const SYSVIEW_EVTID_EX_HEAP_ALLOC: u32 = 3;
pub const SYSVIEW_EVTID_EX_HEAP_ALLOC_EX: u32 = 4;
pub const SYSVIEW_EVTID_EX_HEAP_FREE: u32 = 5;
pub const SYSVIEW_EVTMASK_NOP: u32 = 1;
pub const SYSVIEW_EVTMASK_OVERFLOW: u32 = 2;
pub const SYSVIEW_EVTMASK_ISR_ENTER: u32 = 4;
pub const SYSVIEW_EVTMASK_ISR_EXIT: u32 = 8;
pub const SYSVIEW_EVTMASK_TASK_START_EXEC: u32 = 16;
pub const SYSVIEW_EVTMASK_TASK_STOP_EXEC: u32 = 32;
pub const SYSVIEW_EVTMASK_TASK_START_READY: u32 = 64;
pub const SYSVIEW_EVTMASK_TASK_STOP_READY: u32 = 128;
pub const SYSVIEW_EVTMASK_TASK_CREATE: u32 = 256;
pub const SYSVIEW_EVTMASK_TASK_INFO: u32 = 512;
pub const SYSVIEW_EVTMASK_TRACE_START: u32 = 1024;
pub const SYSVIEW_EVTMASK_TRACE_STOP: u32 = 2048;
pub const SYSVIEW_EVTMASK_SYSTIME_CYCLES: u32 = 4096;
pub const SYSVIEW_EVTMASK_SYSTIME_US: u32 = 8192;
pub const SYSVIEW_EVTMASK_SYSDESC: u32 = 16384;
pub const SYSVIEW_EVTMASK_IDLE: u32 = 131072;
pub const SYSVIEW_EVTMASK_ISR_TO_SCHEDULER: u32 = 262144;
pub const SYSVIEW_EVTMASK_TIMER_ENTER: u32 = 524288;
pub const SYSVIEW_EVTMASK_TIMER_EXIT: u32 = 1048576;
pub const SYSVIEW_EVTMASK_STACK_INFO: u32 = 2097152;
pub const SYSVIEW_EVTMASK_MODULEDESC: u32 = 4194304;
pub const SYSVIEW_EVTMASK_INIT: u32 = 16777216;
pub const SYSVIEW_EVTMASK_NAME_RESOURCE: u32 = 33554432;
pub const SYSVIEW_EVTMASK_PRINT_FORMATTED: u32 = 67108864;
pub const SYSVIEW_EVTMASK_NUMMODULES: u32 = 134217728;
pub const SYSVIEW_EVTMASK_END_CALL: u32 = 268435456;
pub const SYSVIEW_EVTMASK_TASK_TERMINATE: u32 = 536870912;
pub const SYSVIEW_EVTMASK_EX: u32 = 2147483648;
pub const SYSVIEW_EVTMASK_ALL_INTERRUPTS: u32 = 262156;
pub const SYSVIEW_EVTMASK_ALL_TASKS: u32 = 538969072;
pub type va_list = u32;
///       Types
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SEGGER_BUFFER_DESC {
    pub pBuffer: *mut cty::c_char,
    pub BufferSize: cty::c_int,
    pub Cnt: cty::c_int,
}
#[test]
fn bindgen_test_layout_SEGGER_BUFFER_DESC() {
    const UNINIT: ::core::mem::MaybeUninit<SEGGER_BUFFER_DESC> = ::core::mem::MaybeUninit::uninit();
    let ptr = UNINIT.as_ptr();
    assert_eq!(
        ::core::mem::size_of::<SEGGER_BUFFER_DESC>(),
        12usize,
        concat!("Size of: ", stringify!(SEGGER_BUFFER_DESC))
    );
    assert_eq!(
        ::core::mem::align_of::<SEGGER_BUFFER_DESC>(),
        4usize,
        concat!("Alignment of ", stringify!(SEGGER_BUFFER_DESC))
    );
    assert_eq!(
        unsafe { ::core::ptr::addr_of!((*ptr).pBuffer) as usize - ptr as usize },
        0usize,
        concat!("Offset of field: ", stringify!(SEGGER_BUFFER_DESC), "::", stringify!(pBuffer))
    );
    assert_eq!(
        unsafe { ::core::ptr::addr_of!((*ptr).BufferSize) as usize - ptr as usize },
        4usize,
        concat!("Offset of field: ", stringify!(SEGGER_BUFFER_DESC), "::", stringify!(BufferSize))
    );
    assert_eq!(
        unsafe { ::core::ptr::addr_of!((*ptr).Cnt) as usize - ptr as usize },
        8usize,
        concat!("Offset of field: ", stringify!(SEGGER_BUFFER_DESC), "::", stringify!(Cnt))
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SEGGER_CACHE_CONFIG {
    pub CacheLineSize: cty::c_uint,
    pub pfDMB: ::core::option::Option<unsafe extern "C" fn()>,
    pub pfClean: ::core::option::Option<unsafe extern "C" fn(p: *mut cty::c_void, NumBytes: cty::c_ulong)>,
    pub pfInvalidate:
        ::core::option::Option<unsafe extern "C" fn(p: *mut cty::c_void, NumBytes: cty::c_ulong)>,
}
#[test]
fn bindgen_test_layout_SEGGER_CACHE_CONFIG() {
    const UNINIT: ::core::mem::MaybeUninit<SEGGER_CACHE_CONFIG> = ::core::mem::MaybeUninit::uninit();
    let ptr = UNINIT.as_ptr();
    assert_eq!(
        ::core::mem::size_of::<SEGGER_CACHE_CONFIG>(),
        16usize,
        concat!("Size of: ", stringify!(SEGGER_CACHE_CONFIG))
    );
    assert_eq!(
        ::core::mem::align_of::<SEGGER_CACHE_CONFIG>(),
        4usize,
        concat!("Alignment of ", stringify!(SEGGER_CACHE_CONFIG))
    );
    assert_eq!(
        unsafe { ::core::ptr::addr_of!((*ptr).CacheLineSize) as usize - ptr as usize },
        0usize,
        concat!("Offset of field: ", stringify!(SEGGER_CACHE_CONFIG), "::", stringify!(CacheLineSize))
    );
    assert_eq!(
        unsafe { ::core::ptr::addr_of!((*ptr).pfDMB) as usize - ptr as usize },
        4usize,
        concat!("Offset of field: ", stringify!(SEGGER_CACHE_CONFIG), "::", stringify!(pfDMB))
    );
    assert_eq!(
        unsafe { ::core::ptr::addr_of!((*ptr).pfClean) as usize - ptr as usize },
        8usize,
        concat!("Offset of field: ", stringify!(SEGGER_CACHE_CONFIG), "::", stringify!(pfClean))
    );
    assert_eq!(
        unsafe { ::core::ptr::addr_of!((*ptr).pfInvalidate) as usize - ptr as usize },
        12usize,
        concat!("Offset of field: ", stringify!(SEGGER_CACHE_CONFIG), "::", stringify!(pfInvalidate))
    );
}
pub type SEGGER_SNPRINTF_CONTEXT = SEGGER_SNPRINTF_CONTEXT_struct;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SEGGER_SNPRINTF_CONTEXT_struct {
    pub pContext: *mut cty::c_void,
    pub pBufferDesc: *mut SEGGER_BUFFER_DESC,
    pub pfFlush: ::core::option::Option<unsafe extern "C" fn(pContext: *mut SEGGER_SNPRINTF_CONTEXT)>,
}
#[test]
fn bindgen_test_layout_SEGGER_SNPRINTF_CONTEXT_struct() {
    const UNINIT: ::core::mem::MaybeUninit<SEGGER_SNPRINTF_CONTEXT_struct> =
        ::core::mem::MaybeUninit::uninit();
    let ptr = UNINIT.as_ptr();
    assert_eq!(
        ::core::mem::size_of::<SEGGER_SNPRINTF_CONTEXT_struct>(),
        12usize,
        concat!("Size of: ", stringify!(SEGGER_SNPRINTF_CONTEXT_struct))
    );
    assert_eq!(
        ::core::mem::align_of::<SEGGER_SNPRINTF_CONTEXT_struct>(),
        4usize,
        concat!("Alignment of ", stringify!(SEGGER_SNPRINTF_CONTEXT_struct))
    );
    assert_eq!(
        unsafe { ::core::ptr::addr_of!((*ptr).pContext) as usize - ptr as usize },
        0usize,
        concat!("Offset of field: ", stringify!(SEGGER_SNPRINTF_CONTEXT_struct), "::", stringify!(pContext))
    );
    assert_eq!(
        unsafe { ::core::ptr::addr_of!((*ptr).pBufferDesc) as usize - ptr as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(SEGGER_SNPRINTF_CONTEXT_struct),
            "::",
            stringify!(pBufferDesc)
        )
    );
    assert_eq!(
        unsafe { ::core::ptr::addr_of!((*ptr).pfFlush) as usize - ptr as usize },
        8usize,
        concat!("Offset of field: ", stringify!(SEGGER_SNPRINTF_CONTEXT_struct), "::", stringify!(pfFlush))
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SEGGER_PRINTF_API {
    pub pfStoreChar: ::core::option::Option<
        unsafe extern "C" fn(
            pBufferDesc: *mut SEGGER_BUFFER_DESC,
            pContext: *mut SEGGER_SNPRINTF_CONTEXT,
            c: cty::c_char,
        ),
    >,
    pub pfPrintUnsigned: ::core::option::Option<
        unsafe extern "C" fn(
            pBufferDesc: *mut SEGGER_BUFFER_DESC,
            pContext: *mut SEGGER_SNPRINTF_CONTEXT,
            v: cty::c_ulong,
            Base: cty::c_uint,
            Flags: cty::c_char,
            Width: cty::c_int,
            Precision: cty::c_int,
        ) -> cty::c_int,
    >,
    pub pfPrintInt: ::core::option::Option<
        unsafe extern "C" fn(
            pBufferDesc: *mut SEGGER_BUFFER_DESC,
            pContext: *mut SEGGER_SNPRINTF_CONTEXT,
            v: cty::c_long,
            Base: cty::c_uint,
            Flags: cty::c_char,
            Width: cty::c_int,
            Precision: cty::c_int,
        ) -> cty::c_int,
    >,
}
#[test]
fn bindgen_test_layout_SEGGER_PRINTF_API() {
    const UNINIT: ::core::mem::MaybeUninit<SEGGER_PRINTF_API> = ::core::mem::MaybeUninit::uninit();
    let ptr = UNINIT.as_ptr();
    assert_eq!(
        ::core::mem::size_of::<SEGGER_PRINTF_API>(),
        12usize,
        concat!("Size of: ", stringify!(SEGGER_PRINTF_API))
    );
    assert_eq!(
        ::core::mem::align_of::<SEGGER_PRINTF_API>(),
        4usize,
        concat!("Alignment of ", stringify!(SEGGER_PRINTF_API))
    );
    assert_eq!(
        unsafe { ::core::ptr::addr_of!((*ptr).pfStoreChar) as usize - ptr as usize },
        0usize,
        concat!("Offset of field: ", stringify!(SEGGER_PRINTF_API), "::", stringify!(pfStoreChar))
    );
    assert_eq!(
        unsafe { ::core::ptr::addr_of!((*ptr).pfPrintUnsigned) as usize - ptr as usize },
        4usize,
        concat!("Offset of field: ", stringify!(SEGGER_PRINTF_API), "::", stringify!(pfPrintUnsigned))
    );
    assert_eq!(
        unsafe { ::core::ptr::addr_of!((*ptr).pfPrintInt) as usize - ptr as usize },
        8usize,
        concat!("Offset of field: ", stringify!(SEGGER_PRINTF_API), "::", stringify!(pfPrintInt))
    );
}
pub type SEGGER_pFormatter = ::core::option::Option<
    unsafe extern "C" fn(
        pBufferDesc: *mut SEGGER_BUFFER_DESC,
        pContext: *mut SEGGER_SNPRINTF_CONTEXT,
        pApi: *const SEGGER_PRINTF_API,
        pParamList: *mut va_list,
        Lead: cty::c_char,
        Width: cty::c_int,
        Precision: cty::c_int,
    ),
>;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SEGGER_PRINTF_FORMATTER {
    pub pNext: *mut SEGGER_PRINTF_FORMATTER,
    pub pfFormatter: SEGGER_pFormatter,
    pub Specifier: cty::c_char,
}
#[test]
fn bindgen_test_layout_SEGGER_PRINTF_FORMATTER() {
    const UNINIT: ::core::mem::MaybeUninit<SEGGER_PRINTF_FORMATTER> = ::core::mem::MaybeUninit::uninit();
    let ptr = UNINIT.as_ptr();
    assert_eq!(
        ::core::mem::size_of::<SEGGER_PRINTF_FORMATTER>(),
        12usize,
        concat!("Size of: ", stringify!(SEGGER_PRINTF_FORMATTER))
    );
    assert_eq!(
        ::core::mem::align_of::<SEGGER_PRINTF_FORMATTER>(),
        4usize,
        concat!("Alignment of ", stringify!(SEGGER_PRINTF_FORMATTER))
    );
    assert_eq!(
        unsafe { ::core::ptr::addr_of!((*ptr).pNext) as usize - ptr as usize },
        0usize,
        concat!("Offset of field: ", stringify!(SEGGER_PRINTF_FORMATTER), "::", stringify!(pNext))
    );
    assert_eq!(
        unsafe { ::core::ptr::addr_of!((*ptr).pfFormatter) as usize - ptr as usize },
        4usize,
        concat!("Offset of field: ", stringify!(SEGGER_PRINTF_FORMATTER), "::", stringify!(pfFormatter))
    );
    assert_eq!(
        unsafe { ::core::ptr::addr_of!((*ptr).Specifier) as usize - ptr as usize },
        8usize,
        concat!("Offset of field: ", stringify!(SEGGER_PRINTF_FORMATTER), "::", stringify!(Specifier))
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SEGGER_BSP_API {
    pub pfGetHPTimestamp: ::core::option::Option<unsafe extern "C" fn() -> cty::c_ulong>,
    pub pfGetUID: ::core::option::Option<unsafe extern "C" fn(abUID: *mut cty::c_uchar) -> cty::c_int>,
}
#[test]
fn bindgen_test_layout_SEGGER_BSP_API() {
    const UNINIT: ::core::mem::MaybeUninit<SEGGER_BSP_API> = ::core::mem::MaybeUninit::uninit();
    let ptr = UNINIT.as_ptr();
    assert_eq!(
        ::core::mem::size_of::<SEGGER_BSP_API>(),
        8usize,
        concat!("Size of: ", stringify!(SEGGER_BSP_API))
    );
    assert_eq!(
        ::core::mem::align_of::<SEGGER_BSP_API>(),
        4usize,
        concat!("Alignment of ", stringify!(SEGGER_BSP_API))
    );
    assert_eq!(
        unsafe { ::core::ptr::addr_of!((*ptr).pfGetHPTimestamp) as usize - ptr as usize },
        0usize,
        concat!("Offset of field: ", stringify!(SEGGER_BSP_API), "::", stringify!(pfGetHPTimestamp))
    );
    assert_eq!(
        unsafe { ::core::ptr::addr_of!((*ptr).pfGetUID) as usize - ptr as usize },
        4usize,
        concat!("Offset of field: ", stringify!(SEGGER_BSP_API), "::", stringify!(pfGetUID))
    );
}
extern "C" {
    ///       Utility functions
    pub fn SEGGER_ARM_memcpy(pDest: *mut cty::c_void, pSrc: *const cty::c_void, NumBytes: cty::c_int);
}
extern "C" {
    pub fn SEGGER_memcpy(pDest: *mut cty::c_void, pSrc: *const cty::c_void, NumBytes: cty::c_uint);
}
extern "C" {
    pub fn SEGGER_memxor(pDest: *mut cty::c_void, pSrc: *const cty::c_void, NumBytes: cty::c_uint);
}
extern "C" {
    pub fn SEGGER_atoi(s: *const cty::c_char) -> cty::c_int;
}
extern "C" {
    pub fn SEGGER_isalnum(c: cty::c_int) -> cty::c_int;
}
extern "C" {
    pub fn SEGGER_isalpha(c: cty::c_int) -> cty::c_int;
}
extern "C" {
    pub fn SEGGER_strlen(s: *const cty::c_char) -> cty::c_uint;
}
extern "C" {
    pub fn SEGGER_tolower(c: cty::c_int) -> cty::c_int;
}
extern "C" {
    pub fn SEGGER_strcasecmp(sText1: *const cty::c_char, sText2: *const cty::c_char) -> cty::c_int;
}
extern "C" {
    pub fn SEGGER_strncasecmp(
        sText1: *const cty::c_char,
        sText2: *const cty::c_char,
        Count: cty::c_uint,
    ) -> cty::c_int;
}
extern "C" {
    pub fn SEGGER_StoreChar(pBufferDesc: *mut SEGGER_BUFFER_DESC, c: cty::c_char);
}
extern "C" {
    pub fn SEGGER_PrintUnsigned(
        pBufferDesc: *mut SEGGER_BUFFER_DESC,
        v: cty::c_ulong,
        Base: cty::c_uint,
        Precision: cty::c_int,
    );
}
extern "C" {
    pub fn SEGGER_PrintInt(
        pBufferDesc: *mut SEGGER_BUFFER_DESC,
        v: cty::c_long,
        Base: cty::c_uint,
        Precision: cty::c_int,
    );
}
extern "C" {
    pub fn SEGGER_snprintf(
        pBuffer: *mut cty::c_char,
        BufferSize: cty::c_int,
        sFormat: *const cty::c_char,
        ...
    ) -> cty::c_int;
}
extern "C" {
    pub fn SEGGER_vsnprintf(
        pBuffer: *mut cty::c_char,
        BufferSize: cty::c_int,
        sFormat: *const cty::c_char,
        ParamList: va_list,
    ) -> cty::c_int;
}
extern "C" {
    pub fn SEGGER_vsnprintfEx(
        pContext: *mut SEGGER_SNPRINTF_CONTEXT,
        sFormat: *const cty::c_char,
        ParamList: va_list,
    ) -> cty::c_int;
}
extern "C" {
    pub fn SEGGER_PRINTF_AddFormatter(
        pFormatter: *mut SEGGER_PRINTF_FORMATTER,
        pfFormatter: SEGGER_pFormatter,
        c: cty::c_char,
    ) -> cty::c_int;
}
extern "C" {
    pub fn SEGGER_PRINTF_AddDoubleFormatter();
}
extern "C" {
    pub fn SEGGER_PRINTF_AddIPFormatter();
}
extern "C" {
    pub fn SEGGER_PRINTF_AddBLUEFormatter();
}
extern "C" {
    pub fn SEGGER_PRINTF_AddCONNECTFormatter();
}
extern "C" {
    pub fn SEGGER_PRINTF_AddSSLFormatter();
}
extern "C" {
    pub fn SEGGER_PRINTF_AddSSHFormatter();
}
extern "C" {
    pub fn SEGGER_PRINTF_AddHTMLFormatter();
}
extern "C" {
    pub fn SEGGER_BSP_GetUID(abUID: *mut cty::c_uchar) -> cty::c_int;
}
extern "C" {
    pub fn SEGGER_BSP_GetUID32(pUID: *mut cty::c_ulong) -> cty::c_int;
}
extern "C" {
    pub fn SEGGER_BSP_SetAPI(pAPI: *const SEGGER_BSP_API);
}
extern "C" {
    pub fn SEGGER_BSP_SeedUID();
}
extern "C" {
    pub fn SEGGER_VERSION_GetString(acText: *mut cty::c_char, Version: cty::c_uint);
}
///       Structures
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SEGGER_SYSVIEW_TASKINFO {
    pub TaskID: cty::c_ulong,
    pub sName: *const cty::c_char,
    pub Prio: cty::c_ulong,
    pub StackBase: cty::c_ulong,
    pub StackSize: cty::c_ulong,
}
#[test]
fn bindgen_test_layout_SEGGER_SYSVIEW_TASKINFO() {
    const UNINIT: ::core::mem::MaybeUninit<SEGGER_SYSVIEW_TASKINFO> = ::core::mem::MaybeUninit::uninit();
    let ptr = UNINIT.as_ptr();
    assert_eq!(
        ::core::mem::size_of::<SEGGER_SYSVIEW_TASKINFO>(),
        20usize,
        concat!("Size of: ", stringify!(SEGGER_SYSVIEW_TASKINFO))
    );
    assert_eq!(
        ::core::mem::align_of::<SEGGER_SYSVIEW_TASKINFO>(),
        4usize,
        concat!("Alignment of ", stringify!(SEGGER_SYSVIEW_TASKINFO))
    );
    assert_eq!(
        unsafe { ::core::ptr::addr_of!((*ptr).TaskID) as usize - ptr as usize },
        0usize,
        concat!("Offset of field: ", stringify!(SEGGER_SYSVIEW_TASKINFO), "::", stringify!(TaskID))
    );
    assert_eq!(
        unsafe { ::core::ptr::addr_of!((*ptr).sName) as usize - ptr as usize },
        4usize,
        concat!("Offset of field: ", stringify!(SEGGER_SYSVIEW_TASKINFO), "::", stringify!(sName))
    );
    assert_eq!(
        unsafe { ::core::ptr::addr_of!((*ptr).Prio) as usize - ptr as usize },
        8usize,
        concat!("Offset of field: ", stringify!(SEGGER_SYSVIEW_TASKINFO), "::", stringify!(Prio))
    );
    assert_eq!(
        unsafe { ::core::ptr::addr_of!((*ptr).StackBase) as usize - ptr as usize },
        12usize,
        concat!("Offset of field: ", stringify!(SEGGER_SYSVIEW_TASKINFO), "::", stringify!(StackBase))
    );
    assert_eq!(
        unsafe { ::core::ptr::addr_of!((*ptr).StackSize) as usize - ptr as usize },
        16usize,
        concat!("Offset of field: ", stringify!(SEGGER_SYSVIEW_TASKINFO), "::", stringify!(StackSize))
    );
}
pub type SEGGER_SYSVIEW_MODULE = SEGGER_SYSVIEW_MODULE_STRUCT;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SEGGER_SYSVIEW_MODULE_STRUCT {
    pub sModule: *const cty::c_char,
    pub NumEvents: cty::c_ulong,
    pub EventOffset: cty::c_ulong,
    pub pfSendModuleDesc: ::core::option::Option<unsafe extern "C" fn()>,
    pub pNext: *mut SEGGER_SYSVIEW_MODULE,
}
#[test]
fn bindgen_test_layout_SEGGER_SYSVIEW_MODULE_STRUCT() {
    const UNINIT: ::core::mem::MaybeUninit<SEGGER_SYSVIEW_MODULE_STRUCT> = ::core::mem::MaybeUninit::uninit();
    let ptr = UNINIT.as_ptr();
    assert_eq!(
        ::core::mem::size_of::<SEGGER_SYSVIEW_MODULE_STRUCT>(),
        20usize,
        concat!("Size of: ", stringify!(SEGGER_SYSVIEW_MODULE_STRUCT))
    );
    assert_eq!(
        ::core::mem::align_of::<SEGGER_SYSVIEW_MODULE_STRUCT>(),
        4usize,
        concat!("Alignment of ", stringify!(SEGGER_SYSVIEW_MODULE_STRUCT))
    );
    assert_eq!(
        unsafe { ::core::ptr::addr_of!((*ptr).sModule) as usize - ptr as usize },
        0usize,
        concat!("Offset of field: ", stringify!(SEGGER_SYSVIEW_MODULE_STRUCT), "::", stringify!(sModule))
    );
    assert_eq!(
        unsafe { ::core::ptr::addr_of!((*ptr).NumEvents) as usize - ptr as usize },
        4usize,
        concat!("Offset of field: ", stringify!(SEGGER_SYSVIEW_MODULE_STRUCT), "::", stringify!(NumEvents))
    );
    assert_eq!(
        unsafe { ::core::ptr::addr_of!((*ptr).EventOffset) as usize - ptr as usize },
        8usize,
        concat!("Offset of field: ", stringify!(SEGGER_SYSVIEW_MODULE_STRUCT), "::", stringify!(EventOffset))
    );
    assert_eq!(
        unsafe { ::core::ptr::addr_of!((*ptr).pfSendModuleDesc) as usize - ptr as usize },
        12usize,
        concat!(
            "Offset of field: ",
            stringify!(SEGGER_SYSVIEW_MODULE_STRUCT),
            "::",
            stringify!(pfSendModuleDesc)
        )
    );
    assert_eq!(
        unsafe { ::core::ptr::addr_of!((*ptr).pNext) as usize - ptr as usize },
        16usize,
        concat!("Offset of field: ", stringify!(SEGGER_SYSVIEW_MODULE_STRUCT), "::", stringify!(pNext))
    );
}
pub type SEGGER_SYSVIEW_SEND_SYS_DESC_FUNC = ::core::option::Option<unsafe extern "C" fn()>;
extern "C" {
    pub static mut SEGGER_SYSVIEW_TickCnt: cty::c_uint;
}
extern "C" {
    pub static mut SEGGER_SYSVIEW_InterruptId: cty::c_uint;
}
///       API functions
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SEGGER_SYSVIEW_OS_API {
    pub pfGetTime: ::core::option::Option<unsafe extern "C" fn() -> cty::c_ulonglong>,
    pub pfSendTaskList: ::core::option::Option<unsafe extern "C" fn()>,
}
#[test]
fn bindgen_test_layout_SEGGER_SYSVIEW_OS_API() {
    const UNINIT: ::core::mem::MaybeUninit<SEGGER_SYSVIEW_OS_API> = ::core::mem::MaybeUninit::uninit();
    let ptr = UNINIT.as_ptr();
    assert_eq!(
        ::core::mem::size_of::<SEGGER_SYSVIEW_OS_API>(),
        8usize,
        concat!("Size of: ", stringify!(SEGGER_SYSVIEW_OS_API))
    );
    assert_eq!(
        ::core::mem::align_of::<SEGGER_SYSVIEW_OS_API>(),
        4usize,
        concat!("Alignment of ", stringify!(SEGGER_SYSVIEW_OS_API))
    );
    assert_eq!(
        unsafe { ::core::ptr::addr_of!((*ptr).pfGetTime) as usize - ptr as usize },
        0usize,
        concat!("Offset of field: ", stringify!(SEGGER_SYSVIEW_OS_API), "::", stringify!(pfGetTime))
    );
    assert_eq!(
        unsafe { ::core::ptr::addr_of!((*ptr).pfSendTaskList) as usize - ptr as usize },
        4usize,
        concat!("Offset of field: ", stringify!(SEGGER_SYSVIEW_OS_API), "::", stringify!(pfSendTaskList))
    );
}
extern "C" {
    ///       Control and initialization functions
    pub fn SEGGER_SYSVIEW_Init(
        SysFreq: cty::c_ulong,
        CPUFreq: cty::c_ulong,
        pOSAPI: *const SEGGER_SYSVIEW_OS_API,
        pfSendSysDesc: SEGGER_SYSVIEW_SEND_SYS_DESC_FUNC,
        pUpBuffer: *mut cty::c_void,
        UpBufferSize: cty::c_ulong,
        pDownBuffer: *mut cty::c_void,
        DownBufferSize: cty::c_ulong,
    );
}
extern "C" {
    pub fn SEGGER_SYSVIEW_SetRAMBase(RAMBaseAddress: cty::c_ulong);
}
extern "C" {
    pub fn SEGGER_SYSVIEW_Start();
}
extern "C" {
    pub fn SEGGER_SYSVIEW_Stop();
}
extern "C" {
    pub fn SEGGER_SYSVIEW_GetSysDesc();
}
extern "C" {
    pub fn SEGGER_SYSVIEW_SendTaskList();
}
extern "C" {
    pub fn SEGGER_SYSVIEW_SendTaskInfo(pInfo: *const SEGGER_SYSVIEW_TASKINFO);
}
extern "C" {
    pub fn SEGGER_SYSVIEW_SendSysDesc(sSysDesc: *const cty::c_char);
}
extern "C" {
    pub fn SEGGER_SYSVIEW_IsStarted() -> cty::c_int;
}
extern "C" {
    pub fn SEGGER_SYSVIEW_GetChannelID() -> cty::c_int;
}
extern "C" {
    ///       Event recording functions
    pub fn SEGGER_SYSVIEW_RecordVoid(EventId: cty::c_uint);
}
extern "C" {
    pub fn SEGGER_SYSVIEW_RecordU32(EventId: cty::c_uint, Para0: cty::c_ulong);
}
extern "C" {
    pub fn SEGGER_SYSVIEW_RecordU32x2(EventId: cty::c_uint, Para0: cty::c_ulong, Para1: cty::c_ulong);
}
extern "C" {
    pub fn SEGGER_SYSVIEW_RecordU32x3(
        EventId: cty::c_uint,
        Para0: cty::c_ulong,
        Para1: cty::c_ulong,
        Para2: cty::c_ulong,
    );
}
extern "C" {
    pub fn SEGGER_SYSVIEW_RecordU32x4(
        EventId: cty::c_uint,
        Para0: cty::c_ulong,
        Para1: cty::c_ulong,
        Para2: cty::c_ulong,
        Para3: cty::c_ulong,
    );
}
extern "C" {
    pub fn SEGGER_SYSVIEW_RecordU32x5(
        EventId: cty::c_uint,
        Para0: cty::c_ulong,
        Para1: cty::c_ulong,
        Para2: cty::c_ulong,
        Para3: cty::c_ulong,
        Para4: cty::c_ulong,
    );
}
extern "C" {
    pub fn SEGGER_SYSVIEW_RecordU32x6(
        EventId: cty::c_uint,
        Para0: cty::c_ulong,
        Para1: cty::c_ulong,
        Para2: cty::c_ulong,
        Para3: cty::c_ulong,
        Para4: cty::c_ulong,
        Para5: cty::c_ulong,
    );
}
extern "C" {
    pub fn SEGGER_SYSVIEW_RecordU32x7(
        EventId: cty::c_uint,
        Para0: cty::c_ulong,
        Para1: cty::c_ulong,
        Para2: cty::c_ulong,
        Para3: cty::c_ulong,
        Para4: cty::c_ulong,
        Para5: cty::c_ulong,
        Para6: cty::c_ulong,
    );
}
extern "C" {
    pub fn SEGGER_SYSVIEW_RecordU32x8(
        EventId: cty::c_uint,
        Para0: cty::c_ulong,
        Para1: cty::c_ulong,
        Para2: cty::c_ulong,
        Para3: cty::c_ulong,
        Para4: cty::c_ulong,
        Para5: cty::c_ulong,
        Para6: cty::c_ulong,
        Para7: cty::c_ulong,
    );
}
extern "C" {
    pub fn SEGGER_SYSVIEW_RecordU32x9(
        EventId: cty::c_uint,
        Para0: cty::c_ulong,
        Para1: cty::c_ulong,
        Para2: cty::c_ulong,
        Para3: cty::c_ulong,
        Para4: cty::c_ulong,
        Para5: cty::c_ulong,
        Para6: cty::c_ulong,
        Para7: cty::c_ulong,
        Para8: cty::c_ulong,
    );
}
extern "C" {
    pub fn SEGGER_SYSVIEW_RecordU32x10(
        EventId: cty::c_uint,
        Para0: cty::c_ulong,
        Para1: cty::c_ulong,
        Para2: cty::c_ulong,
        Para3: cty::c_ulong,
        Para4: cty::c_ulong,
        Para5: cty::c_ulong,
        Para6: cty::c_ulong,
        Para7: cty::c_ulong,
        Para8: cty::c_ulong,
        Para9: cty::c_ulong,
    );
}
extern "C" {
    pub fn SEGGER_SYSVIEW_RecordString(EventId: cty::c_uint, pString: *const cty::c_char);
}
extern "C" {
    pub fn SEGGER_SYSVIEW_RecordSystime();
}
extern "C" {
    pub fn SEGGER_SYSVIEW_RecordEnterISR();
}
extern "C" {
    pub fn SEGGER_SYSVIEW_RecordExitISR();
}
extern "C" {
    pub fn SEGGER_SYSVIEW_RecordExitISRToScheduler();
}
extern "C" {
    pub fn SEGGER_SYSVIEW_RecordEnterTimer(TimerId: cty::c_ulong);
}
extern "C" {
    pub fn SEGGER_SYSVIEW_RecordExitTimer();
}
extern "C" {
    pub fn SEGGER_SYSVIEW_RecordEndCall(EventID: cty::c_uint);
}
extern "C" {
    pub fn SEGGER_SYSVIEW_RecordEndCallU32(EventID: cty::c_uint, Para0: cty::c_ulong);
}
extern "C" {
    pub fn SEGGER_SYSVIEW_OnIdle();
}
extern "C" {
    pub fn SEGGER_SYSVIEW_OnTaskCreate(TaskId: cty::c_ulong);
}
extern "C" {
    pub fn SEGGER_SYSVIEW_OnTaskTerminate(TaskId: cty::c_ulong);
}
extern "C" {
    pub fn SEGGER_SYSVIEW_OnTaskStartExec(TaskId: cty::c_ulong);
}
extern "C" {
    pub fn SEGGER_SYSVIEW_OnTaskStopExec();
}
extern "C" {
    pub fn SEGGER_SYSVIEW_OnTaskStartReady(TaskId: cty::c_ulong);
}
extern "C" {
    pub fn SEGGER_SYSVIEW_OnTaskStopReady(TaskId: cty::c_ulong, Cause: cty::c_uint);
}
extern "C" {
    pub fn SEGGER_SYSVIEW_MarkStart(MarkerId: cty::c_uint);
}
extern "C" {
    pub fn SEGGER_SYSVIEW_MarkStop(MarkerId: cty::c_uint);
}
extern "C" {
    pub fn SEGGER_SYSVIEW_Mark(MarkerId: cty::c_uint);
}
extern "C" {
    pub fn SEGGER_SYSVIEW_NameMarker(MarkerId: cty::c_uint, sName: *const cty::c_char);
}
extern "C" {
    pub fn SEGGER_SYSVIEW_HeapDefine(
        pHeap: *mut cty::c_void,
        pBase: *mut cty::c_void,
        HeapSize: cty::c_uint,
        MetadataSize: cty::c_uint,
    );
}
extern "C" {
    pub fn SEGGER_SYSVIEW_HeapAlloc(
        pHeap: *mut cty::c_void,
        pUserData: *mut cty::c_void,
        UserDataLen: cty::c_uint,
    );
}
extern "C" {
    pub fn SEGGER_SYSVIEW_HeapAllocEx(
        pHeap: *mut cty::c_void,
        pUserData: *mut cty::c_void,
        UserDataLen: cty::c_uint,
        Tag: cty::c_uint,
    );
}
extern "C" {
    pub fn SEGGER_SYSVIEW_HeapFree(pHeap: *mut cty::c_void, pUserData: *mut cty::c_void);
}
extern "C" {
    pub fn SEGGER_SYSVIEW_NameResource(ResourceId: cty::c_ulong, sName: *const cty::c_char);
}
extern "C" {
    pub fn SEGGER_SYSVIEW_SendPacket(
        pPacket: *mut cty::c_uchar,
        pPayloadEnd: *mut cty::c_uchar,
        EventId: cty::c_uint,
    ) -> cty::c_int;
}
extern "C" {
    ///       Event parameter encoding functions
    pub fn SEGGER_SYSVIEW_EncodeU32(pPayload: *mut cty::c_uchar, Value: cty::c_ulong) -> *mut cty::c_uchar;
}
extern "C" {
    pub fn SEGGER_SYSVIEW_EncodeData(
        pPayload: *mut cty::c_uchar,
        pSrc: *const cty::c_char,
        Len: cty::c_uint,
    ) -> *mut cty::c_uchar;
}
extern "C" {
    pub fn SEGGER_SYSVIEW_EncodeString(
        pPayload: *mut cty::c_uchar,
        s: *const cty::c_char,
        MaxLen: cty::c_uint,
    ) -> *mut cty::c_uchar;
}
extern "C" {
    pub fn SEGGER_SYSVIEW_EncodeId(pPayload: *mut cty::c_uchar, Id: cty::c_ulong) -> *mut cty::c_uchar;
}
extern "C" {
    pub fn SEGGER_SYSVIEW_ShrinkId(Id: cty::c_ulong) -> cty::c_ulong;
}
extern "C" {
    ///       Middleware module registration
    pub fn SEGGER_SYSVIEW_RegisterModule(pModule: *mut SEGGER_SYSVIEW_MODULE);
}
extern "C" {
    pub fn SEGGER_SYSVIEW_RecordModuleDescription(
        pModule: *const SEGGER_SYSVIEW_MODULE,
        sDescription: *const cty::c_char,
    );
}
extern "C" {
    pub fn SEGGER_SYSVIEW_SendModule(ModuleId: cty::c_uchar);
}
extern "C" {
    pub fn SEGGER_SYSVIEW_SendModuleDescription();
}
extern "C" {
    pub fn SEGGER_SYSVIEW_SendNumModules();
}
extern "C" {
    pub fn SEGGER_SYSVIEW_PrintfHostEx(s: *const cty::c_char, Options: cty::c_ulong, ...);
}
extern "C" {
    pub fn SEGGER_SYSVIEW_VPrintfHostEx(
        s: *const cty::c_char,
        Options: cty::c_ulong,
        pParamList: *mut va_list,
    );
}
extern "C" {
    pub fn SEGGER_SYSVIEW_PrintfTargetEx(s: *const cty::c_char, Options: cty::c_ulong, ...);
}
extern "C" {
    pub fn SEGGER_SYSVIEW_VPrintfTargetEx(
        s: *const cty::c_char,
        Options: cty::c_ulong,
        pParamList: *mut va_list,
    );
}
extern "C" {
    pub fn SEGGER_SYSVIEW_PrintfHost(s: *const cty::c_char, ...);
}
extern "C" {
    pub fn SEGGER_SYSVIEW_VPrintfHost(s: *const cty::c_char, pParamList: *mut va_list);
}
extern "C" {
    pub fn SEGGER_SYSVIEW_PrintfTarget(s: *const cty::c_char, ...);
}
extern "C" {
    pub fn SEGGER_SYSVIEW_VPrintfTarget(s: *const cty::c_char, pParamList: *mut va_list);
}
extern "C" {
    pub fn SEGGER_SYSVIEW_WarnfHost(s: *const cty::c_char, ...);
}
extern "C" {
    pub fn SEGGER_SYSVIEW_VWarnfHost(s: *const cty::c_char, pParamList: *mut va_list);
}
extern "C" {
    pub fn SEGGER_SYSVIEW_WarnfTarget(s: *const cty::c_char, ...);
}
extern "C" {
    pub fn SEGGER_SYSVIEW_VWarnfTarget(s: *const cty::c_char, pParamList: *mut va_list);
}
extern "C" {
    pub fn SEGGER_SYSVIEW_ErrorfHost(s: *const cty::c_char, ...);
}
extern "C" {
    pub fn SEGGER_SYSVIEW_VErrorfHost(s: *const cty::c_char, pParamList: *mut va_list);
}
extern "C" {
    pub fn SEGGER_SYSVIEW_ErrorfTarget(s: *const cty::c_char, ...);
}
extern "C" {
    pub fn SEGGER_SYSVIEW_VErrorfTarget(s: *const cty::c_char, pParamList: *mut va_list);
}
extern "C" {
    pub fn SEGGER_SYSVIEW_Print(s: *const cty::c_char);
}
extern "C" {
    pub fn SEGGER_SYSVIEW_Warn(s: *const cty::c_char);
}
extern "C" {
    pub fn SEGGER_SYSVIEW_Error(s: *const cty::c_char);
}
extern "C" {
    ///       Run-time configuration functions
    pub fn SEGGER_SYSVIEW_EnableEvents(EnableMask: cty::c_ulong);
}
extern "C" {
    pub fn SEGGER_SYSVIEW_DisableEvents(DisableMask: cty::c_ulong);
}
extern "C" {
    ///       Application-provided functions
    pub fn SEGGER_SYSVIEW_Conf();
}
extern "C" {
    pub fn SEGGER_SYSVIEW_X_GetTimestamp() -> cty::c_ulong;
}
extern "C" {
    pub fn SEGGER_SYSVIEW_X_GetInterruptId() -> cty::c_ulong;
}
extern "C" {
    pub fn SEGGER_SYSVIEW_X_StartComm();
}
extern "C" {
    pub fn SEGGER_SYSVIEW_X_OnEventRecorded(NumBytes: cty::c_uint);
}
