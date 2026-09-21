# Calibre Whitepaper — Part 3: Fast-Path Mempool and Interoperability

## 5. Fast-Path Mempool

### 5.1 Motivation

In traditional blockchains, all transactions — conflicting or not — must
wait for block inclusion. The block interval bounds system throughput.
For a 6-second block, a payment's minimum confirmation latency is 6
seconds regardless of whether it conflicts with any other transaction.

Calibre's UTXO model lets us do better. Because UTXO conflicts are
*syntactic* (shared inputs), a node can determine whether two pending
transactions conflict without executing either. This enables a two-tier
mempool:

- **Fast Path** — non-conflicting transactions are broadcast immediately
  to peers with no dependency on block production.
- **Slow Path** — conflicting transactions are held in a dependency-ordered
  queue and routed to the standard block producer.

### 5.2 Implementation in Substrate

Substrate's transaction pool supports this via per-transaction "tags".
A tag is a byte string that uniquely identifies a resource the transaction
consumes. Two transactions that provide the same tag conflict.

Calibre's validate_unsigned generates one tag per input:

    for input in tx.inputs.iter() {
        let id = Self::calculate_utxo_hash(input.tx_hash, input.output_index);
        if !UtxoSet::<T>::contains_key(&id) {
            return InvalidTransaction::Stale.into();
        }
        builder = builder.and_provides(id.as_ref().to_vec());
    }

Two effects follow:

1. **Double-spend rejection.** If two transactions claim the same UTXO,
   they provide the same tag. The pool rejects the second.
2. **Replay rejection.** If a UTXO is already spent on-chain, the pool's
   UtxoSet::contains_key check fires and returns Stale — before the
   block producer even sees the transaction.

### 5.3 Demonstration

The testnet includes scripts that demonstrate both effects:

- **Mempool duel** — two conflicting transactions submitted in parallel.
  One succeeds; the other returns Priority is too low.
- **Replay test** — the same transaction submitted after block inclusion.
  Returns Transaction is outdated.

### 5.4 What this does NOT yet deliver

True "blocklessness" — settlement without any block — requires a different
consensus model (DAG, hashgraph, or BFT without slots). Calibre's Fast-Path
reduces mempool latency and eliminates mempool-level double-spends, but
finality still depends on Aura + GRANDPA block production. This is a
deliberate choice: it preserves Substrate's battle-tested consensus
guarantees in exchange for a small amount of latency on the fast path.

---

## 6. Interoperability — Intent Bridge (Roadmap)

### 6.1 Design principles

- **No wrapped tokens.** Solvers front liquidity on Calibre from their own
  balance; they are reimbursed from a bridge pool, not minted into existence.
- **No multi-signature custodian.** Cross-chain settlement is verified by
  Zero-Knowledge proofs, not by a committee of signers.
- **No honeypot.** There is no locked asset pool to drain — the only
  collateral is Solver bonds, which are cryptographically slashed on fraud.

### 6.2 Flow (design)

1. User on Ethereum locks an asset in a CalibreIntent contract and
   broadcasts an intent: "I will pay 1 ETH to whoever sends me 5,000
   CAL on Calibre."
2. A Solver observes the intent, sends 5,000 CAL to the user's lock
   on Calibre, and obtains a signed receipt.
3. The Solver submits a ZK proof to the Calibre verifier: "I have
   observed an Ethereum block containing a valid IntentFulfilled event
   that matches this intent."
4. The verifier checks the proof and releases the reimbursement to the
   Solver from the bridge pool.

### 6.3 Status

Design complete. No code deployed. Full implementation is Phase 9.

---

## 7. Tokenomics — $CAL

### 7.1 Roles

$CAL is the native token of the Calibre network. It has three roles:

- **Gas** — every Q-UTXO transaction burns an implicit fee equal to
  Σ inputs − Σ outputs.
- **Collateral** — Solvers on the Intent Bridge (roadmap) post $CAL
  bonds, which are slashed on fraudulent ZK proofs.
- **Governance** — $CAL holders vote on protocol parameters including
  hybrid signature activation, fee schedules, and runtime upgrades.

### 7.2 Supply

*Roadmap.* Current testnet mints are unlimited (dev-only). Mainnet
supply will be capped. The exact emission schedule, allocation, and
deflation curve are not yet finalized.

### 7.3 Fee mechanics

Every spend burns Σ inputs − Σ outputs from the total issuance. This is
implemented in the current testnet as a passive burn: the difference
between consumed and created UTXOs is simply not re-minted.

---

*Continue to Part 4: Status and Roadmap.*
