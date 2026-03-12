#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
# SPDX-License-Identifier: GPL-3.0-or-later

set -euo pipefail

# Run `python3 -m serial.tools.list_ports -v` to list available serial ports
CONSOLE_DEV="${KEYOS_CONSOLE_DEV:-/dev/tty.usbserial-1140}"   # allow using custom TTY with an env var

HOME=$KEYOS_HOME minicom -D ${CONSOLE_DEV} -b 115200
