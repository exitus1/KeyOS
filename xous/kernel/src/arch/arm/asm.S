/*
 * SPDX-FileCopyrightText: 2022 Foundation Devices, Inc <hello@foundation.xyz>
 * SPDX-License-Identifier: Apache-2.0
 */

.section .text.reset_vector, "ax"

.global reset
.align 4
.arm

reset:

_exception_vectors_kernel:
  b   reset_vector          /* Entrypoint */
  b   trap_handler          /* Undefined Instruction */
  b   trap_handler          /* Supervisor Call (used in syscalls) */
  b   trap_handler          /* Prefetch Abort */
  b   trap_handler          /* Data Abort */
.word  0 /* Empty */
  b   trap_handler           /* IRQ */
  b   fiq_handler            /* FIQ */

fiq_handler:
  bkpt /* FIQs aren't supported yet */
  b fiq_handler

reset_vector:

/* Branch to the kernel init and then to the kernel entrypoint */
_branch_main:
    # Configure VBAR to point to the kernel's interrupt vector table defined above
    mrc p15, 0, r4, c12, c0, 0 /* Read VBAR into r4 */
    ldr r4, =_exception_vectors_kernel
    mcr p15, 0, r4, c12, c0, 0

    /* Clear all faults if any */
    mov ip, #0
    mcr p15, 0, ip, c6, c0, 0
    mcr p15, 0, ip, c6, c0, 2
    mcr p15, 0, ip, c5, c0, 0
    mcr p15, 0, ip, c5, c0, 1

    /* Save received stack pointer for later use */
    mov r4, sp

    /* Configure IRQ/FIQ stack pointers */
    msr cpsr_c, #0x12
    ldr sp, =({IRQ_STACK_TOP})
    ldr sp, [sp]
    msr cpsr_c, #0x11
    ldr sp, =({IRQ_STACK_TOP})
    ldr sp, [sp]
    msr cpsr_c, #0x17
    ldr sp, =({EXCEPTION_STACK_TOP})
    ldr sp, [sp]
    msr cpsr_c, #0x1b
    ldr sp, =({EXCEPTION_STACK_TOP})
    ldr sp, [sp]

    msr cpsr_c, #(0x1f | 0x80 | 0x40) /* Switch to System mode (IRQs disabled) */

    /* Set System mode stack */
    mov sp, r4

    /* Note that registers r0-r3 still contain the kernel arguments */
    ldr lr, =(init)
    blx lr
    b .

/*
    Syscall/IRQ/Abort trap handler.

    Saves all process context into a structure that is
    mapped to each thread at THREAD_CONTEXT_AREA.
    Return happens elsewhere.
*/
trap_handler:

    ldr sp, =({THREAD_CONTEXT_AREA})
    ldr sp, [sp]
    str lr, [sp]            /* Store LR into scratch area before clobbering it */
    ldr lr, [sp, #4]        /* Load current context number */
    add sp, sp, lr, lsl #9  /* ip = THREAD_CONTEXT_AREA + (context_number * 128) */
    stm sp, {{r0 - r14}}^   /* 0-14: Saved user registers (R0 and R1 saved below) */
    add sp, #(4 * 15)

    ldr lr, =({THREAD_CONTEXT_AREA})
    ldr lr, [lr]
    ldr lr, [lr]
    stm sp!, {{lr}}         /* 15: This context's LR is the PC of the original one */

    # Special registers
    mrs r1, spsr            /* 16: Saved PSR */
    mrc p15, 0, r2, c13, c0, 2  /* 17: Hardware thread pointer register */
    vmrs r3, fpscr          /* 18: Floating point control reg */
    stm sp!, {{r1 - r3}}    /* Save special registers read above */

    # Floating point
    vstm sp!, {{d0 - d15}}  /* 19-50: FPU registers */
    vstm sp!, {{d16 - d31}} /* 51-82: FPU registers */

    # Set a default stack pointer
    ldr sp, =({EXCEPTION_STACK_TOP})
    ldr sp, [sp]

    # Choose where to go next depending on the current mode: SVC (syscall), abort or IRQ
    mrs ip, cpsr                /* Access the current mode register */
    and ip, ip, #0x1f           /* Extract the mode field */

    cmp ip, #0x13               /* Compare with 'SVC' mode */
    ldreq lr, =(_swi_handler_rust)
    bxeq lr                     /* Jump to SVC handler */

    cmp ip, #0x17               /* Compare with 'Abort' mode */
    ldreq lr, =(_abort_handler_rust)
    bxeq lr                     /* Jump to abort handler */

    cmp ip, #0x1b               /* Compare with 'Undefined' mode */
    ldreq lr, =(_undef_handler_rust)
    bxeq lr                     /* Jump to undefined instruction handler */

    cmp ip, #0x12               /* Compare with 'IRQ' mode */
    bne _skip_irq_handler

    msr cpsr_c, #(0x13 | 0x80 | 0x40) /* Handle IRQs in supervisor mode (IRQs disabled) */
    ldr sp, =({IRQ_STACK_TOP})        /* Reset the stack */
    ldr sp, [sp]
    ldr lr, =(_irq_handler_rust)
    bx lr                       /* Jump to IRQ handler */

    _skip_irq_handler:
    # Any other unhandled exception goes here
    bkpt
    b .

/*
   A thread resume routine.
*/
.align 4
.thumb
.global _resume_trampoline
_resume_trampoline:
    bx pc
    nop
    .arm
    add ip, r0, #(4 * 15)
    ldm ip!, {{lr}}             /* 15: Resume address */
    ldm ip!, {{r1-r3}}          /* Load special registers in one go*/
    msr spsr_cxsf, r1           /* 16: Saved PSR */
    mcr p15, 0, r2, c13, c0, 2  /* 17: Hardware thread pointer register */
    vmsr    fpscr, r3           /* 18: Floating point control */

    vldm    ip!, {{d0 - d15}}   /* 19-50: FPU registers */
    vldm    ip!, {{d16 - d31}}  /* 51-82: FPU registers */

    ldm r0, {{r0 - r14}}^       /* 0-14: Saved user registers */

    movs pc, lr

.global __aeabi_unwind_cpp_pr0
__aeabi_unwind_cpp_pr0:
 b .

