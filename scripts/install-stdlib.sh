#!/usr/bin/env bash

# SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
# SPDX-License-Identifier: GPL-3.0-or-later

set -euo pipefail

TARGET=armv7a-unknown-xous-elf
SYSROOT="$(rustc --print sysroot)"
RUST_VERSION="$(rustc --version | cut -d' ' -f 2)"

echo "Looking for released stdlib for rust version $RUST_VERSION ($TARGET)" >&2

TOOLCHAIN_URL="$(curl -Ls \
    --header "X-GitHub-Api-Version: 2022-11-28" \
    --header "Accept: application/vnd.github+json" \
    https://api.github.com/repos/Foundation-Devices/rust-keyos/releases | \
    jq -r "first(.[] | select(.tag_name|startswith(\"$RUST_VERSION\"))) | .assets[0].url"
)"

echo "Downloading $TOOLCHAIN_URL" >&2
curl -L \
    --header "X-GitHub-Api-Version: 2022-11-28" \
    --header "Accept: application/octet-stream" \
    -o "rust-stdlib-$TARGET.zip" \
    "$TOOLCHAIN_URL"

echo "Extracting into $SYSROOT" >&2
rm -fr "$SYSROOT/lib/rustlib/$TARGET"
unzip -q -d "$SYSROOT" "rust-stdlib-$TARGET.zip"
rm "rust-stdlib-$TARGET.zip"

echo "Done. Have fun compiling!" >&2
