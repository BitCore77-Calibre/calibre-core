import json, urllib.request, hashlib

RPC = "http://127.0.0.1:9944"

def compact(n):
    if n < 1 << 6:  return bytes([n << 2])
    if n < 1 << 14: return ((n << 2) | 0b01).to_bytes(2, "little")
    raise ValueError("too big")

def u32(n): return n.to_bytes(4, "little")
def u64(n): return n.to_bytes(8, "little")
def u128(n): return n.to_bytes(16, "little")

# --- THE FIRST SPEND: burn 1B genesis UTXO -> 600M + 400M ---
# 1. Input: The Genesis UTXO (Hash of 0909...09, index 0)
inputs  = compact(1) + bytes.fromhex("09" * 32) + u32(0)

# 2. Outputs: 600M to key 02...02, 400M to key 03...03
out1    = u128(600_000_000) + b"\x00" + bytes([0x02]) * 32   # \x00 = SingleSig variant
out2    = u128(400_000_000) + b"\x00" + bytes([0x03]) * 32
outputs = compact(2) + out1 + out2

# 3. Empty PQ Signature (we rely on the witness)
pq_sig  = b"\x00"                                            

# 4. The A.E.G.I.S. Witness (SCALE encoded with empty vectors and zero hashes)
witness_inner = (b"\x00" + b"\x00" + b"\x00"                 # 3 empty vectors
                 + bytes(32) + bytes(32)                     # attestation hashes
                 + u64(0))                                   # time_drift
witness = compact(len(witness_inner)) + witness_inner

# Pallet 8 (qutxo), Call 0 (execute_utxo_tx)
call = bytes([0x08, 0x00]) + inputs + outputs + pq_sig + witness  

def rpc(method, params):
    req = urllib.request.Request(
        RPC, data=json.dumps({"jsonrpc": "2.0", "id": 1,
                              "method": method, "params": params}).encode(),
        headers={"Content-Type": "application/json"})
    return json.load(urllib.request.urlopen(req))

# Substrate expects the extrinsic with a compact length prefix
for vb in (b"\x04", b"\x05"):  # Try unsigned v4, then unsigned v5
    payload = vb + call
    full_ext = compact(len(payload)) + payload
    hex_ext = "0x" + full_ext.hex()
    
    res = rpc("author_submitExtrinsic", [hex_ext])
    print(f"version {vb.hex()} ->", res)
    if "result" in res:
        print("\n✅ ACCEPTED INTO MEMPOOL. Extrinsic hash:", res["result"])
        break
else:
    raise SystemExit("❌ rejected — paste me the errors above.")

# Calculate the hashes of the new child UTXOs so we can track them
tx_hash = hashlib.blake2b(inputs + outputs, digest_size=32).digest()
for i in range(2):
    child = hashlib.blake2b(tx_hash + u32(i), digest_size=32).digest()
    print(f"child UTXO [{i}] id: 0x{child.hex()}")
