<!--
SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
SPDX-License-Identifier: GPL-3.0-or-later
-->

# `spi`

A KeyOS server that exclusively owns and controls the SPI bus of the SAMA5D2x MPU.

Allows permitted clients to:

- Claim individual predefined SPI bus slaves
- Interface an [embedded_hal::spi::SpiDevice](https://docs.rs/embedded-hal/1.0.0/embedded_hal/spi/trait.SpiDevice.html) based driver to claimed SPI bus slaves

The `spi` server and its API guarantees that:

- Every slave can only be claimed by one client
- No client can access the slaves claimed by other clients
