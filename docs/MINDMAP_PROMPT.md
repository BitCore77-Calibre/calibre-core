# Calibre Protocol — Visual Mind Map Prompt

Paste the block below into any AI image or mind-map tool: Napkin.ai,
Whimsical AI, Miro AI, ChatGPT, Claude, Mermaid Live, or XMind.

For the fastest result with zero tooling, skip this and paste
`docs/diagrams/07-protocol-map.mmd` directly into https://mermaid.live.

---

## PROMPT (copy everything below this line)

Create a comprehensive visual mind map of a post-quantum Layer-1 blockchain
called Calibre Protocol.

Style: dark technical. Accent colors:

- Green (#3fb950) for shipped
- Amber (#d29922) for partial
- Gray (#8b949e) for planned
- Red (#f85149) for unscheduled work

Layout: radial or layered with clear grouping. Add a legend.

Central node: CALIBRE PROTOCOL — v0.8.9 · 68 tests passing · post-quantum Layer-1

### Branch 1 — Problem being solved

- Quantum threat: Shor's algorithm breaks ECDSA/Ed25519 in 8-30 years
- Seed-phrase fragility: USD 1B+/year in losses
- Bridge honeypots: Ronin 625M, Wormhole 320M, Nomad 190M
- Blind signing: users sign opaque bytecode
- Design goal: PQ-first, four threats addressed at protocol layer

### Branch 2 — Architecture (all SHIPPED, green)

- Q-UTXO state model — map of UTXO id to {value, lock}, no AccountId
- UTXO identifier — blake2_256(origin_tx_hash || output_index)
- QuantumLock enum — AegisThreshold (production), SingleSig (test-only)
- Four verification gates, as four sub-branches:
  - Mempool: per-input conflict tags, microsecond double-spend rejection
  - State: UtxoSet::contains_key, rejects replays
  - Runtime: blake2_256(witness.pk[0]) == utxo.lock, rejects lock mismatch
  - Crypto: MlDsaKeyPair::verify, rejects forgeries
- Fast-Path mempool — syntactic conflict detection enables parallel verification

### Branch 3 — Cryptography (all SHIPPED, green)

- ML-DSA-44 — NIST FIPS 204, CRYSTALS-Dilithium lattice, Level 2
- Key sizes: public 1312B, secret 2560B, signature 2420B
- Verify cost: ~208 microseconds per signature
- Library: dilithium-rs 0.4, pure Rust, no_std
- Lock binding: lock = blake2_256(pub_key)
- Sign flow: keygen, hash to lock, sign (inputs, outputs).encode() with empty context
- Verify flow: decode witness, match lock, MlDsaKeyPair::verify

### Branch 4 — Consensus (SHIPPED, green)

- Aura block production, 6-second slots
- GRANDPA finality, ~2-block lag (12 seconds)
- Validator set: n=7, f=2, threshold 5/7
- Author selection: slot modulo authorities.len()
- Verified: full peer mesh, 1s RTT survived, 169 TPS single vs multi parity

### Branch 5 — Economic layer (all SHIPPED, green)

- Fee market — EIP-1559-style dynamic base fee
  - Minimum fee: base_fee + per_in_out_fee * (inputs + outputs)
  - Base fee 1000, per-in-out 100, range 500 to 1,000,000
  - Adjustment: 12 percent max per block, target 50 percent fullness
- Fee split — 50 percent producer, 30 percent treasury, 20 percent burn
- Block rewards — 5.3 CAL per block, 4-year halving, 1B cap
- Reward split — 70 percent producer, 30 percent treasury
- Priority fee — anything above minimum, 100 percent to producer
- Producer payout routing — settle at block end, pending if no lock, never burn
- Stake pallet — bond consumes UTXOs into ledger, unbond mints back
- Genesis: 777M allocation, self-funded bootstrap (EUR 13K, no VC)

### Branch 6 — Partial work (amber)

- Light client — calibre-light v0.4 shipped, production mobile app pending
- ZK verifier — risc0-zkvm 3.0.6 on-chain, verified end-to-end
- Merkle inclusion proofs — calibre-merkle plus QutxoApi RPC endpoint
- Audit prep — hand-derived weights landed; fuzz, try-runtime, audit pending
- Hardware attestation — fields exist, never verified; AppAttest no-op
- Throughput — empty blocks while txs pend, cause unknown
- Treasury — accumulates but nothing withdraws it

### Branch 7 — Not started (gray)

- ZK Intent Bridge — solver network, CAL bonding and slashing
- ZkShield — Pedersen commitments plus nullifiers
- HybridPqClassical lock — second QuantumLock variant
- A.E.G.I.S. threshold MPC — single key today, multi-party planned
- Seed-phrase elimination — stated goal, no design
- Cryptographic governance — migration path claimed, no mechanism
- Mainnet — final tokenomics, launch ops

### Branch 8 — Unscheduled work (red, discovered by testing)

- Stake gating — bond exists but gates nothing
- Unbonding period — no delay means flash-stake attack
- Slashing conditions — no risk, stake secures nothing
- Validator selection by stake — Aura uses fixed list
- Dual-key staking — Cardano model, protocol redesign
- TotalIssuance integrity — stale on execute_utxo_tx path
- Whitepaper refresh — still v0.6.0, no economic layer

### Branch 9 — Timeline of commits, 2026-09-21 to 09-23

- 09-21: Core engine, testnet, 7-validator finality, ZK design, Merkle
- 09-22: calibre-light v0.4, RISC Zero verifier, anti-spam, honest weights
- 09-23: Fee market, producer payouts, stake, test harness, bench fees, docs

### Branch 10 — Revised plan, proposed sub-phases

- 6.8 Stake gating
- 6.9 Treasury governance
- 6.10 Economic model v1
- 8.5 Throughput investigation
- 10.5 Dual-key staking
- 11.5 Whitepaper v1.0
- Key correction: Phase 6.6 done was premature; Phase 6 stays open until 6.10

### Layout guidance

- Center: CALIBRE PROTOCOL
- 5 primary branches: Architecture, Cryptography, Consensus, Economics, Roadmap
- Sub-branches fan out under each, color-coded by status
- Legend: green shipped, amber partial, gray planned, red unscheduled
- Inline formulas where relevant (fee split, base fee adjustment, lock binding)
- Technical style, monospace labels for identifiers, no clip art

## END PROMPT

---

## Tool notes

| Tool | What to do |
|---|---|
| Mermaid Live | Paste docs/diagrams/07-protocol-map.mmd directly. Fastest, no install. |
| Napkin.ai | Paste the prompt. Best for text-heavy mind maps. |
| Whimsical AI | Paste into "Generate with AI". Clean radial layouts. |
| Miro AI | Paste into a new board, use "Generate mind map". |
| ChatGPT or Claude | Paste the prompt, ask for Mermaid or PlantUML output. |
| XMind | Import the outline structure manually. |
