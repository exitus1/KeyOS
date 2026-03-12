#!/usr/bin/env bash

set -euo pipefail

cargo build --release --bin aesb

GDB="${KEYOS_GDB:-arm-none-eabi-gdb}"    # allow using custom gdb with an env var

$GDB -q ../target/armv7a-none-eabi/release/aesb -x init.gdb
