/*
 * SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
 * SPDX-License-Identifier: Apache-2.0
 */

MEMORY
{
    RAM    : ORIGIN = 0xffd00000, LENGTH = 1M
    RTT_CB : ORIGIN = 0xbeef0000, LENGTH = 128K  /* Must be kept in sync with RTT_CONTROL_BLOCK_VIRT_ADDR */
}

ENTRY(reset);

REGION_ALIAS("REGION_TEXT", RAM);
REGION_ALIAS("REGION_RODATA", RAM);
REGION_ALIAS("REGION_DATA", RAM);
REGION_ALIAS("REGION_BSS", RAM);

SECTIONS
{
  .text :
  {
    /* Put reset handler first in .text section so it ends up as the entry */
    /* point of the program. */
    KEEP(*(.text.reset_vector));
    KEEP(*(.text.init));
    KEEP(*(.init));
    KEEP(*(.init.rust));
    . = ALIGN(4);
    KEEP(*(.trap));
    KEEP(*(.trap.rust));

    *(.text .text.*);
  } > REGION_TEXT

  .rodata : ALIGN(4)
  {
    *(.rodata .rodata.*);
    *(.got .got.*);

    /* RELRO */
    *(.data.rel.ro .data.rel.ro.*);

    /* Align end of text+rodata region to 64 bytes for ICM */
    . = ALIGN(64);
    _etext = .;
  } > REGION_RODATA

  .data : ALIGN(4096)
  {
    _sidata = LOADADDR(.data);
    _sdata = .;
    *(.sdata .sdata.* .sdata2 .sdata2.*);
    *(.data .data.*);
    . = ALIGN(4);
    _edata = .;
  } > REGION_DATA AT > REGION_RODATA

  .bss (NOLOAD) :
  {
    _sbss = .;
    *(.sbss .sbss.* .bss .bss.*);
    . = ALIGN(4);
    _ebss = .;
  } > REGION_BSS

  .rtt (NOLOAD) : {
    /* Force placing _SEGGER_RTT control block here */
    *(.rtt_cb_section)
    . = ALIGN(8);
  } > RTT_CB

  /* Discard .eh_frame, we are not doing unwind on panic so it is not needed */
  /DISCARD/ :
  {
    *(.eh_frame);
    *(.eh_frame_hdr);
  }
}

PROVIDE(_romsize = _edata - _stext);
PROVIDE(_sramsize = _ebss - _stext);

/* Do not exceed this mark in the error messages above                                    | */
ASSERT(ORIGIN(RAM) % 4 == 0, "
ERROR(arm-rt): the start of the RAM must be 4-byte aligned");

ASSERT(_sdata % 4 == 0 && _edata % 4 == 0, "
BUG(arm-rt): .data is not 4-byte aligned");

ASSERT(_sbss % 4 == 0 && _ebss % 4 == 0, "
BUG(arm-rt): .bss is not 4-byte aligned");

/* Do not exceed this mark in the error messages above                                    | */
