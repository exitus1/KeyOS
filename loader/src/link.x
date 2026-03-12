/*
 * SPDX-FileCopyrightText: 2023 Foundation Devices, Inc <hello@foundation.xyz>
 * SPDX-License-Identifier: Apache-2.0
 */

MEMORY
{
  /* Origin is offset by the cosign2 header */
  RAM : ORIGIN = 0x20000800, LENGTH = 0xff800
}

PROVIDE(_mem_start = ORIGIN(RAM));                    /* Start of the memory */

ENTRY(reset);

REGION_ALIAS("REGION_TEXT", RAM);
REGION_ALIAS("REGION_RODATA", RAM);
REGION_ALIAS("REGION_DATA", RAM);
REGION_ALIAS("REGION_BSS", RAM);

/* Size of the loader stack */
_stack_size = 64K;

SECTIONS
{
  .text :
  {
    *(.text);    /* Place .text section first so that the reset vector is always placed at the load address */
    *(.text.*);  /* Place other code sections next */
  } > REGION_TEXT

  .rodata : ALIGN(4)
  {
    *(.rodata .rodata.*);
    *(.got .got.*);

    /* RELRO */
    *(.data.rel.ro .data.rel.ro.*);

    /* 4-byte align the end (VMA) of this section.
       This is required by LLD to ensure the LMA of the following .data
       section will have the correct alignment. */
    . = ALIGN(4);
    _etext = .;
  } > REGION_RODATA

  .data : ALIGN(4)
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
    /* Leave space for the stack. We are going to use _ebss to set SP */
    . += _stack_size;
    . = ALIGN(4096);
    _ebss = .;
  } > REGION_BSS

  /* Discard .eh_frame, we are not doing unwind on panic so it is not needed */
  /DISCARD/ :
  {
    *(.eh_frame);
    *(.eh_frame_hdr);
  }
}

end = .;  /* define a global symbol marking the end of application */

/* Do not exceed this mark in the error messages below                                    | */
ASSERT(ORIGIN(RAM) % 4 == 0, "
ERROR(arm-rt): the start of the RAM must be 4-byte aligned");

ASSERT(_sdata % 4 == 0 && _edata % 4 == 0, "
BUG(arm-rt): .data is not 4-byte aligned");

ASSERT(_sbss % 4 == 0 && _ebss % 4 == 0, "
BUG(arm-rt): .bss is not 4-byte aligned");

/* Do not exceed this mark in the error messages above                                    | */
