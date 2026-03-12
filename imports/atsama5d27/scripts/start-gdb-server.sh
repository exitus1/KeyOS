#!/usr/bin/env bash

# SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
# SPDX-License-Identifier: GPL-3.0-or-later

set -euo pipefail

if [[ -v JLINK_GDB_SERVER_PATH ]] then
    GDB_SERVER=${JLINK_GDB_SERVER_PATH}
else
    GDB_SERVER="$(which JLinkGDBServer 2>/dev/null || echo '/Applications/SEGGER/JLink_V786d/JLinkGDBServer')"
fi

GDB_SPEED_KHZ=100000
$GDB_SERVER  -select USB -device ATSAMA5D27C-CU -endian little -if JTAG -speed $GDB_SPEED_KHZ -noir -noLocalhostOnly \
  -nologtofile -port 3334 -SWOPort 2311 -TelnetPort 2333
