# Calibre Session State

## Where we are

- **Commit:** `16da3d67` (local, not pushed — `origin/main` at `4e65e6f0`)
- **Phase:** 8.6 complete. Next: 8.7 (stake pallet).
- **Build:** native + WASM clean. `SKIP_WASM_BUILD=1 cargo test --workspace` -> 54 passed, 1 ignored (pre-existing Phase 5 regression).

## Locked this session

### Economic (unchanged from prior sessions)
- 777M genesis, 5.3 CAL/block, 4y halving, 1B cap.
- Self-funded bootstrap (EUR 13.5K, no VC raise pre-product).
- Fee split 50/30/20, reward split 70/30, priority fee 100% producer.
- Dynamic base fee EIP-1559-style (12% change, 500-1M range).

### Phase 8.6 design (Q1/Q2)
- **Q1 = once per block.** Producer payouts settle in `on_finalize`, batched, not per-tx. Matches existing `adjust_base_fee` / `distribute_block_reward` cadence. One UTXO per producer per block.
- **Q2(a) = leave pending, never burn.** Unregistered producers accrue in `ProducerPending[account]`; mint happens on a later block once `register_producer_lock` is called. Follows Ethereum 0x00-credential + Cosmos outstanding-commission precedent.
- Author resolution via a `FindAuthor` config type (`AuraFindAuthor` at runtime), **not** a `pallet-authorship` dependency. If other pallets later want `Authorship::author()`, add pallet-authorship on top of the same struct — 4-line follow-up.

### Phase 8.7 design (Q3/Q4)
- **Q3 = option (i), burn-and-record.** Bond consumes UTXOs into an internal `Stake[account]` ledger; unbond mints a fresh UTXO. Simpler, achievable in scope, upgradeable to time-lock (option ii) later.
- **Q4 = separate.** Staking does not auto-register a producer lock. `register_producer_lock` stays `ensure_root`. Staking and fee-payout-routing are different concerns.

### Future: Cardano-style liquid staking
Cardano does NOT use burn-and-record. It uses a **dual-key model**: payment key (spends UTXOs) + staking key (delegates). Stake weight is counted from the delegation registry, not from locked UTXOs. ADA never moves and is never locked — you can spend while delegated, no slashing, no lock-up.

This is the eventual target for **staking liquidity** because stakers keep spend-access to their principal. To build it, we need:
1. A staking key registered per account (separate from the ML-DSA payment lock).
2. A delegation certificate signed by that key.
3. Consensus (Aura author selection, reward weighting) reading stake from the delegation registry, not from UTXO ownership.

That is a multi-session redesign — not in 8.7. Recorded here so the door stays open and (i) is understood as a stepping stone, not the final shape.

## Phase status

| Phase | Description | Status |
|---|---|---|
| 7.x | Anti-spam, weights, 7-validator Docker, TPS parity | done |
| 8.1 | calibre-fees scaffold | done |
| 8.2 | FeeHandler wired into qutxo | done |
| 8.3 | Pool admission rejects under-priced txs | done |
| 8.4 | Dynamic base fee (EIP-1559-style) | done |
| 8.5 | Block rewards | done |
| 8.6 | Producer payout routing | done |
| 8.7 | Stake pallet (bond/unbond) | **next** |
| 8.8 | Valid-witness test harness | todo |
| 8.9 | Bench txs pay fees | todo |

## Open debt

1. **`qutxo::TotalIssuance` is stale** on the `execute_utxo_tx` path — never decremented when inputs are consumed. Staking (8.7) will decrement it on its own path; `execute_utxo_tx` still needs a fix. Phase 8.x.
2. **`AuraFindAuthor` slot -> author math is not unit-tested.** Runtime integration; can't be tested with the `u64` mock AccountId. Needs an integration test in `runtime`.
3. **Treasury remains accounting-only.** `TreasuryAccumulated` is written but never minted. Treasury withdrawal (governance-gated) is a future concern.
4. **`ProducerPending` exit semantics undocumented.** If a producer leaves the validator set with pending, it stays until they author again. Deliberate (no sweep, no governance call), but should be written down.
5. **Bench txs pay zero fee.** With 8.3 live, next bench burst will fail pool admission. Fix in 8.9.
6. **No valid-witness test harness.** Same debt as the ignored Phase 5 test; un-ignores two tests. Phase 8.8.
7. **`/tmp/calibre-head` worktree check** confirmed `4e65e6f0` builds WASM clean — the earlier WASM failure was stale artifact from before the qutxo serde fix, not a real regression.

## Next priorities

1. **8.7 stake pallet** — `bond` / `unbond`, `Stake` ledger, `qutxo::consume_utxos_with_witness` helper. ~3-4h.
2. **8.8 valid-witness test harness** — unblocks Phase 5 ignored test and the fee-rejection path E2E.
3. **8.9 bench update** — bench txs must pay fees or 8.3 rejects them.
4. **Push + tag** — session note update and push before next session.

## Working rules

- Stay in `~/calibre-template`.
- Long heredocs get mangled — use `python3 - << 'PYEOF'`.
- `git --no-pager` on every log/show/diff.
- On Rust compile errors: `cargo build 2>&1 | head -60` to see the real cause, not the build-script wrapper.
- `SKIP_WASM_BUILD=1` for fast iteration; full WASM build before commit.
