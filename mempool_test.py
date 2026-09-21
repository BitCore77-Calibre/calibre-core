import json, urllib.request, hashlib, time, struct

RPC = "http://127.0.0.1:9944"

def compact(n):
    if n < 1 << 6:  return bytes([n << 2])
    if n < 1 << 14: return ((n << 2) | 0b01).to_bytes(2, "little")
def u32_le(n): return n.to_bytes(4, "little")
def u128_le(n): return n.to_bytes(16, "little")

WIT = compact(75) + bytes(75)

# Reconstruct the EXACT first CLI spend using SCALE encoding
gen_inputs  = compact(1) + bytes.fromhex("09" * 32) + u32_le(0)
gen_out1    = u128_le(600_000_000) + b"\x00" + bytes([0x02]) * 32
gen_out2    = u128_le(400_000_000) + b"\x00" + bytes([0x03]) * 32
gen_outputs = compact(2) + gen_out1 + gen_out2

# SCALE encode the TUPLE (inputs, outputs) exactly like Substrate does
tx_payload = gen_inputs + gen_outputs
TX_HASH_0 = hashlib.blake2b(tx_payload, digest_size=32).hexdigest()
print("Recomputed Genesis Tx Hash:", TX_HASH_0)

def build_new_spend(outs):
    inputs = compact(1) + bytes.fromhex(TX_HASH_0) + u32_le(1)
    outputs = compact(len(outs))
    for v, kb in outs:
        outputs += u128_le(v) + b"\x00" + bytes([kb]) * 32
    return inputs, outputs

def ext(ins, outs):
    call = bytes([8, 0]) + ins + outs + b"\x00" + WIT
    payload = b"\x04" + call
    return "0x" + (compact(len(payload)) + payload).hex()

def submit(h):
    req = urllib.request.Request(RPC, data=json.dumps({"jsonrpc": "2.0", "id": 1, "method": "author_submitExtrinsic", "params": [h]}).encode(), headers={"Content-Type": "application/json"})
    try:
        return json.load(urllib.request.urlopen(req))
    except Exception as e:
        return {"error": str(e)}

iX, oX = build_new_spend([(200_000_000, 30), (200_000_000, 31)])
iY, oY = build_new_spend([(400_000_000, 32)])

print("\n--- THE DUEL ---")
print("TxX (200M+200M):", submit(ext(iX, oX)))
print("TxY (same input, 400M):", submit(ext(iY, oY)))
time.sleep(9)
print("TxX replay after block:", submit(ext(iX, oX)))
