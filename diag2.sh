#!/usr/bin/env bash
# NO set -e — we want to keep going after failures
cd "$(dirname "$0")"

BIN=./target/release/solochain-template-node
SPEC=chain-specs/calibre-testnet-raw.json
BASE=/tmp/calibre-d2
LOG=/tmp/calibre-d2-logs

pkill -9 -f solochain-template-node 2>/dev/null
sleep 2
rm -rf "$BASE" "$LOG"
mkdir -p "$BASE" "$LOG"

# ─── Setup Alice ────────────────────────────────────────────────
echo "SETUP: Alice"
ALICE_BP="$BASE/Alice"
mkdir -p "$ALICE_BP"
"$BIN" key generate-node-key --chain "$SPEC" --base-path "$ALICE_BP" 2>&1 | grep -oE '12D3KooW[A-Za-z0-9]+' | head -1 > "$LOG/alice-peer.txt"
"$BIN" key insert --chain "$SPEC" --base-path "$ALICE_BP" --scheme Sr25519 --suri //Alice --key-type aura >/dev/null 2>&1
"$BIN" key insert --chain "$SPEC" --base-path "$ALICE_BP" --scheme Ed25519 --suri //Alice --key-type gran >/dev/null 2>&1

ALICE_PEER=$(cat "$LOG/alice-peer.txt")
echo "Alice peer: $ALICE_PEER"

# ─── Start Alice ────────────────────────────────────────────────
echo ""
echo "START: Alice on :30333"
nohup "$BIN" --chain "$SPEC" --validator --force-authoring --name Alice \
  --base-path "$ALICE_BP" --port 30333 --rpc-port 9944 --prometheus-port 9615 \
  > "$LOG/Alice.log" 2>&1 &
ALICE_PID=$!
echo "Alice PID: $ALICE_PID"

sleep 12

if kill -0 $ALICE_PID 2>/dev/null; then
  echo "Alice ALIVE"
  curl -s --max-time 2 http://127.0.0.1:9944 -H 'Content-Type: application/json' \
    -d '{"jsonrpc":"2.0","id":1,"method":"chain_getHeader","params":[]}' \
    | python3 -c "import sys,json; h=json.load(sys.stdin)['result']; print('Alice block:', int(h['number'],16))" 2>/dev/null
else
  echo "Alice CRASHED"
  tail -20 "$LOG/Alice.log"
fi

# ─── Test 1: Bob with NO bootnodes ─────────────────────────────
echo ""
echo "TEST 1: Bob with NO bootnodes (isolates bootnode as cause)"

BOB_BP="$BASE/Bob"
mkdir -p "$BOB_BP"
"$BIN" key generate-node-key --chain "$SPEC" --base-path "$BOB_BP" >/dev/null 2>&1
"$BIN" key insert --chain "$SPEC" --base-path "$BOB_BP" --scheme Sr25519 --suri //Bob --key-type aura >/dev/null 2>&1
"$BIN" key insert --chain "$SPEC" --base-path "$BOB_BP" --scheme Ed25519 --suri //Bob --key-type gran >/dev/null 2>&1

nohup "$BIN" --chain "$SPEC" --validator --force-authoring --name Bob \
  --base-path "$BOB_BP" --port 30334 --rpc-port 9945 --prometheus-port 9616 \
  > "$LOG/Bob-noboot.log" 2>&1 &
BOB_PID=$!

sleep 12

if kill -0 $BOB_PID 2>/dev/null; then
  echo "TEST 1 RESULT: Bob ALIVE without bootnodes"
  curl -s --max-time 2 http://127.0.0.1:9945 -H 'Content-Type: application/json' \
    -d '{"jsonrpc":"2.0","id":1,"method":"chain_getHeader","params":[]}' \
    | python3 -c "import sys,json; h=json.load(sys.stdin)['result']; print('  Bob block:', int(h['number'],16))" 2>/dev/null
  kill -9 $BOB_PID 2>/dev/null
else
  echo "TEST 1 RESULT: Bob CRASHED without bootnodes"
  echo "  → cause is in Bob's own setup, NOT the network"
fi

# ─── Test 2: Bob WITH bootnode (--bootnodes) ───────────────────
echo ""
echo "TEST 2: Bob with --bootnodes"

BOB_BP="$BASE/Bob2"
mkdir -p "$BOB_BP"
"$BIN" key generate-node-key --chain "$SPEC" --base-path "$BOB_BP" >/dev/null 2>&1
"$BIN" key insert --chain "$SPEC" --base-path "$BOB_BP" --scheme Sr25519 --suri //Bob --key-type aura >/dev/null 2>&1
"$BIN" key insert --chain "$SPEC" --base-path "$BOB_BP" --scheme Ed25519 --suri //Bob --key-type gran >/dev/null 2>&1

nohup "$BIN" --chain "$SPEC" --validator --force-authoring --name Bob2 \
  --base-path "$BOB_BP" --port 30335 --rpc-port 9946 --prometheus-port 9617 \
  --bootnodes "/ip4/127.0.0.1/tcp/30333/p2p/$ALICE_PEER" \
  > "$LOG/Bob-boot.log" 2>&1 &
BOB2_PID=$!

sleep 12

if kill -0 $BOB2_PID 2>/dev/null; then
  echo "TEST 2 RESULT: Bob ALIVE with --bootnodes"
  curl -s --max-time 2 http://127.0.0.1:9946 -H 'Content-Type: application/json' \
    -d '{"jsonrpc":"2.0","id":1,"method":"system_health","params":[]}' \
    | python3 -c "import sys,json; r=json.load(sys.stdin)['result']; print('  peers:', r['peers'])" 2>/dev/null
  kill -9 $BOB2_PID 2>/dev/null
else
  echo "TEST 2 RESULT: Bob CRASHED with --bootnodes"
fi

# ─── Test 3: Bob WITH reserved-nodes ───────────────────────────
echo ""
echo "TEST 3: Bob with --reserved-nodes"

BOB_BP="$BASE/Bob3"
mkdir -p "$BOB_BP"
"$BIN" key generate-node-key --chain "$SPEC" --base-path "$BOB_BP" >/dev/null 2>&1
"$BIN" key insert --chain "$SPEC" --base-path "$BOB_BP" --scheme Sr25519 --suri //Bob --key-type aura >/dev/null 2>&1
"$BIN" key insert --chain "$SPEC" --base-path "$BOB_BP" --scheme Ed25519 --suri //Bob --key-type gran >/dev/null 2>&1

nohup "$BIN" --chain "$SPEC" --validator --force-authoring --name Bob3 \
  --base-path "$BOB_BP" --port 30336 --rpc-port 9947 --prometheus-port 9618 \
  --reserved-nodes "/ip4/127.0.0.1/tcp/30333/p2p/$ALICE_PEER" \
  --reserved-only \
  > "$LOG/Bob-reserved.log" 2>&1 &
BOB3_PID=$!

sleep 12

if kill -0 $BOB3_PID 2>/dev/null; then
  echo "TEST 3 RESULT: Bob ALIVE with --reserved-nodes"
  curl -s --max-time 2 http://127.0.0.1:9947 -H 'Content-Type: application/json' \
    -d '{"jsonrpc":"2.0","id":1,"method":"system_health","params":[]}' \
    | python3 -c "import sys,json; r=json.load(sys.stdin)['result']; print('  peers:', r['peers'])" 2>/dev/null
  kill -9 $BOB3_PID 2>/dev/null
else
  echo "TEST 3 RESULT: Bob CRASHED with --reserved-nodes"
fi

# ─── Diagnostics ───────────────────────────────────────────────
echo ""
echo "═══════════ LOG TAILS ═══════════"
echo ""
echo "─── Bob-noboot (last 8 lines) ───"
tail -8 "$LOG/Bob-noboot.log" 2>/dev/null || echo "(no log)"

echo ""
echo "─── Bob-boot (last 8 lines) ───"
tail -8 "$LOG/Bob-boot.log" 2>/dev/null || echo "(no log)"

echo ""
echo "─── Bob-reserved (last 8 lines) ───"
tail -8 "$LOG/Bob-reserved.log" 2>/dev/null || echo "(no log)"

echo ""
echo "═══════════ SUMMARY ═══════════"
echo "Alice alive:         $(kill -0 $ALICE_PID 2>/dev/null && echo YES || echo NO)"
echo "Bob-noboot alive:    $(kill -0 $BOB_PID 2>/dev/null && echo YES || echo NO)"
echo "Bob-boot alive:      $(kill -0 $BOB2_PID 2>/dev/null && echo YES || echo NO)"
echo "Bob-reserved alive:  $(kill -0 $BOB3_PID 2>/dev/null && echo YES || echo NO)"

echo ""
echo "Kill everything: pkill -9 -f solochain-template-node"
