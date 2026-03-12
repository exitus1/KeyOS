// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

/// BQ24157 charger configuration shared between bootloader and `power-manager`.
pub const CHARGER_CONFIG_DUMP: [(u8, u8); 6] = [
    (6, 0x7c), // Safety Limit Register (must be written first)
    (0, 0x50), // Status/Control Register
    (1, 0x78), // Control Register
    (2, 0xb6), // Control/Battery Voltage Register
    (4, 0x00), // Battery Termination/Fast Charge Current Register
    (5, 0x02), // Special Charger Voltage/Enable Pin Status Register
];
