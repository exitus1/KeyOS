/*
 * Copyright (C) 2006 Microchip Technology Inc. and its subsidiaries
 *
 * SPDX-License-Identifier: MIT
 */

OUTPUT_FORMAT("elf32-littlearm", "elf32-littlearm", "elf32-littlearm")
OUTPUT_ARCH(arm)
ENTRY(reset)
SECTIONS
{
        . = ALIGN(4);
	.text : { 
		_stext = .;
		*(.text)
		*(.text*)
		*(.ARM.exidx.text.*)
		*(.rodata)                 /* read-only data (constants) */
		*(.rodata*)
		. = ALIGN(4);
		*(.glue_7)
		. = ALIGN(4);		
		*(.eh_frame)
 		_etext = . ;
		_edummy = .;
	}

	.data : {
		_sdata = .;
		*(.vectors)
		*(.data)
		*(.data.*)
		_edata = .;
		ASSERT(_sdata == _edata, " !! RW data sections break hash calculation. Use zero-init on statics. !!");
	}

	/* collect all uninitialized .bss sections */
	.bss (NOLOAD) : { 
		. = ALIGN(4);
		_sbss = .;
		*(.bss)
		*(.bss.*)
		_ebss = .;
	}
}
_romsize = _edata - _stext;
_sramsize = _ebss - _stext;
end = .;  /* define a global symbol marking the end of application */

