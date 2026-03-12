// Copyright (C) 2021 Microchip Technology Inc. and its subsidiaries
//
// SPDX-License-Identifier: MIT

#include "hardware.h"
#include "arch/at91_sfrbu.h"

/**
 * sfrbu_ba_power_source_auto: set backup area power source to hardware-controlled.
 *
 * Returns:	void
 */
void sfrbu_ba_power_source_auto()
{
    /*
        SCTRL:   0, Power Switch BU is controlled by hardware
        SSWCTRL: 0, (unused)
        SMCTRL:  0,  No automatic supply source switching from security module.
    */
	writel(AT91C_PSWBU_PSWKEY, AT91C_BASE_SFRBU + SFRBU_PSWBU);
}

/**
 * sfrbu_ddr_is_powered: get DDR power mode
 *
 * Returns: 	1 - DDR ON
 * 		0 - DDR OFF
 */
int sfrbu_ddr_is_powered(void)
{
	unsigned int val = readl(AT91C_BASE_SFRBU + SFRBU_DDRBUMCR);

	return !(val & AT91C_DDRBUMCR_BUMEN);
}

/**
 * sfrbu_set_ddr_power_mode: set DDR power mode
 * @on:		if 1 set DDR to power mode ON
 * 		if 0 set DDR to power mode OFF
 *
 * Returns:	void
 */
void sfrbu_set_ddr_power_mode(int on)
{
	writel(!on, AT91C_BASE_SFRBU + SFRBU_DDRBUMCR);
}

