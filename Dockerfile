# SPDX-FileCopyrightText: 2025 Foundation Devices Inc.
#
# SPDX-License-Identifier: GPL-3.0-or-later

FROM ubuntu:24.04

ENV \
    PATH=/root/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin \
    CARGO_NET_GIT_FETCH_WITH_CLI=true \
    HOME=/root

WORKDIR /root

COPY \
    scripts/init-docker-image.sh \
    scripts/install-stdlib.sh \
    rust-toolchain.toml \
    /root/
COPY imports/cosign2 /root/cosign2/

RUN ./init-docker-image.sh from-dockerfile

WORKDIR /src

CMD ["just", "build-all"]
