#!/usr/bin/env python3
"""Prepare and run a disposable Phase 9.4 election on one computer.

This deliberately seeds a raw genesis fixture with synthetic stake. It does
not exercise QUTXO bonding or signed candidate registration. The user starts
the long-running network explicitly; importing this module has no side effects.
"""

from __future__ import annotations

import argparse
import ctypes
import ctypes.util
import hashlib
import json
import os
import re
import socket
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path
from urllib.request import Request, urlopen

ROOT = Path(__file__).resolve().parents[1]
NODE = ROOT / "target/release/solochain-template-node"
EXPLORER = ROOT / "observatory/server.py"
IDENTITIES = ("alice", "bob", "charlie", "dave", "eve", "ferdie", "one", "two")
UNITS_PER_CAL = 10**18
SESSION_SLOTS = 600


def twox64(data: bytes) -> bytes:
    name = ctypes.util.find_library("xxhash")
    if not name:
        raise RuntimeError("libxxhash is required to construct Substrate storage keys")
    library = ctypes.CDLL(name)
    function = library.XXH64
    function.argtypes = (ctypes.c_void_p, ctypes.c_size_t, ctypes.c_ulonglong)
    function.restype = ctypes.c_ulonglong
    buffer = ctypes.create_string_buffer(data)
    return function(buffer, len(data), 0).to_bytes(8, "little")


def twox128(data: bytes) -> bytes:
    name = ctypes.util.find_library("xxhash")
    if not name:
        raise RuntimeError("libxxhash is required to construct Substrate storage keys")
    library = ctypes.CDLL(name)
    function = library.XXH64
    function.argtypes = (ctypes.c_void_p, ctypes.c_size_t, ctypes.c_ulonglong)
    function.restype = ctypes.c_ulonglong
    buffer = ctypes.create_string_buffer(data)
    return b"".join(function(buffer, len(data), seed).to_bytes(8, "little") for seed in (0, 1))


def key(pallet: str, item: str) -> bytes:
    return twox128(pallet.encode()) + twox128(item.encode())


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        while chunk := source.read(1024 * 1024):
            digest.update(chunk)
    return digest.hexdigest()


def compact_vector(value: bytes, item_size: int) -> list[bytes]:
    if not value:
        raise ValueError("Missing SCALE vector")
    mode = value[0] & 3
    if mode == 0:
        count, offset = value[0] >> 2, 1
    elif mode == 1:
        count, offset = int.from_bytes(value[:2], "little") >> 2, 2
    elif mode == 2:
        count, offset = int.from_bytes(value[:4], "little") >> 2, 4
    else:
        raise ValueError("Unexpected large SCALE vector")
    if len(value) != offset + count * item_size:
        raise ValueError("Unexpected SCALE vector size")
    return [value[offset + i * item_size:offset + (i + 1) * item_size]
            for i in range(count)]


def inspect_test_key(scheme: str) -> tuple[bytes, bytes]:
    # The well-known //Two development URI is public test material. Keep the
    # command's JSON (which includes a derived test seed) out of logs.
    result = subprocess.run(
        [str(NODE), "key", "inspect", "//Two", "--scheme", scheme,
         "--output-type", "json"], capture_output=True, text=True, check=True)
    payload = json.loads(result.stdout)
    return (bytes.fromhex(payload["accountId"].removeprefix("0x")),
            bytes.fromhex(payload["publicKey"].removeprefix("0x")))


def prepare(run_dir: Path) -> dict:
    if run_dir.exists():
        raise FileExistsError(f"Run directory already exists: {run_dir}")
    if not NODE.is_file() or not EXPLORER.is_file():
        raise FileNotFoundError("Build the release node and keep observatory/server.py in the repository")
    raw = subprocess.run([str(NODE), "build-spec", "--chain", "staging", "--raw"],
                         capture_output=True, text=True, check=True)
    spec = json.loads(raw.stdout)
    top = spec["genesis"]["raw"]["top"]
    top = {k.lower(): v.lower() for k, v in top.items()}
    active = compact_vector(bytes.fromhex(top[("0x" + key("Session", "Validators").hex())][2:]), 32)
    if len(active) != 7 or len(set(active)) != 7:
        raise ValueError("Staging genesis must have seven distinct active validators")
    standby, standby_babe = inspect_test_key("sr25519")
    _, standby_grandpa = inspect_test_key("ed25519")
    if len(standby) != 32 or len(standby_babe) != 32 or len(standby_grandpa) != 32:
        raise ValueError("Unexpected //Two public key size")
    if standby in active:
        raise ValueError("//Two is already an active validator")
    next_keys_prefix = key("Session", "NextKeys")
    next_keys_key = "0x" + (next_keys_prefix + twox64(standby) + standby).hex()
    if next_keys_key in top:
        raise ValueError("//Two already has NextKeys; expected a clean staging genesis")
    top[next_keys_key] = "0x" + (standby_babe + standby_grandpa).hex()

    # Six incumbents remain above //Two, and //One falls below it. These are
    # synthetic ledger values in a throwaway chain, not QUTXO-backed deposits.
    weights_cal = [2000] * 6 + [1000, 1500]
    for account, amount in zip([*active, standby], weights_cal):
        top["0x" + (key("Stake", "Stake") + hashlib.blake2b(
            account, digest_size=16).digest() + account).hex()] = "0x" + (
                amount * UNITS_PER_CAL).to_bytes(16, "little").hex()
        top["0x" + (key("Stake", "Candidates") + hashlib.blake2b(
            account, digest_size=16).digest() + account).hex()] = "0x"
    top["0x" + key("Stake", "TotalStaked").hex()] = "0x" + (
        sum(weights_cal) * UNITS_PER_CAL).to_bytes(16, "little").hex()
    spec["genesis"]["raw"]["top"] = top
    spec["name"] = "CALIBRE Phase 9.4 local synthetic-stake lab"
    spec["id"] = "calibre_phase94_local_lab"
    spec["bootNodes"] = []

    spec_text = json.dumps(spec, indent=2) + "\n"
    source_commit = subprocess.run(["git", "rev-parse", "HEAD"], cwd=ROOT,
                                   capture_output=True, text=True, check=True).stdout.strip()
    manifest = {
        "format": 1, "scope": "single-computer synthetic-stake Phase 9.4 lab",
        "prepared_at": datetime.now(timezone.utc).isoformat(),
        "source_commit": source_commit,
        "node_binary_sha256": sha256_file(NODE),
        "chain_spec_sha256": hashlib.sha256(spec_text.encode()).hexdigest(),
        "epoch_slots": SESSION_SLOTS, "sessions_per_election": 6,
        "expected_spec_version": 103,
        "identity_order": list(IDENTITIES),
        "initial_validators": ["0x" + account.hex() for account in active],
        "replacement_validator": "0x" + standby.hex(),
        "removed_validator": "0x" + active[-1].hex(),
        "initial_babe": ["0x" + x[:32].hex() for x in compact_vector(
            bytes.fromhex(top[("0x" + key("Babe", "Authorities").hex())][2:]), 40)],
        "initial_grandpa": ["0x" + x[:32].hex() for x in compact_vector(
            bytes.fromhex(top[("0x" + key("Grandpa", "Authorities").hex())][2:]), 40)],
        "replacement_babe": "0x" + standby_babe.hex(),
        "replacement_grandpa": "0x" + standby_grandpa.hex(),
        "limits": ["synthetic stake is not backed by QUTXO bonding",
                   "session keys and candidates are prepared in genesis, not signed calls",
                   "one computer does not verify multi-computer networking"],
    }
    run_dir.mkdir(parents=True, mode=0o700)
    (run_dir / "chain-spec.json").write_text(spec_text)
    (run_dir / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    return manifest


def rpc(port: int, method: str, params=None):
    payload = json.dumps({"jsonrpc": "2.0", "id": 1, "method": method,
                          "params": params or []}).encode()
    request = Request(f"http://127.0.0.1:{port}", data=payload,
                      headers={"Content-Type": "application/json"})
    with urlopen(request, timeout=2) as response:
        result = json.load(response)
    if result.get("error"):
        raise RuntimeError(f"{method}: {result['error']}")
    return result["result"]


def port_available(port: int) -> bool:
    with socket.socket() as sock:
        try:
            sock.bind(("127.0.0.1", port))
            return True
        except OSError:
            return False


def run(run_dir: Path, rpc_start: int, p2p_start: int, explorer_port: int):
    ports = ([rpc_start + i for i in range(8)] +
             [p2p_start + i for i in range(8)] + [explorer_port])
    if len(ports) != len(set(ports)) or any(not port_available(p) for p in ports):
        raise RuntimeError("Requested RPC, P2P, or explorer port is already in use")
    manifest = json.loads((run_dir / "manifest.json").read_text())
    if manifest["identity_order"] != list(IDENTITIES):
        raise ValueError("Unexpected lab manifest identity order")
    processes = []
    logs = []
    try:
        bootnode = None
        for index, identity in enumerate(IDENTITIES):
            log_path = run_dir / f"{identity}.log"
            log = log_path.open("w")
            logs.append(log)
            args = [str(NODE), "--chain", str(run_dir / "chain-spec.json"), "--tmp",
                    f"--{identity}", "--validator", "--force-authoring",
                    "--name", f"Phase94-{identity}", "--port", str(p2p_start + index),
                    "--rpc-port", str(rpc_start + index), "--no-prometheus",
                    "--no-telemetry", "--unsafe-force-node-key-generation"]
            if bootnode:
                args.extend(("--bootnodes", bootnode))
            process = subprocess.Popen(args, stdout=log, stderr=subprocess.STDOUT)
            processes.append(process)
            if index == 0:
                deadline = time.monotonic() + 30
                while time.monotonic() < deadline:
                    if process.poll() is not None:
                        raise RuntimeError("First validator exited; inspect alice.log")
                    match = re.search(r"Local node identity is:\s*(\S+)",
                                      log_path.read_text(errors="replace"))
                    if match:
                        bootnode = (f"/ip4/127.0.0.1/tcp/{p2p_start}/p2p/"
                                    f"{match.group(1)}")
                        break
                    time.sleep(0.5)
                if not bootnode:
                    raise RuntimeError("Could not read first validator's peer ID")

        deadline = time.monotonic() + 60
        genesis_hash = None
        while time.monotonic() < deadline:
            if any(process.poll() is not None for process in processes):
                raise RuntimeError("A validator exited during startup; inspect its log")
            try:
                genesis_hash = rpc(rpc_start, "chain_getBlockHash", [0])
                if genesis_hash:
                    break
            except (OSError, TimeoutError, ValueError):
                pass
            time.sleep(1)
        if not genesis_hash:
            raise RuntimeError("First validator RPC did not start")
        version = rpc(rpc_start, "state_getRuntimeVersion")
        config = bytes.fromhex(rpc(rpc_start, "state_call", ["BabeApi_configuration", "0x"])[2:])
        epoch_slots = int.from_bytes(config[8:16], "little") if len(config) >= 16 else None
        if version.get("specVersion") != manifest["expected_spec_version"] or epoch_slots != SESSION_SLOTS:
            raise RuntimeError("Node runtime version or BABE epoch length does not match this lab")
        env = os.environ.copy()
        env["CALIBRE_RPC_URLS"] = ",".join(
            f"http://127.0.0.1:{rpc_start+i}" for i in range(8))
        env["CALIBRE_EXPECTED_GENESIS_HASH"] = genesis_hash
        env["CALIBRE_PHASE94_MANIFEST"] = str(run_dir / "manifest.json")
        explorer_log = (run_dir / "explorer.log").open("w")
        logs.append(explorer_log)
        explorer = subprocess.Popen(
            [sys.executable, str(EXPLORER), "--port", str(explorer_port),
             "--data", str(run_dir / "observations.sqlite3")],
            env=env, stdout=explorer_log, stderr=subprocess.STDOUT)
        processes.append(explorer)
        print(f"Run directory: {run_dir}", flush=True)
        print(f"Explorer: http://127.0.0.1:{explorer_port}/", flush=True)
        print("Seven initial validators and one standby are starting.", flush=True)
        print("Keep this terminal open for the normal six-session election (~6 hours).", flush=True)
        print("Press Ctrl+C to stop; logs and observation history remain in the run directory.", flush=True)
        while True:
            for identity, process in zip([*IDENTITIES, "explorer"], processes):
                if process.poll() is not None:
                    raise RuntimeError(f"{identity} exited; inspect {identity}.log")
            time.sleep(5)
    finally:
        for process in processes:
            if process.poll() is None:
                process.terminate()
        for process in processes:
            try:
                process.wait(timeout=8)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait()
        for log in logs:
            log.close()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--run-dir", type=Path, help="new directory for this run")
    parser.add_argument("--rpc-start", type=int, default=19944)
    parser.add_argument("--p2p-start", type=int, default=20333)
    parser.add_argument("--explorer-port", type=int, default=18765)
    args = parser.parse_args()
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    run_dir = (args.run_dir or ROOT / ".phase94-runs" / stamp).expanduser().resolve()
    try:
        prepare(run_dir)
        run(run_dir, args.rpc_start, args.p2p_start, args.explorer_port)
    except KeyboardInterrupt:
        print("Stopped. Logs and observations were retained.", flush=True)
    except (OSError, ValueError, KeyError, RuntimeError, subprocess.CalledProcessError) as exc:
        print(f"Phase 9.4 lab setup/run failed: {exc}", file=sys.stderr)
        raise SystemExit(1) from exc


if __name__ == "__main__":
    main()
