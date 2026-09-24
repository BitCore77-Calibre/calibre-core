#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

# Build the node and its embedded Wasm from the checked-out source when the
# operator starts this command. This script never runs during code preparation.
CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-6}" \
WASM_BUILD_RUSTFLAGS="${WASM_BUILD_RUSTFLAGS:--C link-arg=--allow-undefined}" \
    cargo build --release --locked -p solochain-template-node

exec python3 tools/phase94_local_test.py "$@"
