#!/bin/bash

# SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
# SPDX-License-Identifier: GPL-3.0-or-later

set -euo pipefail

openssl ecparam -noout -genkey -name secp256k1 > cosign2-priv.pem
PUBLIC_KEY="$(cat cosign2-priv.pem | openssl ec -pubout -conv_form compressed -outform DER | tail -c 33 | xxd -p -c 65)"

echo "
pubkey = \"$PUBLIC_KEY\"
secret = \"$PWD/cosign2-priv.pem\"
known_pubkeys = []
target = \"atsama5d27-keyos\"
" >cosign2.toml 
