#!/usr/bin/env bash
set -e
cd "$(dirname "$0")"

BIN=./target/release/solochain-template-node
SPEC=chain-specs/calibre-testnet-raw.json
BASE=/tmp/calibre-7val
LOG=/tmp/calibre-7val-logs

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

echo "=== Step 1: prepare keystores + node keys ==="
for entry in "${NODES[@]}"; do
  NAME=$(echo $entry | awk '{print $1}')
  BP="$BASE/$NAME"
  mkdir -p "$BP"

  PEER=$("$BIN" key generate-node-key --chain "$SPEC" --base-path "$BP" 2>&1 | grep -oE '12D3KooW[A-Za-z0-9]+' | head -1)
  "$BIN" key insert --chain "$SPEC" --base-path "$BP" --scheme Sr25519 --suri "//$NAME" --key-type aura >/dev/null 2>&1
  "$BIN" key insert --chain "$SPEC" --base-path "$BP" --scheme Ed25519 --suri "//$NAME" --key-type gran >/dev/null 2>&1
  echo "  $NAME -> $PEER"

  if [ "$NAME" = "Alice" ]; then
    echo "$PEER" > "$LOG/alice-peer.txt"
  fi
done

ALICE_PEER=$(cat "$LOG/alice-peer.txt")
echo ""
echo "Alice peer ID: $ALICE_PEER"
echo ""
echo "=== Step 2: launch Alice alone (no reserved-nodes) ==="

"$BIN" --chain "$SPEC" --validator --force-authoring --name Alice \
  --base-path "$BASE/Alice" --port 30333 --rpc-port 9944 --prometheus-port 9615 \
  > "$LOG/Alice.log" 2>&1 &
echo "  Alice started (PID $!)"

sleep 8

echo ""
echo "=== Step 3: launch Bob..One with --reserved-nodes → Alice ==="
for entry in "${NODES[@]:1}"; do
  NAME=$(echo $entry | awk '{print $1}')
  PORT=$(echo $entry | awk '{print $2}')
  RPC=$(echo $entry | awk '{print $3}')
  PROM=$((9615 + PORT - 30333))
  BP="$BASE/$NAME"

  "$BIN" --chain "$SPEC" --validator --force-authoring --name "$NAME" \
    --base-path "$BP" --port "$PORT" --rpc-port "$RPC" --prometheus-port "$PROM" \
    --reserved-nodes "/ip4/127.0.0.1/tcp/30333/p2p/$ALICE_PEER" \
    --reserved-only \
    > "$LOG/$NAME.log" 2>&1 &
  echo "  $NAME started on :$PORT"
  sleep 0.4
done

echo ""
echo "=== Step 4: waiting 25s for consensus ==="
sleep 25

echo ""
echo "════════ Node status ════════"
printf "  %-8s %-8s %-8s %s\n" "NAME" "BLOCK" "PEERS" "RPC"
for entry in "${NODES[@]}"; do
  NAME=$(echo $entry | awk '{print $1}')
  PORT=$(echo $entry | awk '{print $2}')
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
