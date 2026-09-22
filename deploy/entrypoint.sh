#!/usr/bin/env bash
set -euo pipefail

: "${VALIDATOR_NAME:?VALIDATOR_NAME is required}"
: "${KEYRING:?KEYRING is required}"
: "${P2P_PORT:=30333}"
: "${RPC_PORT:=9944}"
: "${PROM_PORT:=9615}"
: "${CHAIN:=staging}"
: "${CHAIN_DIR:=calibre_staging}"   # chain spec id — filesystem dir
: "${BASE_PATH:=/data/$VALIDATOR_NAME}"

BIN=/usr/local/bin/node

mkdir -p "$BASE_PATH"

echo "==> [$VALIDATOR_NAME] base-path: $BASE_PATH"

# Aura key (sr25519) — idempotent
if ! ls "$BASE_PATH"/chains/$CHAIN_DIR/keystore/61757261* >/dev/null 2>&1; then
    echo "==> [$VALIDATOR_NAME] inserting aura key"
    $BIN key insert \
        --base-path "$BASE_PATH" --chain "$CHAIN" \
        --scheme sr25519 --key-type aura --suri "//$KEYRING"
fi

# GRANDPA key (ed25519) — idempotent
if ! ls "$BASE_PATH"/chains/$CHAIN_DIR/keystore/6772616e* >/dev/null 2>&1; then
    echo "==> [$VALIDATOR_NAME] inserting grandpa key"
    $BIN key insert \
        --base-path "$BASE_PATH" --chain "$CHAIN" \
        --scheme ed25519 --key-type gran --suri "//$KEYRING"
fi

# Node network key (ed25519) — idempotent, seed-first
NET_DIR="$BASE_PATH"/chains/$CHAIN_DIR/network
NET_KEY="$NET_DIR"/secret_ed25519
mkdir -p "$NET_DIR"
if [ -f /seed/v1-network-key ]; then
    echo "==> [$VALIDATOR_NAME] seeding node network key from /seed"
    cp /seed/v1-network-key "$NET_KEY"
    chmod 600 "$NET_KEY"
elif [ ! -f "$NET_KEY" ]; then
    echo "==> [$VALIDATOR_NAME] generating node network key"
    $BIN key generate-node-key \
        --base-path "$BASE_PATH" --chain "$CHAIN" 2>/dev/null
fi

BOOT_ARGS=()
if [ -n "${BOOTNODE:-}" ]; then
    BOOT_ARGS=(--bootnodes "$BOOTNODE")
    echo "==> [$VALIDATOR_NAME] bootnode: $BOOTNODE"
fi

echo "==> [$VALIDATOR_NAME] starting node"
exec $BIN \
    --base-path "$BASE_PATH" --chain "$CHAIN" \
    --validator --name "$VALIDATOR_NAME" \
    --port "$P2P_PORT" \
    --rpc-port "$RPC_PORT" \
    --rpc-methods=safe --unsafe-rpc-external --rpc-cors=all \
    --prometheus-port "$PROM_PORT" --prometheus-external \
    "${BOOT_ARGS[@]}"
