# Calibre Protocol — Project Log

**Purpose:** the single source of truth for what has shipped, in what order,
proven by which commit. If you're new to the repo, read this first. If you're
returning after weeks away, read this first. If the website roadmap and this
file disagree, this file wins.

**Last updated:** 2026-09-23 at commit `5431db1` on `main`.

---

## Where to look for what

| Question | File |
|---|---|
| What shipped, when, in which commit? | **this file** |
| What is the current phase, decisions, debt? | `docs/SESSION_STATE.md` |
| What is the protocol supposed to be? | `docs/whitepaper/` |
| What does the public roadmap say? | `gh-pages` branch, `index.html` |
| What actually changed in a commit? | `git show <hash>` |
| What tests exist and what do they prove? | this file, Section "Test inventory" |

---

## Phase table

The website groups phases 1-5 into one row and 6.1-6.5 into another. The
actual commit history shows more granularity. This table is derived from
commit messages, which are authoritative.

| Group | Phase | Name | Status | Anchor commit |
|---|---|---|---|---|
| Engine | 1-5 | Q-UTXO, ML-DSA-44, Fast-Path, Console, Docker | shipped | `dc752b0` |
| Publication | 6.1-6.5 | Testnet, faucet, whitepaper, diagrams, GitHub | shipped | `dc752b0` … `bded0c2` |
| Consensus | 6.6 | 7-validator testnet (n=7, f=2) | shipped | `3fcb533`, `00d21f6` |
| ZK / light | 7.0 | Design + RISC Zero selected (SP1 abandoned) | shipped | `2d1734d`, `9dcad9b` |
| ZK / light | 7.1 | Merkle root tracking in pallet-qutxo | shipped | `9b64644` |
| ZK / light | 7.2 | calibre-merkle crate + inclusion proof | shipped | `6330be9`, `dacfbcd` |
| ZK / light | 7.2d | QutxoApi crate + RPC endpoint | shipped | `fb923a7` … `fa63111` |
| ZK / light | 7.4 | On-chain RISC Zero verifier | shipped | `d922a04`, `d7d61fe` |
| ZK / light | 7.5 | calibre-light v0.1-v0.4 | shipped | `21ac57a` … `5b68d4c` |
| ZK / light | 7.6 | Android app | in progress | `08c2696`, `112b199` |
| Consensus | 7.5b | Anti-spam, honest weights | shipped | `4e6c0ab`, `f8e9e23` |
| Consensus | 7.5c | 7-authority staging chain spec | shipped | `f2e6c4b` |
| Consensus | 6.6-fu | 7-validator Docker testnet | shipped | `faed330` |
| Fee market | 8.1 | pallet-calibre-fees scaffold | shipped | `06766f9` |
| Fee market | 8.2 | FeeHandler wired into qutxo | shipped | `530cfce` |
| Fee market | 8.3 | Pool rejects under-priced txs | shipped | `875c75a` |
| Fee market | 8.4 | Dynamic base fee | shipped | `8b5e264` |
| Fee market | 8.5 | Block rewards + tokenomics | shipped | `301a45d` … `24f8afe` |
| Fee market | 8.6 | Producer payout routing | shipped | `16da3d6` |
| Staking | 8.7 | Stake pallet — bond/unbond | shipped | `a73f95f` |
| Testing | 8.8 | Real ML-DSA-44 witness harness | shipped | `b47530a` |
| Testing | 8.9 | Bench txs pay fees | shipped | `ae8a5a1` |
| Docs | 8.x | README + site sync | shipped | `5431db1` |
| Audit | site-8 | Benchmark weights, fuzz, try-runtime, audit | in progress | partial (`f8e9e23`) |
| Bridge | 9 | ZK Intent Bridge | not started | — |
| Privacy | 10 | ZkShield | not started | — |
| Launch | 11 | Final tokenomics + mainnet | not started | — |

**Important:** session work labelled "Phase 8.1-8.9" (fee market + staking)
does NOT correspond to the website's "Phase 8" (audit preparation). The
website was updated on 2026-09-23 to add the fee market + staking work as
`6.7`. Going forward, prefer phase names over bare numbers.


---

## Full commit history

58 commits, 2026-09-21 to 2026-09-23. Abbreviated to 7-char hashes.
`git show <hash>` for full detail.

### 2026-09-21 — Core engine, testnet, ZK design

    dc752b0  Calibre Protocol v0.6.0-testnet — Q-UTXO + ML-DSA-44 + Docker + docs
    0bfc17f  website: expanded landing page
    bed91f2  website: new hero
    87dbd63  website: new hero
    843d356  fix: deploy script uses git worktree
    3fcb533  phase 6.6: working 7-validator testnet (n=7 f=2 BFT)
    c305fd9  docs: mark phase 6.6 as shipped
    2d1734d  phase 7.0: light node + ZK state proof design document
    9dcad9b  phase 7.0: RISC Zero selected (SP1 abandoned)
    9b64644  phase 7.1: Merkle root tracking in pallet-qutxo
    00d21f6  phase 6.6: 7-validator full-mesh testnet launcher
    f004def  chore: cargo lock update
    6330be9  phase 7.2: extract Merkle logic to calibre-merkle
    075505c  fix(calibre-merkle): dedup test module
    342e25c  chore: untrack scratch diag2.sh
    c055d82  phase 7.2: merkle_path + RISC Zero guest scaffold
    65ce26b  chore: ignore zk/target/
    dacfbcd  phase 7.2: end-to-end UTXO inclusion proof verified
    8dfa185  phase 7.2d: InclusionProof primitives + pallet helper
    7065517  phase 7.2d: runtime implements QutxoApi
    fb923a7  phase 7.2d: QutxoApi crate + workspace wiring
    a21e0b7  phase 7.2d: qutxo_getInclusionProof RPC endpoint
    fa63111  phase 7.2d: host fetches inclusion proof from live RPC
    8e94502  docs: mark 7.1, 7.2, 7.2d as shipped
    bff9a78  docs: block time, throughput plan, STARK rationale

### 2026-09-22 — Light client, ZK verifier, anti-spam, finality

    21ac57a  phase 7.5: calibre-light v0.1 — trust tier 1 light client
    16c6159  phase 7.5 v0.2: UniFFI bindings for Swift and Kotlin
    31fef5f  phase 7.5 v0.3: tier 2 state-proof prototype verified
    a96ddbc  phase 7.5 v0.3: tier 2 state-proof verification
    d707979  phase 7.5 v0.3: regenerate UniFFI bindings
    5b68d4c  phase 7.5 v0.4: freshness layer + tier-1/tier-2 upgrade
    e6cdb5d  phase 7.5 v0.4: regenerate UniFFI bindings
    08c2696  phase 7.6-prep: mobile build script + integration guide
    112b199  phase 7.6-prep: Android cross-compile verified
    d922a04  phase 7.4: on-chain verifier infrastructure (upstream-blocked)
    2849876  7.4 infra + 7.5b throughput bench
    d7d61fe  7.4: use risc0-zkvm 3.0.6 for on-chain verification (unblocked)
    8b8e401  docs: session state snapshot at 7.4 completion
    4e6c0ab  anti-spam: full ML-DSA verify in validate_unsigned + 9 tests
    f8e9e23  weight: honest weights for execute_utxo_tx
    f2e6c4b  validator: 7-authority staging chain spec + session-key wiring
    e11f7fe  docs: session state snapshot at 7-validator finality
    faed330  deploy: 7-validator docker testnet
    00be795  docs: multi-validator docker test results
    73b710f  docs: multi-validator throughput benchmark

### 2026-09-23 — Fee market, staking, tests, docs

    06766f9  fee-market: scaffold pallet-calibre-fees (Phase 8.1)
    530cfce  fee-market: wire FeeHandler into qutxo (Phase 8.2)
    875c75a  fee-market: reject under-priced txs at pool admission (Phase 8.3)
    7792a9a  docs: session state snapshot at Phase 8.3
    8b5e264  fee-market: dynamic base fee (Phase 8.4)
    301a45d  fee-market: block rewards + tokenomics + economics
    e2912eb  tokenomics: final 777M genesis (5.3 CAL/block)
    24f8afe  docs: self-funded bootstrap plan
    4e65e6f  deploy: WAN multi-validator deployment scripts
    16da3d6  fee-market: producer payout routing (Phase 8.6)
    a73f95f  stake: bond/unbond pallet (Phase 8.7)
    b47530a  test: real ML-DSA-44 witness harness + fee-rejection E2E (Phase 8.8)
    ae8a5a1  bench: bench txs now pay fees (Phase 8.9)
    5431db1  docs: README sync — fix clone URL, expand proven, link project log

### gh-pages branch (site)

    bded0c2  deploy: landing page + mempool
    637d516  site: sync roadmap with Phase 6.6 → 6.7 shipped
    1d520e3  site: refine Phase 7 + add shipped items from commit history
    a4bc40f  site: version badge + link PROJECT_LOG in nav

---

## Test inventory

Total: **68 tests, 0 failed, 0 ignored.**

| Crate | Count | Coverage |
|---|---|---|
| `pallet-qutxo` | 21 | Conservation, double-spend, Merkle root, 11 validate_unsigned rejections, 3 fee E2E, real ML-DSA-44 signing |
| `pallet-calibre-fees` | 24 | 50/30/20 split, base fee dynamics, block rewards, priority fee, 6 producer-settle cases |
| `pallet-stake` | 10 | Bond, consumer failure, input bound, accumulate, unbond+mint, over-balance, full unbond, isolation |
| `calibre-merkle` | 7 | Path construction, root computation, insertion-order determinism |
| `pallet-template` | 4 | Template baseline |
| `solochain-template-runtime` | 2 | Integrity test, genesis build |

Run: `SKIP_WASM_BUILD=1 cargo test --workspace`

Notable: `pallet-qutxo` uses real ML-DSA-44 keygen via `dilithium-rs` 0.4.
The `real_signed_tx()` helper builds an `AegisThreshold` lock, signs
`(inputs, outputs).encode()` with empty context, and returns the pair. This
un-ignored the conservation test that had been `#[ignore]`'d since the
anti-spam change (`4e6c0ab`).

---

## Decision log

Protocol-level choices, with rationale.

| Decision | Choice | Rationale | Commit |
|---|---|---|---|
| Fee split | 50/30/20 | Balances validator incentive, treasury runway, deflation | `06766f9` |
| Reward split | 70/30 | Validators bear cost; treasury captures slice | `301a45d` |
| Base fee | EIP-1559, 12%, 500-1M | Predictable fees | `8b5e264` |
| Genesis | 777M, 5.3/block, 4y halving, 1B cap | Self-funded viable on EUR 13K | `e2912eb` |
| Payout timing | Once per block | Matches existing cadence | `16da3d6` |
| Unregistered producer | Accrue pending, never burn | Ethereum 0x00, Cosmos precedent | `16da3d6` |
| Author resolution | `FindAuthor` config type | No new pallet, mockable | `16da3d6` |
| Staking model | Burn-and-record | Simplest; upgradeable to time-locked | `a73f95f` |
| Staking / payout | Separate concerns | Orthogonal | `a73f95f` |
| Cardano liquid staking | Future, not built | Needs dual-key redesign | `a73f95f` |

---

## Open debt

Three items. Each small, independent, none blocks functionality.

**1. `TotalIssuance` stale on `execute_utxo_tx`.** The `UtxoConsumer` trait
(used by stake) decrements correctly. The original path doesn't. Fix
candidate: decrement symmetric to `consume_with_witness`, or recompute from
`UtxoSet::iter()` in `on_finalize`.

**2. `AuraFindAuthor` slot-to-author math untested.** Written, compiles,
runs. Can't be tested with the `u64` mock AccountId. Needs a runtime
integration test.

**3. Treasury accounting-only.** `TreasuryAccumulated` is written but never
minted. 30% of fees and 30% of block rewards accumulate into a storage value
that is functionally burned.

---

## Tags

| Tag | Commit | Meaning |
|---|---|---|
| `v0.6.6-7validator` | `faed330` | 7-validator Docker testnet |
| `v0.8.0-fee-market` | `301a45d` | Fee market complete (8.1-8.5) |
| `v0.8.6-payout-routing` | `16da3d6` | Producer payout routing |
| `v0.8.9-phase8-closed` | `ae8a5a1` | Phase 8 closed |

---

## Branches

| Branch | Purpose |
|---|---|
| `main` | Source, pallets, runtime, node, docs, tools |
| `gh-pages` | Public landing page + mempool explorer |

Live site: https://bitcore77-calibre.github.io/calibre-core/

---

## How to update this file

After landing a non-trivial commit:

1. Add a row to the phase table if it introduces a new phase.
2. Add the commit to the history block.
3. If it changes tests, update the test inventory.
4. If it makes a decision, add to the decision log.
5. If it closes a debt item, strike it through.

Keep it honest. If something is partial, say partial.
