# SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundationdevices.com>
# SPDX-License-Identifier: GPL-3.0-or-later

#
# Build and rust FFI library
#
TARGET=armv7a-none-eabi
FFIDIR=../../target/$(TARGET)/bootloader
INCL += -I../common
FFI_LIB ?= keyos_boot
