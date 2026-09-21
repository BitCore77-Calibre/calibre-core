# Calibre Protocol — Whitepaper

**Version:** 0.6.0-testnet
**Date:** September 2026
**Status:** Public draft — not yet audited

> **Note on claims.** This document uses two distinct vocabularies.
> "Calibre ships X" means X is implemented and demonstrable on the current testnet.
> "Calibre's roadmap includes Y" means Y is designed but not yet built.

---

## Abstract

Calibre Protocol is a quantum-resistant Layer-1 blockchain. Its central
innovation is a **Quantum-UTXO (Q-UTXO) state model**: every unit of value
is an unspent transaction output cryptographically bound to a post-quantum
lock derived from a **FIPS 204 ML-DSA-44** (Dilithium) public key. Every
spend is verified at block execution by real lattice cryptography running
inside the WASM runtime. Forgery attempts and double-spends are rejected
by four independent verification gates.

Unlike legacy blockchains, which must migrate to post-quantum cryptography
through hard forks or user-initiated wallet migrations, Calibre was designed
PQ-first. The QuantumLock enum allows governance-driven cryptographic
migration without a chain fork.

## Table of Contents

- **Part 1**: Abstract, Problem, Architecture (this file)
- **Part 2**: Cryptographic Specification, Verification Paths
- **Part 3**: Fast-Path Mempool, Interoperability, Tokenomics
- **Part 4**: Status, Roadmap, References

---

## 1. Problem Statement

### 1.1 The quantum threat

Every major blockchain today — Bitcoin, Ethereum, Solana, Polkadot —
relies on elliptic-curve cryptography (ECDSA or Ed25519). A sufficiently
large quantum computer running Shor's algorithm can invert these signature
schemes in polynomial time. Estimates for a cryptographically relevant
quantum computer (CRQC) range from 8 to 30 years. Whenever it arrives,
every wallet with an exposed public key becomes drainable.

### 1.2 The seed-phrase problem

Twelve- and twenty-four-word seed phrases are a single point of failure.
Annual losses from seed-phrase compromise exceed $1B.

### 1.3 The bridge honeypot problem

Cross-chain bridges (Ronin: $625M, Wormhole: $320M, Nomad: $190M) hold
locked assets under small multi-sig committees. They are centralized
exchanges with worse disclosure.

### 1.4 The blind-signing problem

Users sign opaque bytecode. A wallet displays "swap 1 ETH for 2000 USDC"
but the payload may contain hidden drains or unlimited approvals.

### 1.5 The design goal

Calibre addresses all four threats at the protocol layer.

---

## 2. Architecture

### 2.1 The Q-UTXO state model

In the Account Model (Ethereum), a user's balance is a number stored next
to their address. In the UTXO model (Bitcoin), a user's balance is the sum
of unspent outputs their key can unlock.

Calibre extends UTXO with a post-quantum lock:

    pub enum QuantumLock {
        AegisThreshold([u8; 32]),    // blake2_256(ML-DSA-44 pk)
        HybridPqClassical {          // roadmap
            pq_lock: [u8; 32],
            classical_lock: [u8; 32],
        },
    }

    pub struct Utxo<Balance> {
        pub value: Balance,
        pub lock: QuantumLock,
    }

A UTXO is identified by blake2_256(origin_tx_hash || output_index). There
is no AccountId in the state — the state is purely a map from UTXO ID to
Utxo { value, lock }.

### 2.2 Why UTXO, not accounts

In account-based chains, two transactions might conflict but the conflict
is only discoverable by execution. In a UTXO chain, two transactions
conflict if and only if they share an input. This is a syntactic property,
checkable in microseconds without execution.

This property enables:
- Mempool-level double-spend rejection
- Parallel verification of non-conflicting transactions
- Deterministic conflict detection independent of node state

### 2.3 The four verification gates

Every signed UTXO spend passes through four independent gates:

1. **Mempool** — per-input conflict tags. Rejects double-spends in microseconds.
2. **State** — UtxoSet::contains_key(id). Rejects replays with Stale.
3. **Runtime** — blake2_256(witness_pk) == UTXO.lock.
4. **Crypto** — MlDsaKeyPair::verify(sig, payload, pk).

Each gate catches a distinct class of attack. Failing any gate is
sufficient for rejection. An attacker must defeat all four.

---

*Continue to Part 2: Cryptographic Specification.*
