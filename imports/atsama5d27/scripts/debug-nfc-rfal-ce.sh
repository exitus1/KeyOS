#!/usr/bin/env bash

set -e

cargo build --release --bin nfc-rfal --features nfc-rfal,nfc-ce --target armv7a-none-eabi
arm-none-eabi-gdb -q ../../../target/armv7a-none-eabi/release/nfc-rfal -x init.gdb
