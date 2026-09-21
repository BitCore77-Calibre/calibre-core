#!/usr/bin/env python3
"""
Calibre Testnet Faucet
Receives a Dilithium public key, mints 1B testnet CAL locked to it.

Endpoints:
  GET  /                     — landing page
  POST /faucet {"pubkey":...} — request tokens
  GET  /health               — liveness probe
"""
import json, hashlib, os, time, urllib.request, http.server, socketserver, subprocess, re

RPC = "http://127.0.0.1:9944"
AMOUNT = 1_000_000_000
RATE_LIMIT_SECONDS = 60
PQ_SIGNER = "./target/release/pq-signer"  # only used to verify public keys are well-formed

# Simple in-memory rate limit: IP -> last request unix time
_last_request = {}

def compact(n):
    if n < 1 << 6:  return bytes([n<<2])
    if n < 1 << 14: return ((n<<2)|1).to_bytes(2,"little")
    raise ValueError("int too large for compact")

def u128_le(n): return n.to_bytes(16, "little")

def blake2_256(b): return hashlib.blake2b(b, digest_size=32).digest()

def rpc(method, params):
    req = urllib.request.Request(RPC,
        data=json.dumps({"jsonrpc":"2.0","id":1,"method":method,"params":params}).encode(),
        headers={"Content-Type":"application/json"})
    return json.load(urllib.request.urlopen(req))

def derive_lock(pubkey_hex):
    """blake2_256(bytes.fromhex(pubkey)) → the UTXO lock hash."""
    pk = bytes.fromhex(pubkey_hex.replace("0x", ""))
    return blake2_256(pk)

def mint_to_lock(lock_hash):
    """
    Build sudo.sudo(qutxo.sudoMint(AMOUNT, lock_hash)) as an unsigned call.
    NOTE: sudo requires Alice's signature. This faucet uses a local subxt-style
    signer for Alice. For a quick MVP, we shell out to a small nodejs helper —
    see tools/faucet/sign_mint.js. If that file is missing, we return an error
    telling the operator to install it.
    """
    # Build the INNER call only: qutxo.sudoMint(value, lock)
    # The JS helper will wrap it in sudo.sudo() itself.
    inner = bytes([8, 1]) + u128_le(AMOUNT) + lock_hash

    helper = "tools/faucet/sign_mint.js"
    if not os.path.exists(helper):
        raise RuntimeError(f"missing signer helper {helper}")

    p = subprocess.run(["node", helper, inner.hex()],
                       capture_output=True, text=True, timeout=120)
    if p.returncode != 0:
        raise RuntimeError("signer error: " + p.stderr.strip())

    # The JS helper already signed + submitted + waited for inclusion.
    # Its stdout is the block hash where the mint landed.
    return p.stdout.strip() or "(included)"

class Handler(http.server.BaseHTTPRequestHandler):
    def log_message(self, fmt, *args):
        print(f"[{time.strftime('%H:%M:%S')}] {self.address_string()} {fmt % args}")

    def _send(self, code, body, ctype="application/json"):
        self.send_response(code)
        self.send_header("Content-Type", ctype)
        self.send_header("Access-Control-Allow-Origin", "*")
        self.end_headers()
        self.wfile.write(body if isinstance(body, bytes) else body.encode())

    def do_GET(self):
        if self.path == "/":
            self._send(200, json.dumps({
                "name": "Calibre Testnet Faucet",
                "usage": "POST /faucet {\"pubkey\": \"<hex-dilithium-pubkey>\"}",
                "amount_per_request": AMOUNT,
                "chain": "calibre_testnet_1"
            }, indent=2))
        elif self.path == "/health":
            self._send(200, json.dumps({"ok": True}))
        else:
            self._send(404, json.dumps({"error": "not found"}))

    def do_POST(self):
        if self.path != "/faucet":
            self._send(404, json.dumps({"error": "not found"})); return

        ip = self.client_address[0]
        now = time.time()
        if ip in _last_request and now - _last_request[ip] < RATE_LIMIT_SECONDS:
            wait = int(RATE_LIMIT_SECONDS - (now - _last_request[ip]))
            self._send(429, json.dumps({"error": f"rate limited, retry in {wait}s"})); return

        length = int(self.headers.get("Content-Length", 0))
        try:
            body = json.loads(self.rfile.read(length) or b"{}")
        except Exception as e:
            self._send(400, json.dumps({"error": f"bad json: {e}"})); return

        pubkey = body.get("pubkey", "").strip()
        if not re.fullmatch(r"(0x)?[0-9a-fA-F]{2624}", pubkey):
            self._send(400, json.dumps({"error": "pubkey must be 2624 hex chars (ML-DSA-44)"})); return

        try:
            lock = derive_lock(pubkey)
            tx = mint_to_lock(lock)
            _last_request[ip] = now
            self._send(200, json.dumps({
                "ok": True,
                "lock_hash": "0x" + lock.hex(),
                "amount": AMOUNT,
                "tx_hash": tx,
                "note": "mint pending — visible in the next block (~6s)"
            }, indent=2))
        except Exception as e:
            self._send(500, json.dumps({"error": str(e)}))


def warmup():
    """Pre-warm the node signer so the first real request isn't cold-start-slow."""
    try:
        print("Warming up signer (this may take 10-20s)...")
        p = subprocess.run(["node", "tools/faucet/sign_mint.js", "warmup"],
                           capture_output=True, text=True, timeout=120)
        # We expect nonzero exit (bad call) but successful connection attempt
        print("Signer warm-up complete")
    except subprocess.TimeoutExpired:
        print("Warm-up timed out — first request may be slow")
    except Exception as e:
        print(f"Warm-up error (non-fatal): {e}")


if __name__ == "__main__":
    PORT = int(os.environ.get("FAUCET_PORT", 8080))
    print(f"Calibre Faucet listening on :{PORT} — chain RPC {RPC}")
    warmup()
    with socketserver.TCPServer(("0.0.0.0", PORT), Handler) as httpd:
        httpd.allow_reuse_address = True
        httpd.serve_forever()
