#!/usr/bin/env bash
set -e
cd "$(dirname "$0")"

BIN=./target/release/solochain-template-node
SPEC=chain-specs/calibre-testnet-raw.json
BASE=/tmp/calibre-7val-mesh
LOG=/tmp/calibre-7val-mesh-logs

pkill -9 -f solochain-template-node 2>/dev/null || true
sleep 2
rm -rf "$BASE" "$LOG"
mkdir -p "$BASE" "$LOG"

NODES=(
  "Alice 30333 9944"
  "Bob 30334 9945"
  "Charlie 30335 9946"
  "Dave 30336 9947"
  "Eve 30337 9948"
  "Ferdie 30338 9949"
  "One 30339 9950"
)

echo "=== Step 1: generate node keys + keystores ==="
declare -A PEERS
for entry in "${NODES[@]}"; do
  NAME=$(echo $entry | awk '{print $1}')
  BP="$BASE/$NAME"
  mkdir -p "$BP"

  PEER=$("$BIN" key generate-node-key --chain "$SPEC" --base-path "$BP" 2>&1 | grep -oE '12D3KooW[A-Za-z0-9]+' | head -1)
  "$BIN" key insert --chain "$SPEC" --base-path "$BP" --scheme Sr25519 --suri "//$NAME" --key-type aura >/dev/null 2>&1
  "$BIN" key insert --chain "$SPEC" --base-path "$BP" --scheme Ed25519 --suri "//$NAME" --key-type gran >/dev/null 2>&1
  PEERS[$NAME]="$PEER"
  echo "  $NAME -> $PEER"
done

echo ""
echo "=== Step 2: launch all 7 with FULL-MESH reserved-nodes ==="
echo "    (one --reserved-nodes flag per peer — Substrate rejects comma lists)"
for entry in "${NODES[@]}"; do
  NAME=$(echo $entry | awk '{print $1}')
  PORT=$(echo $entry | awk '{print $2}')
  RPC=$(echo $entry | awk '{print $3}')
  PROM=$((9615 + PORT - 30333))
  BP="$BASE/$NAME"

  # Build an ARRAY of flags
  RESERVED_FLAGS=()
  for other in "${NODES[@]}"; do
    OTHER_NAME=$(echo $other | awk '{print $1}')
    OTHER_PORT=$(echo $other | awk '{print $2}')
    if [ "$OTHER_NAME" != "$NAME" ]; then
      RESERVED_FLAGS+=(--reserved-nodes "/ip4/127.0.0.1/tcp/$OTHER_PORT/p2p/${PEERS[$OTHER_NAME]}")
    fi
  done

  nohup "$BIN" --chain "$SPEC" --validator --force-authoring --name "$NAME" \
    --base-path "$BP" --port "$PORT" --rpc-port "$RPC" --prometheus-port "$PROM" \
    "${RESERVED_FLAGS[@]}" \
    --reserved-only \
    > "$LOG/$NAME.log" 2>&1 &
  echo "  $NAME started on :$PORT with 6 reserved-peers flags"
  sleep 0.4
done

echo ""
echo "=== Step 3: waiting 30s for mesh to form ==="
sleep 30

echo ""
echo "════════ Node status ════════"
printf "  %-8s %-8s %-8s %s\n" "NAME" "BLOCK" "PEERS" "RPC"
for entry in "${NODES[@]}"; do
  NAME=$(echo $entry | awk '{print $1}')
  RPC=$(echo $entry | awk '{print $3}')
  BLOCK=$(curl -s --max-time 2 http://127.0.0.1:$RPC -H 'Content-Type: application/json' \
    -d '{"jsonrpc":"2.0","id":1,"method":"chain_getHeader","params":[]}' 2>/dev/null \
    | python3 -c "import sys,json;print(int(json.load(sys.stdin)['result']['number'],16))" 2>/dev/null || echo "?")
  PEERS=$(curl -s --max-time 2 http://127.0.0.1:$RPC -H 'Content-Type: application/json' \
    -d '{"jsonrpc":"2.0","id":1,"method":"system_health","params":[]}' 2>/dev/null \
    | python3 -c "import sys,json;print(json.load(sys.stdin)['result']['peers'])" 2>/dev/null || echo "?")
  printf "  %-8s %-8s %-8s :%s\n" "$NAME" "$BLOCK" "$PEERS" "$RPC"
done

echo ""
echo "Stop: pkill -9 -f solochain-template-node"
echo "Logs: tail -f $LOG/Alice.log"
