#!/usr/bin/env bash

# SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
# SPDX-License-Identifier: GPL-3.0-or-later

set -euo pipefail

# Set the base directory, or use the current directory if none is specified
BASE_DIR="${1:-.}"

# Find all .slint files tracked by git and check their formatting
git ls-files "$BASE_DIR" | grep '\.slint$' | while read -r file; do
    if ! slint-lsp format "$file" | diff -q "$file" - > /dev/null; then
      echo "$file is not properly formatted."
      exit 1
    fi
done
