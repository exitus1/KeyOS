<!--
SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
SPDX-License-Identifier: GPL-3.0-or-later
-->

# `i2c`

A KeyOS server that exclusively owns and controls the I2C (TWI) bus of the SAMA5D2x MPU.

Allows permitted clients to:

- Claim individual predefined I2C bus slaves
- Read and write the registers of claimed I2C bus slaves

The `i2c` server and its API guarantees that:

- Every slave can only be claimed by one client
- No client can access the slaves claimed by other clients
