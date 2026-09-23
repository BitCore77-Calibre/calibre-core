#!/usr/bin/env bash
# First-boot setup script for a Calibre WAN validator host.
# Idempotent: safe to run twice.

set -euo pipefail

if ! command -v docker >/dev/null 2>&1; then
    echo "==> installing docker"
    curl -fsSL https://get.docker.com | sh
    usermod -aG docker "$USER" || true
fi

echo "==> loading calibre-node image"
docker load -i calibre-node-image.tar

echo "==> docker ready:"
docker images calibre-node:staging
