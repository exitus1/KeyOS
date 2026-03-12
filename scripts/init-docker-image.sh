#!/usr/bin/bash

# SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
# SPDX-License-Identifier: GPL-3.0-or-later
set -euo pipefail

export TZ=America/New_York
export DEBIAN_FRONTEND=noninteractive

if [[ ${1:-} != "from-dockerfile" ]]; then
    echo "This script is used to init the docker image from inside." >&2
    echo "Run scripts/run-in-docker.sh to build and run the docker image." >&2
    exit 1
fi

set -x

apt update
apt upgrade -y --no-install-recommends
apt install -y --no-install-recommends \
    bc \
    build-essential \
    ca-certificates \
    clang \
    cmake \
    curl \
    gcc-arm-none-eabi \
    pkgconf \
    libpcsclite-dev \
    gh \
    git \
    jq \
    just \
    libclang-dev \
    libfontconfig \
    libnewlib-arm-none-eabi \
    openssh-client \
    pkg-config \
    reuse \
    sudo \
    unzip \
    xxd \

# Make the layer a bit smaller
apt clean
rm -rf /var/lib/apt/lists/*

# Allow raw uid setting for sudo in docker_run_wrapper.sh
echo "Defaults runas_allow_unknown_id" >>/etc/sudoers

curl --proto '=https' --tlsv1.3 https://sh.rustup.rs -sSf | sh -s -- -y

rustup target add armv7a-none-eabi
./install-stdlib.sh

cargo install cargo-binutils cargo-audit cargo-sort slint-lsp

# Install cosign2
cargo install --path cosign2/cosign2-bin --bin cosign2

# Setup-git checks for the existence of a token but doesn't actually use it.
# This fake token is not stored and will be re-read with each command.
GH_TOKEN="fake_token" gh auth setup-git

# Give r/w access and execute on directories to everyone so that we can run builds
# with different UIDs while using root's cargo cache
chmod -R ag+rwX /root

usermod -d /root ubuntu
