#!/usr/bin/env bash

# SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
# SPDX-License-Identifier: GPL-3.0-or-later

set -euo pipefail

if [[ $# != 1 ]]; then
    echo "Usage: $0 process_name

    To be used for creating flamegraph charts, e.g.:

    $0 gui-app-authenticator | tee /tmp/trace
    FlameGraph/stackcollapse-gdb.pl </tmp/trace | FlameGraph/flamegraph.pl >flame.svg
    "
    exit 1
fi

GDB="${KEYOS_GDB:-arm-none-eabi-gdb}"    # allow using custom gdb with an env var

$GDB \
    -ex "py process='$1'" \
    -x scripts/backtraces-helper.py \
    -batch
