#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundationdevices.com>
# SPDX-License-Identifier: GPL-3.0-or-later

set -euo pipefail

# Run `python3 -m serial.tools.list_ports -v` to list available serial ports
# Specify your devboard console device file name here
CONSOLE_DEV="${KEYOS_CONSOLE_DEV:-/dev/tty.usbserial-1140}"   # allow using custom TTY with an env var

python3 -m serial.tools.miniterm ${CONSOLE_DEV} 115200
