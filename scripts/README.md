<!--
# SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
# SPDX-License-Identifier: GPL-3.0-or-later
-->

This is a set of helpful scripts that I use to have a reproducible setup.

### Requirements

- SEGGER [JLink software](https://www.segger.com/downloads/jlink/) to be installed
- [`arm-none-eabi`](https://developer.arm.com/downloads/-/gnu-rm) toolchain
- `pip3 install pyserial` for UART console that uses `serial.tools.miniterm`

Edit each `.sh` script to ensure it's configured for your environment.

### The flow

Open *three* separate terminal tabs/windows from the project root directory:

1. In the first tab, run `pushd scripts; ./uart-console.sh; popd` to open a device UART console
2. In the second tab, run `JLinkGDBServer  -device ATSAMA5D27C-CU -if jtag -port 3334 -speed 5000` to connect to the board via JTAG and start the GDB server
3. In the third tab, run `cargo xtask run` to build and run a debug binary.

### Flashing the eMMC image with `sam-ba`

Copy the boot.img into the folder where you have the `sam-ba` tool installed. Then run the following commands:

```bash
sam-ba -t 5 -p usb -d sama5d2:3:2 -a sdmmc:0:1:0:8:3 -c write:boot.img
sam-ba -t 5 -p usb -d sama5d2:3:2 -a sdmmc:0:1:0:8:3 -c verify:boot.img
```

Then power cycle the board.
