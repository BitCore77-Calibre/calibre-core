# Phase 7 — Light Nodes + ZK State Proofs

**Status:** Design locked · Implementation in progress
**Owner agent:** Rosalind (ZK & Light Node Engineer)
**Session plan:** 7.0 → 7.6 (six focused sessions)

## The problem

Calibre's UTXO set grows unboundedly. A phone cannot store it,
sync it, or validate it. Asking an RPC for a balance is trusting
an unverified third party — the exact attack surface that has
cost Web3 users billions.

## The approach: recursive ZK proofs over a Merkle commitment

Every block, validators compute a Merkle root over the UTXO set.
Every 100 blocks, an off-chain prover generates an SP1 STARK proving
the correct state transition from the previous root to the new root.
Each proof recursively includes the previous one, so only the newest
proof needs verification.

A light node downloads:
- The latest state proof (~200 KB)
- Block headers (~50 MB/year at 6s blocks)
- Merkle inclusion proofs on demand

Total permanent storage: ~2 MB. Sync time: seconds. No trust in
any RPC.

## Status (end of Phase 7.2d)

- 7.1 ✅ Merkle root tracking in pallet-qutxo.
- 7.2 ✅ RISC Zero v3.0.6 UTXO inclusion circuit.
- 7.2d ✅ `qutxo_getInclusionProof` RPC + host CLI proven against live chain.

Full loop verified end-to-end:
    chain (3 UTXOs) → RPC → host → RISC Zero guest → receipt

All three UTXOs proved against the same on-chain root, path length 2.

## Architecture

    Validators
      ├── pallet-qutxo: emits Merkle root per block
      └── libp2p pub/sub: "calibre/state-proofs/1"
    
    Prover (off-chain)
      ├── reads blocks
      ├── executes state transitions in SP1 circuit
      └── produces recursive STARK
    
    On-chain verifier
      └── pallet-zk-verifier: stores latest (root, height, issuance)
    
    Light node
      ├── subscribes to proof topic
      ├── verifies STARK
      ├── verifies Merkle inclusion of user UTXOs
      └── reports trustless balance
    
    Mobile (A.E.G.I.S.)
      └── runs the light node inside the Secure Enclave

## Design decisions

| Decision | Choice |
|:---|:---|
| Proving system | SP1 (Succinct) |
| State commitment | Binary Merkle tree, blake2b-256 leaves |
| Proof cadence | Every 100 blocks (~10 min) |
| Recursion | IVC — each proof includes previous |
| Light-node storage | Proof + headers + on-demand inclusions |
| On-chain verifier | New pallet: pallet-zk-verifier |
| Proof distribution | libp2p pub/sub topic |

## Session plan

- **7.0** — SP1 toolchain + hello world circuit
- **7.1** — MerkleRoot storage in pallet-qutxo
- **7.2** — The state-proof circuit
  - ✅ 7.2 — UTXO inclusion circuit (RISC Zero v3.0.6) — shipped
  - ⏳ 7.2d — RPC + runtime API to serve inclusion proofs from live chain
- **7.3** — Recursive proof composition (IVC)
- **7.4** — pallet-zk-verifier + on-chain STARK verification
- **7.5** — calibre-light library + libp2p pub/sub
- **7.6** — Mobile integration

## Cryptographic assumptions

- Blake2b-256: collision-resistant hash for Merkle leaves
- SP1 STARK: soundness under the SP1 security parameters
- Recursion: each proof attests to the correctness of all prior proofs
- An attacker cannot forge a state proof without breaking either Blake2b
  collision resistance or SP1's STARK soundness

## What this unlocks

- Trustless mobile wallets — no RPC dependency
- Fast cold-start for new nodes
- Verifiable bridges (the state proof becomes the source-of-truth for
  cross-chain verification in Phase 9)
- Light-client-based consensus participation (eventually)

## Open questions (resolved during implementation)

1. Merkle tree type: sparse vs. dense. Leaning sparse — cheaper for
   inclusion proofs of specific UTXOs.
2. Proof cadence: 100 blocks vs. adaptive. Start with 100, adjust.
3. Recursion cost: SP1 recursion is not free. Will measure and
   potentially batch.
4. Mobile verifier: SP1 has a WASM verifier, but mobile needs arm64.
   May need to write a custom verifier.
