#!/usr/bin/env python3
"""Read-only, local CALIBRE observatory for Polkadot SDK JSON-RPC nodes."""

from __future__ import annotations

import argparse
import concurrent.futures
import ctypes
import ctypes.util
import functools
import hashlib
import json
import os
import sqlite3
import threading
import time
import urllib.error
import urllib.request
from datetime import datetime, timezone
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qs, urlparse

ROOT = Path(__file__).resolve().parent
SAFE_METHODS = {
    "system_chain", "system_health", "system_properties", "system_version",
    "chain_getBlockHash", "chain_getHeader", "chain_getFinalizedHead",
    "chain_getBlock", "author_pendingExtrinsics", "state_getStorage",
    "state_getRuntimeVersion", "state_call",
}


@functools.lru_cache(maxsize=16)
def storage_key(pallet: str, item: str) -> str:
    library_name = ctypes.util.find_library("xxhash")
    if not library_name:
        raise RuntimeError("libxxhash is required for Phase 9.4 state decoding")
    library = ctypes.CDLL(library_name)
    function = library.XXH64
    function.argtypes = (ctypes.c_void_p, ctypes.c_size_t, ctypes.c_ulonglong)
    function.restype = ctypes.c_ulonglong
    def twox128(value):
        buffer = ctypes.create_string_buffer(value)
        return b"".join(function(buffer, len(value), seed).to_bytes(8, "little")
                        for seed in (0, 1))
    return "0x" + (twox128(pallet.encode()) + twox128(item.encode())).hex()


def decode_vector(raw: str, item_size: int) -> list[str]:
    if not isinstance(raw, str) or not raw.startswith("0x"):
        raise ValueError("Missing SCALE vector")
    data = bytes.fromhex(raw[2:])
    if not data:
        raise ValueError("Empty SCALE vector")
    mode = data[0] & 3
    if mode == 0:
        count, offset = data[0] >> 2, 1
    elif mode == 1:
        count, offset = int.from_bytes(data[:2], "little") >> 2, 2
    elif mode == 2:
        count, offset = int.from_bytes(data[:4], "little") >> 2, 4
    else:
        raise ValueError("Unexpected large SCALE vector")
    if len(data) != offset + count * item_size:
        raise ValueError("Unexpected SCALE vector size")
    return ["0x" + data[offset + i * item_size:
                           offset + i * item_size + 32].hex() for i in range(count)]


def phase94_state(url: str, at_hash: str) -> dict:
    def value(pallet, item):
        return rpc(url, "state_getStorage", [storage_key(pallet, item), at_hash])
    raw_index = value("Session", "CurrentIndex")
    index = int.from_bytes(bytes.fromhex(raw_index[2:]), "little") if raw_index else 0
    config = bytes.fromhex(rpc(url, "state_call", ["BabeApi_configuration", "0x", at_hash])[2:])
    if len(config) < 16:
        raise ValueError("BABE configuration is truncated")
    version = rpc(url, "state_getRuntimeVersion", [at_hash])
    return {
        "at_finalized_hash": at_hash,
        "session": index,
        "epoch_slots": int.from_bytes(config[8:16], "little"),
        "spec_version": version["specVersion"],
        "validators": decode_vector(value("Session", "Validators"), 32),
        "babe": decode_vector(value("Babe", "Authorities"), 40),
        "grandpa": decode_vector(value("Grandpa", "Authorities"), 40),
    }


def now() -> str:
    return datetime.now(timezone.utc).isoformat(timespec="milliseconds")


def hex_number(value):
    try:
        return int(value, 16) if isinstance(value, str) else None
    except ValueError:
        return None


class RpcError(Exception):
    pass


def rpc(url: str, method: str, params=None, timeout=1.7):
    if method not in SAFE_METHODS:
        raise RpcError("RPC method not allowed")
    payload = json.dumps({"jsonrpc": "2.0", "id": 1, "method": method,
                          "params": [] if params is None else params}).encode()
    req = urllib.request.Request(url, data=payload,
                                 headers={"Content-Type": "application/json"})
    try:
        with urllib.request.urlopen(req, timeout=timeout) as response:
            result = json.load(response)
    except (TimeoutError, OSError, ValueError, urllib.error.URLError) as exc:
        raise RpcError(str(exc)) from exc
    if not isinstance(result, dict):
        raise RpcError("Invalid JSON-RPC response")
    if result.get("error"):
        error = result["error"]
        raise RpcError(str(error.get("message", error)) if isinstance(error, dict) else str(error))
    return result.get("result")


class Evidence:
    def __init__(self, path: Path):
        path.parent.mkdir(parents=True, exist_ok=True)
        self.connection = sqlite3.connect(path, check_same_thread=False)
        self.connection.execute("PRAGMA journal_mode=WAL")
        self.connection.execute("""CREATE TABLE IF NOT EXISTS observations (
            id INTEGER PRIMARY KEY AUTOINCREMENT, observed_at TEXT NOT NULL,
            category TEXT NOT NULL, node TEXT, reference TEXT,
            description TEXT NOT NULL, evidence TEXT NOT NULL,
            previous_digest TEXT NOT NULL, digest TEXT NOT NULL)""")
        self.connection.commit()
        self.lock = threading.Lock()

    def record(self, category, node, reference, description, evidence):
        entry = {"observed_at": now(), "category": category, "node": node,
                 "reference": reference, "description": description, "evidence": evidence}
        with self.lock:
            previous = self.connection.execute(
                "SELECT digest FROM observations ORDER BY id DESC LIMIT 1"
            ).fetchone()
            previous_digest = previous[0] if previous else "0" * 64
            digest = hashlib.sha256((previous_digest + json.dumps(
                entry, sort_keys=True, separators=(",", ":"))).encode()).hexdigest()
            cursor = self.connection.execute(
                """INSERT INTO observations
                   (observed_at,category,node,reference,description,evidence,previous_digest,digest)
                   VALUES (?,?,?,?,?,?,?,?)""",
                (entry["observed_at"], category, node, reference, description,
                 json.dumps(evidence, sort_keys=True), previous_digest, digest))
            self.connection.commit()
            return cursor.lastrowid

    def recent(self, limit=100):
        with self.lock:
            rows = self.connection.execute(
                """SELECT id, observed_at, category, node, reference, description, evidence,
                          previous_digest, digest FROM observations ORDER BY id DESC LIMIT ?""",
                (max(1, min(500, int(limit))),)).fetchall()
        return [dict(zip(("id", "observed_at", "category", "node", "reference",
                          "description", "evidence", "previous_digest", "digest"),
                         (*row[:6], json.loads(row[6]), *row[7:]))) for row in rows]


def configured_nodes():
    urls = os.environ.get("CALIBRE_RPC_URLS", "http://127.0.0.1:9944")
    nodes = []
    for index, raw in enumerate(urls.split(","), 1):
        url = raw.strip().rstrip("/")
        if not url:
            continue
        parsed = urlparse(url)
        if (parsed.scheme not in ("http", "https") or parsed.hostname not in
                ("127.0.0.1", "localhost", "::1") or parsed.username or parsed.password or
                parsed.path not in ("", "/") or parsed.query or parsed.fragment):
            raise ValueError("RPC URLs must be local HTTP(S) root endpoints")
        nodes.append({"id": f"node-{index:02d}", "url": url})
    if not (1 <= len(nodes) <= 20):
        raise ValueError("Configure between 1 and 20 local node RPC URLs")
    return nodes


class Collector:
    def __init__(self, nodes, evidence, interval, phase94_manifest=None):
        self.nodes = nodes
        self.evidence = evidence
        self.interval = interval
        self.lock = threading.Lock()
        self.running = threading.Event()
        self.snapshot = {"collected_at": None, "nodes": [], "network": {},
                         "capabilities": {"transactions": "not instrumented",
                                          "signing": "not connected",
                                          "experiments": "not connected",
                                          "runtime_events": "not decoded",
                                          "consensus_engine": "not verified"}}
        self.previous = {}
        self.phase94_manifest = phase94_manifest
        self.phase94_first_post_finalized = None
        self.phase94_previous_status = None

    def collect_node(self, definition):
        url, node_id = definition["url"], definition["id"]
        result = {"id": node_id, "url": url, "observed_at": now(), "online": False,
                  "errors": [], "pool": {"state": "unavailable"}}
        try:
            result["chain"] = rpc(url, "system_chain")
            result["genesis_hash"] = rpc(url, "chain_getBlockHash", [0])
            result["best_header"] = rpc(url, "chain_getHeader")
            result["finalized_hash"] = rpc(url, "chain_getFinalizedHead")
            result["finalized_header"] = rpc(url, "chain_getHeader", [result["finalized_hash"]])
            result["health"] = rpc(url, "system_health")
            result["properties"] = rpc(url, "system_properties")
            result["best_height"] = hex_number((result["best_header"] or {}).get("number"))
            result["finalized_height"] = hex_number((result["finalized_header"] or {}).get("number"))
            if result["best_height"] is not None:
                result["best_hash"] = rpc(url, "chain_getBlockHash", [result["best_height"]])
            result["online"] = True
        except RpcError as exc:
            result["errors"].append({"method": "core", "message": str(exc)})
            return result
        try:
            result["version"] = rpc(url, "system_version")
        except RpcError as exc:
            result["errors"].append({"method": "system_version", "message": str(exc)})
        try:
            pending = rpc(url, "author_pendingExtrinsics")
            if not isinstance(pending, list):
                raise RpcError("Expected a list of pending extrinsics")
            # Pending extrinsic bytes are local node observations, never canonical chain evidence.
            result["pool"] = {"state": "snapshot", "count": len(pending),
                              "extrinsics": pending[:80], "truncated": len(pending) > 80,
                              "ready_future_breakdown": "not exposed by this RPC"}
        except RpcError as exc:
            result["pool"] = {"state": "unavailable", "reason": str(exc)}
        if self.phase94_manifest:
            try:
                result["phase94"] = phase94_state(url, result["finalized_hash"])
            except (RpcError, ValueError, KeyError, TypeError, RuntimeError) as exc:
                result["phase94"] = {"error": str(exc)}
        return result

    def phase94_summary(self, nodes, network):
        manifest = self.phase94_manifest
        if not manifest:
            return None
        summary = {"status": "AWAITING_NODES", "scope": manifest["scope"],
                   "expected_epoch_slots": manifest["epoch_slots"],
                   "replacement_validator": manifest["replacement_validator"],
                   "removed_validator": manifest["removed_validator"],
                   "limits": manifest["limits"]}
        if len(nodes) != 8 or any(not n["online"] or "error" in n.get("phase94", {})
                                   for n in nodes):
            summary["reason"] = "Waiting for all eight nodes and finalized state reads"
            return summary
        states = [n["phase94"] for n in nodes]
        summaries = {s["session"] for s in states}
        summary["sessions"] = sorted(summaries)
        summary["authority_count"] = len(states[0]["validators"])
        summary["finalized_heights"] = [n["finalized_height"] for n in nodes]
        if not network["genesis_match"] or any(
            s["epoch_slots"] != manifest["epoch_slots"] or
            s["spec_version"] != manifest["expected_spec_version"] for s in states
        ):
            summary["status"] = "FAIL"
            summary["reason"] = "Genesis, epoch duration, or runtime version mismatch"
            return summary
        if min(summaries) < 6:
            summary["status"] = "AWAITING_ELECTION"
            return summary
        if len(summaries) != 1:
            summary["status"] = "OBSERVING"
            summary["reason"] = "Nodes have not reached the same session"
            return summary
        expected = (set(manifest["initial_validators"]) -
                    {manifest["removed_validator"]}) | {manifest["replacement_validator"]}
        for field, initial, replacement in (
            ("validators", manifest["initial_validators"], manifest["replacement_validator"]),
            ("babe", manifest["initial_babe"], manifest["replacement_babe"]),
            ("grandpa", manifest["initial_grandpa"], manifest["replacement_grandpa"]),
        ):
            target = expected if field == "validators" else (
                set(initial) - {initial[-1]}) | {replacement}
            if (len(target) != 7 or any(len(s[field]) != 7 or set(s[field]) != target
                                        for s in states)):
                summary["status"] = "FAIL"
                summary["reason"] = f"{field} authorities differ from the expected seven"
                return summary
        if any(n["finalized_height"] is None for n in nodes):
            summary["status"] = "OBSERVING"
            summary["reason"] = "Waiting for finalized heights"
            return summary
        common_height = min(n["finalized_height"] for n in nodes)
        if self.phase94_first_post_finalized is None:
            self.phase94_first_post_finalized = common_height
        hashes = []
        try:
            for node in nodes:
                hashes.append(rpc(node["url"], "chain_getBlockHash", [common_height]))
        except RpcError:
            summary["status"] = "OBSERVING"
            summary["reason"] = "Waiting for common finalized hash RPC"
            return summary
        summary["common_finalized_height"] = common_height
        summary["common_finalized_hash"] = hashes[0]
        summary["common_hash_agreement"] = len(set(hashes)) == 1
        if not summary["common_hash_agreement"]:
            summary["status"] = "FAIL"
            summary["reason"] = "Nodes disagree on the common finalized block"
        elif common_height >= self.phase94_first_post_finalized + 3:
            summary["status"] = "PASS"
            summary["reason"] = "Seven elected authorities and continuing common finality observed"
        else:
            summary["status"] = "OBSERVING"
            summary["reason"] = "Waiting for three more finalized blocks after election"
        return summary

    def poll(self):
        with concurrent.futures.ThreadPoolExecutor(max_workers=min(10, len(self.nodes))) as workers:
            nodes = list(workers.map(self.collect_node, self.nodes))
        online = [node for node in nodes if node["online"]]
        genesis_hashes = sorted({n["genesis_hash"] for n in online if n.get("genesis_hash")})
        finalized = sorted({n["finalized_hash"] for n in online if n.get("finalized_hash")})
        expected = os.environ.get("CALIBRE_EXPECTED_GENESIS_HASH", "").strip()
        network = {"online": len(online), "configured": len(nodes),
                   "genesis_hashes": genesis_hashes, "genesis_match": len(genesis_hashes) <= 1,
                   "expected_genesis_hash": expected or None,
                   "finalized_hashes": finalized,
                   "best_height": max((n["best_height"] for n in online
                                       if n.get("best_height") is not None), default=None),
                   "finalized_height": max((n["finalized_height"] for n in online
                                            if n.get("finalized_height") is not None), default=None),
                   "status": "LIVE" if online else "OFFLINE"}
        if expected and genesis_hashes and any(h != expected for h in genesis_hashes):
            network["genesis_match"] = False
        state = {"collected_at": now(), "nodes": nodes, "network": network,
                 "capabilities": self.snapshot["capabilities"]}
        if self.phase94_manifest:
            state["phase94"] = self.phase94_summary(nodes, network)
        self.track_changes(state)
        with self.lock:
            self.snapshot = state

    def track_changes(self, state):
        phase94 = state.get("phase94")
        if phase94 and phase94["status"] != self.phase94_previous_status:
            if phase94["status"] in ("PASS", "FAIL"):
                self.evidence.record("phase94_" + phase94["status"].lower(), None,
                                     phase94.get("common_finalized_hash"),
                                     phase94.get("reason", phase94["status"]), phase94)
            self.phase94_previous_status = phase94["status"]
        for node in state["nodes"]:
            key = node["id"]
            before = self.previous.get(key, {})
            if node["online"] != before.get("online"):
                self.evidence.record("node_online" if node["online"] else "node_offline",
                                     key, node.get("genesis_hash"),
                                     "Node connected" if node["online"] else "Node disconnected",
                                     {"endpoint": node["url"], "errors": node["errors"]})
            if node["online"]:
                for field, category in (("best_height", "best_head"),
                                        ("finalized_hash", "finalized_head")):
                    if node.get(field) is not None and node.get(field) != before.get(field):
                        self.evidence.record(category, key, str(node.get(field)),
                                             f"{field.replace('_', ' ').title()} changed",
                                             {"previous": before.get(field), "current": node.get(field),
                                              "genesis_hash": node.get("genesis_hash")})
            self.previous[key] = {"online": node["online"], "best_height": node.get("best_height"),
                                  "finalized_hash": node.get("finalized_hash")}
        if state["network"]["online"] and not state["network"]["genesis_match"]:
            if not self.previous.get("_genesis_alarm"):
                self.evidence.record("invariant_failure", None, "genesis_hash",
                                     "Connected nodes disagree on genesis hash",
                                     {"genesis_hashes": state["network"]["genesis_hashes"],
                                      "expected": state["network"]["expected_genesis_hash"]})
            self.previous["_genesis_alarm"] = True
        else:
            self.previous["_genesis_alarm"] = False

    def run(self):
        self.running.set()
        while self.running.is_set():
            start = time.monotonic()
            try:
                self.poll()
            except Exception as exc:
                self.evidence.record("collector_error", None, None, "Collector failed",
                                     {"error": str(exc)})
            time.sleep(max(0.1, self.interval - (time.monotonic() - start)))

    def current(self):
        with self.lock:
            return self.snapshot.copy()


def make_handler(collector, evidence):
    class Handler(BaseHTTPRequestHandler):
        def json_response(self, status, payload):
            data = json.dumps(payload, separators=(",", ":")).encode()
            self.send_response(status)
            self.send_header("Content-Type", "application/json; charset=utf-8")
            self.send_header("Cache-Control", "no-store")
            self.send_header("X-Content-Type-Options", "nosniff")
            self.send_header("Content-Security-Policy", "default-src 'none'")
            self.send_header("Content-Length", str(len(data)))
            self.end_headers()
            self.wfile.write(data)

        def do_GET(self):
            path = urlparse(self.path)
            if path.path == "/api/snapshot":
                return self.json_response(200, collector.current())
            if path.path == "/api/events":
                try:
                    limit = int(parse_qs(path.query).get("limit", ["100"])[0])
                except ValueError:
                    return self.json_response(400, {"error": "Invalid limit"})
                return self.json_response(200, {"events": evidence.recent(limit)})
            if path.path.startswith("/api/block/"):
                block_hash = path.path.removeprefix("/api/block/")
                if len(block_hash) != 66 or not block_hash.startswith("0x") or any(
                    c not in "0123456789abcdefABCDEF" for c in block_hash[2:]
                ):
                    return self.json_response(400, {"error": "Expected a 32-byte block hash"})
                node = next((n for n in collector.current()["nodes"] if n["online"]), None)
                if not node:
                    return self.json_response(503, {"error": "No connected node"})
                try:
                    block = rpc(node["url"], "chain_getBlock", [block_hash], timeout=4)
                except RpcError as exc:
                    return self.json_response(502, {"error": str(exc)})
                return self.json_response(200, {"source_node": node["id"], "block": block})
            if path.path not in ("/", "/app.js", "/style.css"):
                return self.json_response(404, {"error": "Not found"})
            resource = ROOT / ("index.html" if path.path == "/" else path.path.lstrip("/"))
            data = resource.read_bytes()
            self.send_response(200)
            self.send_header("Content-Type", {".html": "text/html", ".js": "text/javascript",
                                              ".css": "text/css"}[resource.suffix] + "; charset=utf-8")
            self.send_header("Cache-Control", "no-store")
            self.send_header("X-Content-Type-Options", "nosniff")
            self.send_header("Content-Security-Policy",
                             "default-src 'self'; script-src 'self'; style-src 'self'; "
                             "connect-src 'self'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'")
            self.send_header("Content-Length", str(len(data)))
            self.end_headers()
            self.wfile.write(data)

        def log_message(self, format, *args):
            print(f"{now()} {self.client_address[0]} {format % args}")

    return Handler


def main():
    parser = argparse.ArgumentParser(description="Local read-only CALIBRE testnet observatory")
    parser.add_argument("--port", type=int, default=8765)
    parser.add_argument("--interval", type=float, default=3.0)
    parser.add_argument("--data", type=Path, default=ROOT / "data" / "observations.sqlite3")
    args = parser.parse_args()
    nodes = configured_nodes()
    manifest_path = os.environ.get("CALIBRE_PHASE94_MANIFEST")
    manifest = json.loads(Path(manifest_path).read_text()) if manifest_path else None
    if manifest and (manifest.get("format") != 1 or len(manifest.get("initial_validators", [])) != 7):
        raise ValueError("Invalid Phase 9.4 manifest")
    evidence = Evidence(args.data)
    collector = Collector(nodes, evidence, max(1.0, args.interval), manifest)
    collector.poll()
    thread = threading.Thread(target=collector.run, daemon=True)
    thread.start()
    server = ThreadingHTTPServer(("127.0.0.1", args.port), make_handler(collector, evidence))
    print(f"CALIBRE Observatory: http://127.0.0.1:{args.port}/", flush=True)
    print("Read-only; configured RPC endpoints:", ", ".join(n["url"] for n in nodes), flush=True)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()
        collector.running.clear()


if __name__ == "__main__":
    main()
