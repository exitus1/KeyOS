// Copyright (C) 2006 Microchip Technology Inc. and its subsidiaries
//
// SPDX-License-Identifier: MIT

#include "autoconf.h"
#include "common.h"
#include "hardware.h"
#include "board.h"

#include "string.h"

#include "ff.h"

#include "debug.h"

// **************** Foundation Devices ************
#include "ffi.h"

// The single physical eMMC drive is split into two partitions:
// 0: phys0, logic1: Boot Volume (splash, recovery OS)
// 1: phys0, logic2: System Volume (KeyOS, apps, etc.)
const PARTITION VolToPart[_VOLUMES] = {
    { pd: 0, pt: 1 },
    { pd: 0, pt: 2 },
};
// **************** Foundation Devices ************

#define CHUNK_SIZE 0x40000
#define PROGRESS_BAR_UPDATE_LEN 0x100000

// **************** Foundation Devices ************
static int sdcard_loadimage(char *filename, BYTE *dest, unsigned int max_size, int show_progress, unsigned int *size)
// **************** Foundation Devices ************
{
	FIL file;
	UINT byte_to_read;
	UINT byte_read;
	FRESULT fret;
	FILINFO filinfo = {0};
	int ret;

	fret = f_stat(filename, &filinfo);
	if (fret != FR_OK)
	{
		dbg_info("*** FATFS: f_stat, filename: [%s]: error %d\n", filename, fret);
		ret = -1;
		goto open_fail;
	}

	// **************** Foundation Devices ************
	*size = min(filinfo.fsize, max_size);
	// **************** Foundation Devices ************

	fret = f_open(&file, filename, FA_OPEN_EXISTING | FA_READ);
	if (fret != FR_OK)
	{
		dbg_info("*** FATFS: f_open, filename: [%s]: error\n", filename);
		ret = -1;
		goto open_fail;
	}

	UINT total_read = 0;
	UINT last_progress = 0;
	UINT progress_max = *size;

	do
	{
		byte_to_read = min(CHUNK_SIZE, *size - total_read);
		byte_read = 0;
		fret = f_read(&file, (void *)(dest), byte_to_read, &byte_read);
		dest += byte_read;
		total_read += byte_read;

		// **************** Foundation Devices ************
		if (show_progress && total_read > last_progress + PROGRESS_BAR_UPDATE_LEN)
		{
			ffi_set_progress_bar(total_read, progress_max);
			last_progress = total_read;
		}
		// **************** Foundation Devices ************
	} while (*size > total_read && byte_read > 0);

	// **************** Foundation Devices ************
	// Reset progress bar to 100% after loading
	ffi_set_progress_bar(total_read, total_read);
	// **************** Foundation Devices ************

	if (fret != FR_OK)
	{
		dbg_info("*** FATFS: f_read: error\n");
		ret = -1;
		goto read_fail;
	}
	ret = 0;

read_fail:
	fret = f_close(&file);

open_fail:
	return ret;
}

#ifdef CONFIG_OVERRIDE_CMDLINE_FROM_EXT_FILE
static int sdcard_read_cmd(char *cmdline_file, char *cmdline_args)
{
	FIL file;
	UINT byte_to_read = CMDLINE_BUF_LEN;
	UINT byte_read;
	FRESULT fret;
	int ret;

	fret = f_open(&file, cmdline_file, FA_OPEN_EXISTING | FA_READ);
	if (fret != FR_OK)
	{
		dbg_info("*** FATFS: f_open, filename: [%s]: error %d\n",
				 cmdline_file, fret);
		ret = -1;
		goto open_fail;
	}

	do
	{
		byte_read = 0;
		fret = f_read(&file, (char *)(cmdline_args), byte_to_read,
					  &byte_read);
	} while (0);

	dbg_info("SD/MMC: kernel arg string: %s\n", cmdline_args);

	if (fret != FR_OK)
	{
		dbg_info("*** FATFS: cmdline f_read: error\n");
		ret = -1;
		goto read_fail;
	}

	ret = 0;

read_fail:
	fret = f_close(&file);

open_fail:
	return ret;
}
#endif

// **************** Foundation Devices ************
int load_sdcard(struct image_info *image, unsigned int max_size, int show_progress, unsigned int *size)
// **************** Foundation Devices ************
{
	FATFS fs;
	FRESULT fret;
	int ret;
	static bool initialized = false;

	if (!initialized)
	{
#ifdef CONFIG_AT91_MCI
#if defined(CONFIG_AT91_MCI0)
		at91_mci0_hw_init();
#elif defined(CONFIG_AT91_MCI1)
		at91_mci1_hw_init();
#elif defined(CONFIG_AT91_MCI2)
		at91_mci2_hw_init();
#endif
#endif

#ifdef CONFIG_SDHC
		at91_sdhc_hw_init();
#endif
		initialized = true;
	}

	/* get logical drive number from the path name */
	unsigned char vol = image->filename[0] - '0'; // Is there a drive number?
	if (vol > 9 || image->filename[1] != ':') { // No drive number is given
		vol = 0; // Use drive 0
	}

	/* mount fs */
	fret = f_mount(vol, &fs);
	if (fret != FR_OK)
	{
		dbg_info("*** FATFS: f_mount mount error **\n");
		return -1;
	}

	dbg_info("SD/MMC: Image: Read file %s (volume %d) to %x\n",
			 image->filename, vol, image->dest);

	// **************** Foundation Devices ************
	ret = sdcard_loadimage(image->filename, image->dest, max_size, show_progress, size);
	// **************** Foundation Devices ************
	if (ret)
	{
		(void)f_mount(vol, NULL);
		return ret;
	}

#ifdef CONFIG_OF_LIBFDT
	if (image->of_dest)
	{
		at91_board_set_dtb_name(image->of_filename);

		if (strcmp(CONFIG_OF_OVERRIDE_DTB_NAME, ""))
		{
			strcpy(image->of_filename, CONFIG_OF_OVERRIDE_DTB_NAME);
		}

		dbg_info("SD/MMC: dt blob: Read file %s to %x\n",
				 image->of_filename, image->of_dest);

		ret = sdcard_loadimage(image->of_filename, image->of_dest, max_size);
		if (ret)
		{
			(void)f_mount(vol, NULL);
			return ret;
		}
	}

#endif

#ifdef CONFIG_OVERRIDE_CMDLINE_FROM_EXT_FILE
	if (image->cmdline_args)
	{
		dbg_info("SD/MMC: kernel arg string: Read file %s\n",
				 image->cmdline_file);

		ret = sdcard_read_cmd(image->cmdline_file, image->cmdline_args);
		if (ret)
		{
			(void)f_mount(vol, NULL);
			return ret;
		}
	}

#endif

	/* umount fs */
	fret = f_mount(vol, NULL);
	if (fret != FR_OK)
	{
		dbg_info("*** FATFS: f_mount umount error **\n");
		return -1;
	}

	return 0;
}
