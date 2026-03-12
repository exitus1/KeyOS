// Copyright (C) 2006 Microchip Technology Inc. and its subsidiaries
//
// SPDX-License-Identifier: MIT

#include "common.h"
#include "board.h"
#include "usart.h"
#include "slowclk.h"
#include "board_hw_info.h"
#include "tz_utils.h"
#include "pm.h"
#include "act8865.h"
#include "mcp16502.h"
#include "backup.h"
#include "secure.h"
#include "autoconf.h"
#include "optee.h"
#include "sfr_aicredir.h"

#ifdef CONFIG_CACHES
#include "l1cache.h"
#endif

#ifdef CONFIG_MMU
#include "mmu.h"
static unsigned int *tlb = (unsigned int *)MMU_TABLE_BASE_ADDR;
#endif

// **************** Foundation Devices ************
typedef unsigned char bool;
#include "ffi.h"
#include "string.h"
#include "debug.h"
#include "sdcard.h"

// **************** Foundation Devices ************

#ifdef CONFIG_HW_DISPLAY_BANNER
static void
display_banner(void)
{
	usart_puts(BANNER);
}
#endif

int main(void)
{
#ifdef CONFIG_LOAD_SW
	struct image_info image;
#endif
	hw_init();

#ifdef CONFIG_OCMS_STATIC
	ocms_init_keys();
	ocms_enable();
#endif

#if defined(CONFIG_SCLK)
#if !defined(CONFIG_SCLK_BYPASS)
	slowclk_enable_osc32();
#endif
#elif defined(CONFIG_SCLK_INTRC)
	slowclk_switch_rc32();
#endif

#ifdef CONFIG_BACKUP_MODE
	int ret = backup_mode_resume();
	if (ret)
	{
		/* Backup+Self-Refresh mode detected... */
#ifdef CONFIG_REDIRECT_ALL_INTS_AIC
		redirect_interrupts_to_nsaic();
#endif
		slowclk_switch_osc32();

		/* ...jump to Linux here */
		return ret;
	}
	usart_puts("Backup mode enabled\n");
#endif

#ifdef CONFIG_HW_DISPLAY_BANNER
	display_banner();
#endif

#ifdef CONFIG_REDIRECT_ALL_INTS_AIC
	redirect_interrupts_to_nsaic();
#endif

#ifdef CONFIG_LOAD_HW_INFO
	load_board_hw_info();
#endif

#ifdef CONFIG_PM
	at91_board_pm();
#endif

#ifdef CONFIG_ACT8865
	act8865_workaround();

	act8945a_suspend_charger();
#endif

#ifdef CONFIG_MCP16502_SET_VOLTAGE
	mcp16502_voltage_select();
#endif

#ifdef CONFIG_SAMA7G5
	hw_postinit();
#endif

	init_load_image(&image);

#if defined(CONFIG_SECURE)
	image.dest -= sizeof(at91_secure_header_t);
#endif

#ifdef CONFIG_MMU
	mmu_tlb_init(tlb);
	mmu_configure(tlb);
	mmu_enable();
#endif
#ifdef CONFIG_CACHES
	icache_enable();
	dcache_enable();
#endif

	// **************** Foundation Devices ************
	ffi_random_boot_delay();
	ffi_bootloader_entrypoint();

	while (true)
	{
	}
	// **************** Foundation Devices ************
	return 0;
}

#define HEADER_SIZE 2048

// **************** Foundation Devices ************
unsigned int load_os_image_file(const uint8_t *image_name, const bool header_only)
{
	unsigned int loaded_size = 0;
	struct image_info image;
	init_load_image(&image);
	strcpy(image.filename, image_name);

	if (load_sdcard(&image, header_only ? HEADER_SIZE : ffi_get_os_image_max_size(), true, &loaded_size) == 0)
	{
		return loaded_size;
	}

	return 0;
}
// **************** Foundation Devices ************
