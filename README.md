# Calibre Protocol

**A post-quantum Layer-1 blockchain with real FIPS 204 ML-DSA-44 (Dilithium) verification on-chain.**

Calibre replaces elliptic-curve cryptography in the value path with NIST-standardized lattice cryptography. Every UTXO is bound to a QuantumLock; every spend is verified at block execution by MlDsaKeyPair::verify().

> **Status:** Research prototype. Not audited. Not for real value transfer.

---

## Quick Start

Prerequisite: [Docker](https://docs.docker.com/get-docker/).

    git clone https://github.com/calibre-protocol/calibre-template
    cd calibre-template
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
- Conservation of mass: sum(inputs) >= sum(outputs)

## Documentation

- [Whitepaper](docs/whitepaper/README.md)
- [Diagrams](docs/diagrams/)
- [Security policy](SECURITY.md)
- [Contributing](CONTRIBUTING.md)

## Status

| Phase | Milestone | Status |
|---|---|---|
| 1-6.4 | Q-UTXO, ML-DSA-44, Console, Fast-Path, Faucet, Docker | Done |
| 6.5 | GitHub publication | Pending |
| 6.6 | Multi-validator testnet | Planned |
| 7-11 | Light nodes, audit, ZK bridge, privacy, mainnet | Planned |

## License

GPL-3.0-only. See LICENSE.

---

*Calibre Protocol is a research prototype.*