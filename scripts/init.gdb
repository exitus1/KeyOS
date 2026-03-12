# SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
# SPDX-License-Identifier: GPL-3.0-or-later

set pagination off

# MMU and other CP15 stuff
source scripts/mmu.gdb

# Examine memory with hex and ascii
source scripts/xac.gdb

# Add and enable fancy text-based UI
#tui new-layout mylayout {-horizontal src 1 asm 1} 2 status 0 cmd 1
#layout mylayout

# JLinkGDB server is expected to be running at this port
target remote :3334

monitor reset
shell sleep 1
monitor halt

# Put the loader binary (provided in command args) into the memory first
load

# Load the KeyOS image (kernel + initial processes) at the address expected by the loader
eval "restore %s binary %u", $OS_IMG, $OS_ADDRESS

# Load the kernel symbols for the ease of debugging
eval "add-symbol-file %s", $KERNEL_ELF

# Load the symbols for the debugged service
eval "add-symbol-file %s", $SERVICE

set $cpsr = 0x13

c
