# CALIBRE Testnet Observatory

This package serves a 20-panel, 4 × 5 monitor wall and polls **actual local Polkadot SDK JSON-RPC nodes**. It is read-only. Offline nodes and unavailable protocol integrations are visibly marked. It does not start a chain, create a genesis allocation, hold signing keys, submit transactions, or pretend an unavailable feature is live.

## Run locally

Copy this `calibre-observatory` directory into `~/calibre-template/observatory` on the machine running your node. The UI can also run from any other directory on the same computer.

For one node using the usual local HTTP JSON-RPC port:

```bash
cd ~/calibre-template/observatory
python3 server.py --port 8765
```

For seven separately running local nodes, change the ports to match your actual node commands:

```bash
cd ~/calibre-template/observatory
CALIBRE_RPC_URLS='http://127.0.0.1:9944,http://127.0.0.1:9945,http://127.0.0.1:9946,http://127.0.0.1:9947,http://127.0.0.1:9948,http://127.0.0.1:9949,http://127.0.0.1:9950' python3 server.py --port 8765
```

Open **http://127.0.0.1:8765/** in the same computer's browser. Keep the real CALIBRE nodes running in separate terminals. The observatory never launches or restarts nodes; its first version is deliberately read-only.

To check an expected genesis hash, set `CALIBRE_EXPECTED_GENESIS_HASH=0x...` to the exact 32-byte hash when starting the server. A mismatch is recorded in Audit & Incidents. Setting the expected hash does not configure issuance or rewrite genesis.

## Evidence implemented now

- Per-node chain name, genesis hash, best height/hash, finalized height/hash, peer count and version when exposed.
- A **per-node snapshot** of pending extrinsics when `author_pendingExtrinsics` is enabled. This RPC does not expose ready/future details; the UI says so. Polling snapshots cannot prove every transient admission or drop.
- An on-demand raw latest block read (`chain_getBlock`) in Block Production, Block Execution, and Explorer panels. The raw bytes are not falsely decoded as QUTXO events.
- Local SQLite observation history containing node connection changes, head changes, finality changes, collector errors and genesis mismatch detection. Records have chained SHA-256 digests for local accidental-modification detection; they are not externally attested.
- Local-only HTTP server. Only configured loopback RPC endpoints and an allowlist of read-only RPC methods are used. No browser-supplied RPC URL or arbitrary RPC method is accepted.

## Phase 9.4 local election

The user-started launcher at `tools/run_phase94_local.sh` builds and starts a disposable eight-node lab and configures this Observatory to read finalized `Session`, BABE, and GRANDPA state. Open **Slots & epochs** for the live session index, validator and authority sets, common finalized hash, and PASS/FAIL status. The fixture uses synthetic stake and genesis-prepared keys; see `docs/PHASE9_4_LOCAL_RUN.md` for the exact command, checks, and limits. No result is claimed until the user runs it.

Other panels still need runtime metadata decoding, transaction tracing, wallet integration, fee and supply invariant checks, and additional experiment correlation. A block or finality observation does not establish those features.

## API

- `GET /api/snapshot`: latest per-node RPC observations.
- `GET /api/events?limit=100`: recent local observation records.
- `GET /api/block/0x...`: raw block from the first reachable configured node.

Configuration: `CALIBRE_RPC_URLS` is a comma-separated list of 1–20 loopback HTTP(S) root endpoints. `CALIBRE_EXPECTED_GENESIS_HASH` is optional. `--interval` changes the collector period; `--data` changes the SQLite path. This package uses only Python's standard library.

## Verification

```bash
cd ~/calibre-template/observatory
python3 -m unittest discover -s tests -v
```

The tests start a temporary local fake RPC server to check transport and evidence recording. They do not establish that the user's current CALIBRE chain builds or runs. A live-chain acceptance test remains necessary.
