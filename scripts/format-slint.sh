#!/usr/bin/env bash

# SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
# SPDX-License-Identifier: GPL-3.0-or-later

set -euo pipefail

# Set the base directory, or use the current directory if none is specified
BASE_DIR="${1:-.}"

# Find all .slint files and format them
find "$BASE_DIR" -type f -name "*.slint" | while read -r file; do
    slint-lsp format -i "$file"
done
