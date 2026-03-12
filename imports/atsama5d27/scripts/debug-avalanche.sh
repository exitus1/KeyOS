#!/usr/bin/env bash

# SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundationdevices.com>
# SPDX-License-Identifier: MIT OR Apache-2.0

set -euo pipefail

cargo build --release --bin avalanche

GDB="${KEYOS_GDB:-arm-none-eabi-gdb}"    # allow using custom gdb with an env var

$GDB -q ../target/armv7a-none-eabi/release/avalanche -x init.gdb
