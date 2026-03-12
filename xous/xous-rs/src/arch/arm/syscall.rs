use core::arch::asm;

use crate::definitions::SysCallResult;
use crate::syscall::SysCall;

#[inline]
pub fn syscall(call: SysCall) -> SysCallResult {
    let [mut a0, mut a1, mut a2, mut a3, mut a4, mut a5, mut a6, mut a7] = call.as_args();

    unsafe {
        asm!(
            "svc #0",
            inout("r0") a0,
            inout("r1") a1,
            inout("r2") a2,
            inout("r3") a3,
            inout("r4") a4,
            inout("r5") a5,
            // R6 and R7 are used by LLVM internally
            inout("r8") a6,
            inout("r9") a7,
            options(nostack)
        );
    };

    let ret = crate::Result::from_args([a0, a1, a2, a3, a4, a5, a6, a7]);
    match ret {
        crate::Result::Error(e) => Err(e),
        other => Ok(other),
    }
}
