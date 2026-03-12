<!--
SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
SPDX-License-Identifier: GPL-3.0-or-later
-->

# `gpio`

A KeyOS server that exclusively owns and controls the digital I/O (PIO) of the SAMA5D2x MPU.

Allows permitted clients to:

- Claim individual predefined GPIO pins and configure them as input, output or an IRQ source
- Registering a server (`SID`) to receive messages about IRQs occurring on the pins
- Drive the output state of the pin

The `gpio` server and its API guarantees that:

- Every GPIO pin can only be claimed by one client
- No client can access GPIO pins claimed by other clients
- No client can receive IRQ messages for pins it did not claim
- No client can misuse GPIO pins. E.g. using a pin claimed as an input to drive a GPIO output
