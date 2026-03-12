#!/usr/bin/env bash

# SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
# SPDX-License-Identifier: GPL-3.0-or-later

set -euo pipefail

GDB="${KEYOS_GDB:-arm-none-eabi-gdb}"    # allow using custom gdb with an env var

$GDB -q <<SCRIPT
target remote :3334
monitor halt
set  *((unsigned*)0xF8048054) = 0x66830000
monitor reset
SCRIPT
