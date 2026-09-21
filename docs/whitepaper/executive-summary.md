# Calibre Protocol — Executive Summary

## The problem

Every major blockchain today relies on **elliptic-curve cryptography**
(ECDSA or Ed25519) to prove ownership of funds. A future quantum computer
running Shor's algorithm can mathematically invert these signatures. Every
wallet with an exposed public key is at risk.

Beyond quantum, current blockchains also inherit three systemic problems:
seed-phrase fragility, bridge honeypots, and blind smart-contract execution.

## The Calibre approach

Calibre Protocol is a **Layer-1 blockchain** built from the ground up for
the post-quantum era. Its defining choices:

1. **Quantum-UTXO state model** — every unit of value is an unspent
   transaction output cryptographically bound to a post-quantum lock.

2. **Real on-chain ML-DSA-44 verification** — every spend is verified at
   block execution by NIST-standardized lattice cryptography (FIPS 204).

3. **Four independent verification gates** — mempool (conflict tags),
   state (UtxoSet::contains_key), runtime (lock binding), and cryptographic
   (ML-DSA-44 signature check).

4. **Fast-Path conflict detection** — UTXO conflicts are syntactic, making
   parallel verification possible and enabling mempool-level double-spend
   rejection in microseconds.

## What is proven (not claimed)

| Property | Evidence |
|:---|:---|
| Real ML-DSA-44 verification | Phase 5 commit, forgery-rejection test |
| Lock hash ↔ pubkey binding | Enforced in verify_aegis_transaction |
| Double-spend rejected | Mempool conflict detector fires on collision |
| Replay rejected | Stale gate returns Transaction is outdated |
| Conservation of mass | Unit tested |
| Named testnet | Calibre Testnet 1, SHA-256 7b7ba4d1... |
| One-command deployment | docker compose up |

## What is not yet built (roadmap)

- Multi-validator testnet (currently 1 authority)
- A.E.G.I.S. threshold MPC
- Hardware attestation verification
- ZK Intent Bridge
- ZkShield private UTXOs
- Third-party audit

## Comparison

| Property | Bitcoin | Ethereum | QRL | Calibre |
|:---|:---|:---|:---|:---|
| State model | UTXO | Account | Account | Q-UTXO |
| Signature | secp256k1 | secp256k1 | XMSS | ML-DSA-44 |
| Quantum-safe | ❌ | ❌ | ✅ | ✅ |
| Mempool-level conflict detection | ❌ | ❌ | ❌ | ✅ |
| Forkless crypto migration | ❌ | ⚠️ | ❌ | ✅ |

## Status and next steps

- **Today:** working testnet, one-command deployment, documented cryptography.
- **Next:** multi-validator testnet, audit prep, ZK bridge.
- **Eventual:** production mainnet.

Calibre Protocol is **not** a finished product. It is a working research
prototype that demonstrates a specific claim: *a UTXO Layer-1 can verify
real post-quantum signatures on-chain today, without sacrificing throughput
or composability.*
