// SPDX-FileCopyrightText: 2020 Sean Cross <sean@xobs.io>
// SPDX-License-Identifier: Apache-2.0

use xous::{arch::irq::IrqNumber, MemoryAddress, PID};

use crate::{arch, filled_array};

static mut IRQ_HANDLERS: [IrqHandler; 32] = filled_array!(IrqHandler::Free; 32);

enum IrqHandler {
    Free,
    User {
        pid: PID,
        pc: MemoryAddress,
        arg: Option<MemoryAddress>,
    },
    #[allow(dead_code)]
    Kernel {
        handler: fn(),
    },
}

#[cfg(keyos)]
pub fn handle(irq_no: IrqNumber) -> Result<xous::Result, xous::Error> {
    use crate::{
        process::{ArchProcess, ThreadState, IRQ_TID},
        services::SystemServices,
    };
    unsafe {
        match &IRQ_HANDLERS[irq_no as usize] {
            IrqHandler::User { pid, pc, arg } => {
                SystemServices::with_mut(|ss| {
                    klog!("Making a callback to PID{}: {:x?} ({:08x}, {:x?})", pid, pc, irq_no as usize, arg);

                    #[cfg(feature = "trace-systemview")]
                    {
                        crate::platform::atsama5d2::systemview::set_current_isr(irq_no as u32);
                        systemview_keyos::SystemView::isr_enter();
                    }

                    let process = ss.process_mut(*pid).unwrap();
                    process.set_thread_state(IRQ_TID, ThreadState::Ready);
                    process.activate();

                    ArchProcess::with_current_mut(|arch_process| {
                        // Activate the current context
                        arch_process.set_tid(IRQ_TID);
                        arch_process.run_irq_handler(
                            pc.get(),
                            irq_no as usize,
                            arg.map(|x| x.get()).unwrap_or(0),
                        );
                    });
                });
            }
            IrqHandler::Kernel { handler } => {
                klog!("Handling IRQ#{} in the kernel", irq_no as usize);
                handler()
            }
            IrqHandler::Free => {
                klog!("[!] Masked an unhandled IRQ #{:?}", irq_no);
                // If there is no handler, mask this interrupt
                // to prevent an IRQ storm.  This is considered
                // an error.
                arch::irq::disable_irq(irq_no);
            }
        }
    }
    Ok(xous::Result::ResumeProcess)
}

#[allow(dead_code)] // needed to silence a hosted mode warning
pub fn for_user_each_irq(mut f: impl FnMut(usize, &PID, &MemoryAddress, &Option<MemoryAddress>)) {
    unsafe {
        for (idx, handler) in (&*core::ptr::addr_of!(IRQ_HANDLERS)).iter().enumerate() {
            if let IrqHandler::User { pid, pc, arg } = handler {
                f(idx, pid, pc, arg)
            }
        }
    }
}
#[allow(dead_code)] // needed to silence a hosted mode warning
pub fn for_kernel_each_irq(mut f: impl FnMut(usize)) {
    unsafe {
        for (idx, handler) in (&*core::ptr::addr_of!(IRQ_HANDLERS)).iter().enumerate() {
            if let IrqHandler::Kernel { .. } = handler {
                f(idx)
            }
        }
    }
}

#[cfg(keyos)]
pub fn interrupt_claim_kernel(irq: IrqNumber, handler: fn()) {
    unsafe {
        if !matches!(IRQ_HANDLERS[irq as usize], IrqHandler::Free) {
            panic!("Irq {irq:?} already taken");
        }
        IRQ_HANDLERS[irq as usize] = IrqHandler::Kernel { handler };
        arch::irq::enable_irq(irq);
    }
}

pub fn interrupt_claim_user(
    irq: IrqNumber,
    pid: PID,
    pc: MemoryAddress,
    arg: Option<MemoryAddress>,
) -> Result<(), xous::Error> {
    unsafe {
        if !matches!(IRQ_HANDLERS[irq as usize], IrqHandler::Free) {
            Err(xous::Error::InterruptInUse)
        } else {
            IRQ_HANDLERS[irq as usize] = IrqHandler::User { pid, pc, arg };
            arch::irq::enable_irq(irq);
            Ok(())
        }
    }
}

pub fn interrupt_free(irq: IrqNumber, pid: PID) -> Result<(), xous::Error> {
    unsafe {
        match &IRQ_HANDLERS[irq as usize] {
            IrqHandler::User { pid: pid_mapped, .. } if pid == *pid_mapped => {
                arch::irq::disable_irq(irq);
                IRQ_HANDLERS[irq as usize] = IrqHandler::Free;
                Ok(())
            }
            _ => Err(xous::Error::InterruptNotFound),
        }
    }
}

/// Iterate through the IRQ handlers and remove any handler that exists
/// for the given PID.
pub fn release_interrupts_for_pid(pid: PID) {
    unsafe {
        for (_irq, handler) in (&mut *core::ptr::addr_of_mut!(IRQ_HANDLERS)).iter_mut().enumerate() {
            match handler {
                IrqHandler::User { pid: pid_mapped, .. } if pid == *pid_mapped => {
                    #[cfg(keyos)]
                    arch::irq::disable_irq(_irq.try_into().unwrap());
                    *handler = IrqHandler::Free;
                }
                _ => {}
            }
        }
    }
}
