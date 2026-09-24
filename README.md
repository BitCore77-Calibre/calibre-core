# Calibre Protocol

**A post-quantum Layer-1 blockchain with real FIPS 204 ML-DSA-44 (Dilithium) verification on-chain.**

Calibre replaces elliptic-curve cryptography in the value path with NIST-standardized lattice cryptography. Every UTXO is bound to a QuantumLock; every spend is verified at block execution by MlDsaKeyPair::verify().

> **Status:** Research prototype. Not audited. Not for real value transfer.

Runtime 102 binds staking witnesses to the beneficiary account and chain.
Old bond witnesses must be regenerated; see [the signing format](docs/STAKING_SIGNATURES.md).
This repair is not an audit or approval for real-value use.

---

## Quick Start

Use the [local BABE launch guide](chain-specs/README.md). It builds the pinned
Rust 1.88 toolchain and starts a fresh temporary development chain.
The first build can take substantial time.

This source includes the BABE and runtime-102 security repairs, validated locally.
Source publication is not a mainnet release or a live network upgrade.
Existing raw Aura specifications and Docker launch recipes are historical,
not validated BABE launch instructions. Do not reuse an Aura database.

## Implementation and validation

- Real ML-DSA-44 verification in the WASM runtime
- Lock hash <-> pubkey binding: blake2_256(witness_pk) == UTXO.lock
- Forgery rejection at block execution
- Double-spend rejection at mempool: Priority is too low
- Replay rejection: Transaction is outdated
- Conservation of mass: sum(inputs) >= sum(outputs) — un-ignored 2026-09-23 with real ML-DSA-44 keygen in tests
- Under-priced tx rejection at both pool admission and runtime execution
- Multi-input spends and staking require one identical authorized lock across every input; mixed owners, duplicates, missing inputs and value overflow are rejected before mutation
- Local BABE baseline `ec400e4`: seven validators crossed the one-hour epoch boundary and finalized block 612. This is not a throughput, Byzantine-fault or production-readiness test
- Fee market: EIP-1559-style dynamic base fee, 50/30/20 split, block rewards, priority fee 100% producer
- Producer payout routing: fees mint to producer UTXO at block end; unregistered producers accrue pending, never burned
- Stake pallet: bond/unbond with a burn-and-record ledger

Run the current native tests with `SKIP_WASM_BUILD=1 cargo test --locked --workspace`.
See [the local repair record](docs/REVIEW_REPAIRS.md) for scope and validation.
Historical Docker throughput and latency results in the project log apply
to their original revision, not automatically to this BABE runtime.

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

Historical milestone groups follow. Current local BABE repair status is recorded
in [Session state](docs/SESSION_STATE.md); these milestones are not audit signoff.

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

CALIBRE-owned code: [Unlicense](LICENSE). Third-party code and dependencies
retain their own licenses and notices, including the Apache-2.0 notice in
`zk/LICENSE` and upstream template notices.

---

*Calibre Protocol is a research prototype.*
