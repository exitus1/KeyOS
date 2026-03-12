#!/bin/bash

# SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
# SPDX-License-Identifier: GPL-3.0-or-later

# Clear SAM-BA environment variables
# Usage: source scripts/clear-samba-env.sh

# Unset all SAM-BA related environment variables
unset SAMBA_CIPHER_TOOL
unset SAMBA_CUSTOMER_KEY
unset SAMBA_CIPHER_LICENSE
unset SAMBA_CIPHER_LICENSE_KEY
unset SAMBA_PASSWORD_FILE
unset EXTRA_ENTROPY

echo "SAM-BA environment variables cleared."
