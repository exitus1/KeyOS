#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
# SPDX-License-Identifier: GPL-3.0-or-later

# Automated slint-viewer wrapper that handles preview setup and cleanup
# Usage: ./scripts/slint-preview.sh [slint-viewer arguments...]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BACKUP_DIR="$PROJECT_ROOT/.slint-preview-backup"

# Files that get modified during preview preparation
PREVIEW_FILES=(
    "ui/ui/images.slint"
    "ui/ui/widgets/card-circle.slint"
    "ui/ui/widgets/line-card.slint"
    "ui/ui/widgets/arc.slint"
    "slint-keyos-platform/runtime/src/lib.rs"
)

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log() {
    echo -e "${GREEN}[slint-preview]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[slint-preview]${NC} $1"
}

error() {
    echo -e "${RED}[slint-preview]${NC} $1" >&2
}

cleanup() {
    local exit_code=$?

    log "Cleaning up and restoring original state..."

    if [ -d "$BACKUP_DIR" ]; then
        # Restore original files
        for file in "${PREVIEW_FILES[@]}"; do
            local backup_file="$BACKUP_DIR/$file"
            local original_file="$PROJECT_ROOT/$file"

            if [ -f "$backup_file" ]; then
                cp "$backup_file" "$original_file"
                log "Restored $file"
            fi
        done

        # Clean up backup directory
        rm -rf "$BACKUP_DIR"
        log "Removed temporary backup directory"
    fi

    if [ $exit_code -eq 0 ]; then
        log "Preview session completed successfully"
    else
        warn "Preview session ended with exit code $exit_code"
    fi
}

# Set up cleanup trap
trap cleanup EXIT

main() {
    cd "$PROJECT_ROOT"

    # Check if we're already in preview mode
    if [ -d "$BACKUP_DIR" ]; then
        error "Preview backup directory already exists. Another preview session may be running."
        error "If you're sure no other session is active, remove: $BACKUP_DIR"
        exit 1
    fi

    log "Starting slint-viewer with automated preview setup..."

    # Create backup directory
    mkdir -p "$BACKUP_DIR"

    # Backup original files
    log "Backing up original files..."
    for file in "${PREVIEW_FILES[@]}"; do
        local original_file="$PROJECT_ROOT/$file"
        local backup_file="$BACKUP_DIR/$file"

        if [ -f "$original_file" ]; then
            mkdir -p "$(dirname "$backup_file")"
            cp "$original_file" "$backup_file"
            log "Backed up $file"
        else
            warn "File not found, skipping: $file"
        fi
    done

    # Run preview preparation
    log "Preparing files for preview..."
    if ! just fix-preview; then
        error "Failed to run 'just fix-preview'"
        exit 1
    fi

    # Run slint-viewer with provided arguments
    log "Starting slint-viewer..."
    log "Command: slint-viewer $*"

    # Run slint-viewer directly (not exec) so cleanup trap can run
    slint-viewer "$@"
}

# Validate that we're in the correct directory
if [ ! -f "$PROJECT_ROOT/Justfile" ] || [ ! -d "$PROJECT_ROOT/ui" ]; then
    error "This script must be run from the KeyOS project root or its scripts directory"
    exit 1
fi

# Check that required commands exist
if ! command -v just >/dev/null 2>&1; then
    error "The 'just' command is not available. Please install it or ensure it's in your PATH."
    exit 1
fi

if ! command -v slint-viewer >/dev/null 2>&1; then
    error "The 'slint-viewer' command is not available. Please install it or ensure it's in your PATH."
    exit 1
fi

main "$@"
