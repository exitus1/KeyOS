// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later
#ifndef _AT91BOOTSTRAP_FFI
#define _AT91BOOTSTRAP_FFI

#define uint32_t unsigned int
#define uint8_t char

extern uint32_t load_os_image_file(const uint8_t *image_name, bool header_only);

void ffi_bootloader_entrypoint(void);

uint32_t ffi_get_os_image_max_size(void);

void ffi_random_boot_delay(void);

void ffi_set_progress_bar(uint32_t curr, uint32_t total);

#endif /* _AT91BOOTSTRAP_FFI */
