# Calibre Protocol

**A post-quantum Layer-1 blockchain with real FIPS 204 ML-DSA-44 (Dilithium) verification on-chain.**

Calibre replaces elliptic-curve cryptography in the value path with NIST-standardized lattice cryptography. Every UTXO is bound to a QuantumLock; every spend is verified at block execution by MlDsaKeyPair::verify().

> **Status:** Research prototype. Not audited. Not for real value transfer.

---

## Quick Start

Prerequisite: [Docker](https://docs.docker.com/get-docker/).

    git clone https://github.com/BitCore77-Calibre/calibre-core.git
    cd calibre-core
    docker compose up

Then in another terminal:

    docker compose exec node pq-signer keygen > pubkey.txt
    curl -X POST http://127.0.0.1:8090/faucet -H "Content-Type: application/json" -d "{\"pubkey\": \"$(cat pubkey.txt)\"}"

You now hold 1,000,000,000 testnet CAL, bound to a Dilithium key that only your private key can spend.

## What is proven

- Real ML-DSA-44 verification in the WASM runtime
- Lock hash <-> pubkey binding: blake2_256(witness_pk) == UTXO.lock
- Forgery rejection at block execution
- Double-spend rejection at mempool: Priority is too low
- Replay rejection: Transaction is outdated
- Conservation of mass: sum(inputs) >= sum(outputs) — un-ignored 2026-09-23 with real ML-DSA-44 keygen in tests
- Under-priced tx rejection at pool admission (`InvalidTransaction::Payment`)
- 7-validator Docker testnet: GRANDPA finality at 5/7, 1s RTT survived, 169 TPS parity
- Fee market: EIP-1559-style dynamic base fee, 50/30/20 split, block rewards, priority fee 100% producer
- Producer payout routing: fees mint to producer UTXO at block end; unregistered producers accrue pending, never burned
- Stake pallet: bond/unbond with a burn-and-record ledger

**68 tests passing, 0 ignored.** `SKIP_WASM_BUILD=1 cargo test --workspace`

## Documentation

**Start here:** [docs/PROJECT_LOG.md](docs/PROJECT_LOG.md) — the commit-by-commit
record of what shipped, when, and why. The single source of truth.

- [Project log](docs/PROJECT_LOG.md) — every commit, decision, and test, in order
- [Session state](docs/SESSION_STATE.md) — current phase, locked decisions, open debt
- [Whitepaper](docs/whitepaper/README.md)
- [Diagrams](docs/diagrams/)
- [Tokenomics](docs/TOKENOMICS.md)
- [Economic model](docs/ECONOMIC_MODEL.md)
- [Fee market](docs/FEE_MARKET.md)
- [Bootstrap plan](docs/BOOTSTRAP_PLAN.md)
- [Security policy](SECURITY.md)
- [Contributing](CONTRIBUTING.md)

## Status

| Phase | Milestone | Status |
|---|---|---|
| 1-5 | Q-UTXO, ML-DSA-44, Fast-Path, Console, Docker | Done |
| 6.1-6.5 | Testnet, faucet, whitepaper, diagrams, GitHub | Done |
| 6.6 | Multi-validator testnet | Done |
| 6.7 | Fee market + staking | Done |
| 7 | Light nodes + ZK state proofs | In progress (calibre-light v0.4, RISC Zero verifier, Android cross-compile shipped; production mobile app pending) |
| 8 | Audit preparation | In progress (hand-derived weights landed; frame-benchmarking, fuzz, try-runtime, third-party audit pending) |
| 9 | ZK Intent Bridge | Planned |
| 10 | ZkShield private UTXOs | Planned |
| 11 | Final tokenomics + mainnet | Planned |

For the full phase table with per-phase commit anchors, see
[docs/PROJECT_LOG.md](docs/PROJECT_LOG.md).

## License

GPL-3.0-only. See LICENSE.

---

*Calibre Protocol is a research prototype.*