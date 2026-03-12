#!/usr/bin/env bash

# SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
# SPDX-License-Identifier: GPL-3.0-or-later

set -euo pipefail

docker build --tag 'foundationdevices/keyos-build' .

warn() { echo "[!] $@ " >&2; }
warn_start() { echo >&2; }
warn_end() { echo >&2; }

CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}"
if [ -d "${CARGO_HOME}" ]; then
    MOUNT_CARGO_REGISTRY="
        --mount type=bind,\"source=${CARGO_HOME}/registry\",\"destination=/root/.cargo/registry\" \
        --mount type=bind,\"source=${CARGO_HOME}/git\",\"destination=/root/.cargo/git\" \
    "
else 
    warn_start
    warn "Could not find cargo registry at ${CARGO_HOME}"
    warn "This will make builds slower."
    warn "If you are running with \`sudo\`, make sure you use the \`-E\` flag."
    warn_end
fi

if [ -t 0 ]; then
    INTERACTIVE="-it"
fi

GH_TOKEN=${GH_TOKEN:-${GITHUB_TOKEN:-$(cat .github-access-token 2>/dev/null || cat ~/.github-access-token 2>/dev/null || true)}}
if [[ "${GH_TOKEN}" == "" ]]; then
    warn_start
    warn "GH_TOKEN env var not set, and could not find the .github-access-token file"
    warn "in $PWD and $HOME"
    warn "Cloning private repositories will not work."
    warn ""
    warn "Create the file '.github-access-token' in the root of the repository annd copy"
    warn "the access token into it, or provide it via 'GH_TOKEN' environment variable."
    warn ""
    warn "To create a new GitHub access token, go to the following URL:"
    warn ""
    warn "https://github.com/settings/tokens/new?description=cargo%20xtask%20keyOS&scopes=repo"
    warn_end
fi

if [[ -f "cosign2.toml" ]]; then
    COSIGN_SECRET_FILE=$(grep "secret" cosign2.toml | sed -E 's/ *secret *= *"?([^"]*)"?.*/\1/')
    if [[ $COSIGN_SECRET_FILE != $PWD/* ]]; then
        warn_start
        warn "The secret key in cosign2.toml is not within $PWD"
        warn "Only the current directory is mounted in the docker image, so signing will not work."
        warn_end
    fi
else 
    warn_start
    warn "Could not find cosign2.toml. Image building for hardware will not work."
    warn "Generate it and a key with 'scripts/generate-cosign2-dev-key.sh'"
    warn_end
fi

# Detect the 'inside' UID and GID of the mounted dir (could be remapped)
eval $(
    docker run --rm \
    --mount "type=bind,\"source=$PWD\",\"destination=$PWD\"" \
    --workdir "$PWD" \
    'foundationdevices/keyos-build' \
    stat -c "PWD_UID=%u; PWD_GID=%g" .
)

docker run \
    --rm \
    ${INTERACTIVE:-} \
    --mount "type=bind,\"source=$PWD\",\"destination=$PWD\"" \
    ${MOUNT_CARGO_REGISTRY:-} \
    --user $PWD_UID:$PWD_GID \
    --env "GH_TOKEN=${GH_TOKEN}" \
    --workdir "$PWD" \
    'foundationdevices/keyos-build' \
    $@
