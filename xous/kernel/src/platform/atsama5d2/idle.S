// SPDX-FileCopyrightText: 2022 Foundation Devices, Inc <hello@foundation.xyz>
// SPDX-License-Identifier: Apache-2.0

.thumb

.equ PMC_PLLAR, 0x28
.equ PMC_MCKR, 0x30
.equ PMC_SR, 0x68
.equ PMC_SR_MCKRDY, (1<<3)
.equ PMC_SR_LOCKA, (1<<1)

.equ MPDDRC_LPR, 0x1c

.macro set_mckr_bits mask, bits
    bic r3, #(\mask)
    orr r3, #(\bits)
    str r3, [r1, #PMC_MCKR]
    // Wait until Master Clock stabilizes
    1:
        ldr r12, [r1, #PMC_SR]
        tst r12, #PMC_SR_MCKRDY
    beq 1b
.endm


// Suspend to SRAM with most clocks slowed and disabled, DRAM is unusable in this state.
// Parameters:
//    R0: Virtual address of MPDDRC
//    R1: Virtual address of PMC
suspend_f:
    push {{r4}}

    // Drain the store buffer, so that normal memory stuff all gets done before we start
    // messing with the clock and DRAM access.
    // Further drains should not be needed, because we only access strongly ordered memory
    // from here on out.
    dsb

    // Putting the reads above self-refresh makes sure that these addresses
    // will already be in the TLB when self-refresh is enabled, avoiding a
    // slow translation table lookup.
    ldr r3, [r1, #PMC_MCKR]
    ldr r4, [r1, #PMC_PLLAR]

    // === Set Self-refresh mode in MPDDRC_LPR LPCB ===
    ldr r2, [r0, #MPDDRC_LPR]
    bic r12, r2, #3
    orr r12, #1
    str r12, [r0, #MPDDRC_LPR]
    // Wait until the Self-refresh command is actually issued
    1:
        ldr r12, [r0, #MPDDRC_LPR]
        tst r12, #(1<<25)       // SELF_DONE flag
    beq 1b

    // === Set Master Clock to MAINCK/64 and disable PLLA ===
    set_mckr_bits 0x03 0x01     // Clock Source: main clock

    mov r12, #(1<<29)           // Bit 29 must be set to 1 to write PLLAR
    str r12, [r1, #PMC_PLLAR]   // Disable PLLA

    set_mckr_bits 0x70 0x60     // Prescaler: DIV64

    // === Wait for an interrupt ===
    // This will not run the actual IRQ handlers, since interrupts are disabled,
    // but it will continue execution.
    wfi

    // === Re-enable PLLA-based master Clock ===
    set_mckr_bits 0x70 0x00     // Prescaler: No division

    str r4, [r1, #PMC_PLLAR]
    // Wait until PLL stabilizes
    1:
        ldr r12, [r1, #PMC_SR]
        tst r12, #PMC_SR_LOCKA
    beq 1b

    set_mckr_bits 0x03 0x02     // Clock Source: PLLACK

    // SR_MCKRDY seems a bit too eager to say that MCK is stable, but apparently it needs
    // a few more cycles before it's actually stable enough to not corrupt RAM reads.
    nop
    nop
    nop
    nop

    // === Get RAM out of self-refresh mode ===
    str r2, [r0, #MPDDRC_LPR]

    // Do not try to prefetch instructions from DRAM before all of the above actually happened.
    dsb
    isb

    pop {{r4}}
    bx lr
suspend_end:
