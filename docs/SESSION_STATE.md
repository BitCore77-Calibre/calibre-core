# Calibre Session State — d7d61fe2

See the SESSION HANDOFF document in the chat history (or the commit messages
since 28498766) for the full picture.

# Calibre Session State — f2e6c4b7

Three commits ahead of origin/main. Push before next session if you want
a backup; nothing depends on it locally.

## Just shipped (this session, f8e9e23d -> f2e6c4b7)

### Anti-spam for execute_utxo_tx  (4e6c0abe)
`validate_unsigned` now does full ML-DSA-44 verification plus bounded-
size, duplicate-input, lock-type, and UTXO-existence pre-checks. Junk
txs are rejected at pool admission, not by the block producer. 9 new
tests in pallets/qutxo/src/tests.rs; 17/17 pallet suite green.

### Honest weights  (f8e9e23d)
New pallets/qutxo/src/weights.rs: WeightInfo trait + hand-derived
constants from measured ML-DSA-44 cost (208us) + RocksDbWeight, scaling
with inputs/outputs. Closes second DoS vector (weight understatement).
Marked for regeneration via frame-benchmarking once the pallet API is
frozen.

### 7-validator staging chain spec  (f2e6c4b7)
- Runtime preset `calibre_staging` with 7 Aura + 7 GRANDPA authorities
  (sp_keyring Alice..Ferdie + One). NOT for production.
- Node chain spec `staging` routed in load_spec.
- Re-export CALIBRE_STAGING_RUNTIME_PRESET from runtime crate root.
- No `--alice`/`--bob` CLI in this SDK — validator startup is two
  `key insert` calls + one `key generate-node-key`.

**Verified (localhost, 7 processes):** full 6-peer mesh, GRANDPA
finality engaged at the 5/7 threshold, best/finalized tracking with
2-block lag at 6s blocks. First proof of multi-authority finality.

## Throughput — status
Batch B (block byte cap 5 MB -> 20 MB) applied and live, but single-node
bench numbers inconclusive: blocks show refTime 12.8% / bytes 27.7% used
yet empty blocks appear while txs pend. Real limiter is upstream of
block weight. Batch C (1s slots) tested and reverted: 167 -> 57 TPS,
per-block fixed overhead (~0.7s) dominates short slots. Keep 6s.
The "99.9% proof_size" claim in the prior handoff was mislabeled —
proof_size is u64::MAX here; the binding limit was block byte length.

## Just shipped (this session, Phase 8.1 -> 8.3)

### 8.1 — pallet-calibre-fees scaffold  (06766f93)
- New pallet `pallets/calibre-fees` with FeeHandler trait, storage
  (TreasuryLock, ProducerLocks, TreasuryAccumulated, BurnCounter),
  extrinsics (set_treasury_lock, register_producer_lock), genesis config.
- Fee split logic: 50% producer / 30% treasury / 20% burn.
- minimum_fee scales with inputs + outputs.
- Wired into runtime as pallet_index(10).
- Design doc: docs/FEE_MARKET.md (seven locked decisions).
- 9 tests green.

### 8.2 — wire FeeHandler into qutxo  (530cfcec)
- FeeHandler trait moved to calibre-primitives (single source of truth).
- pallet-qutxo::Config gains type FeeHandler: FeeHandler<AccountId, Balance>.
- execute_utxo_tx captures implicit fee (inputs - outputs) and calls
  T::FeeHandler::charge_fee — value is no longer silently destroyed.
- Runtime wires CalibreFees.
- 17 qutxo + 9 calibre-fees tests green.

### 8.3 — reject under-priced txs at pool  (875c75a9)
- validate_unsigned computes implicit fee and rejects with
  InvalidTransaction::Payment if below minimum_fee.
- Complements crypto anti-spam: no more zero-fee pool junk.
- 17 qutxo tests still green.

## Open items / known debt

1. qutxo::TotalIssuance storage is stale — never updated by execute_utxo_tx.
   Either wire it or remove it. Phase 8.x.
2. Producer argument to charge_fee is always None. The 50% producer cut is
   computed but not routed. Needs pallet-authorship integration. Phase 8.2b.
3. Bench txs pay zero fee (inputs == outputs). With 8.3 live, next burst
   will fail with pool-rejected. Either bump bench to leave a fee, or zero
   the BaseTxFee/PerInOutFee constants for bench runs.
4. No end-to-end test for the fee rejection path — needs a valid ML-DSA
   witness in the test harness. Same debt as the ignored Phase 5 test.
   Doing the harness un-ignores both.

## Phase 8 remaining

- 8.2b — producer routing via pallet-authorship
- 8.4  — dynamic base fee (EIP-1559 style)
- 8.5  — block rewards (treasury-funded)
- 8.6  — priority fee (optional producer-routed output)
- 8.7  — stake pallet (bond/unbond only)
- 8.8  — valid-witness test harness (un-ignores Phase 5 test)
- 8.9  — bench update to pay fees

## Where we are overall

Phase 7.5 complete: anti-spam, honest weights, 7-validator Docker
testnet, WAN-equivalent latency tolerance (1s RTT), throughput parity
(169 TPS single vs multi).
Phase 8.1-8.3 complete: fee market scaffold, fee capture in qutxo,
fee-aware pool admission.

## Next (priorities)
1. Multi-validator Docker — containerize 7 validators, docker-compose
   on one bridge, then tc netem latency injection. Deliverable:
   docs/MULTI_VALIDATOR.md with topology, TPS, finality latency, HW spec.
2. WAN split across 3 hosts (after Docker).
3. Real keygen tooling (replaces sp_keyring staging keys; needed for
   any non-test validator set).
4. Full weight benchmarks (frame-benchmarking) once pallet API frozen.
5. 7.6 Android (Ada).

## Just shipped
- **Phase 7.4 unblocked**: on-chain RISC Zero verifier works.
  - crates/calibre-risc0-host uses risc0-zkvm 3.0.6 (same crate as prover).
  - vk conversion [u8; 32] → [u32; 8] BE before verify.
  - Verified end-to-end against a fresh 3.0.6 receipt.
  - Standalone test: /tmp/pallet-verify/target/release/pallet-verify

## Measured
- 167 TPS single-validator, 1006 qutxo/6s block
- Bottleneck: proof_size (99.9% used), NOT crypto (14% ref_time)
- ML-DSA-44 verify: 208 µs/tx, ~$1e-9 electricity
- See docs/BENCHMARK.md

## Just shipped (this session)
- **Anti-spam for `execute_utxo_tx`**: `validate_unsigned` now does full
  ML-DSA-44 verification plus bounded-size, duplicate-input, lock-type,
  and UTXO-existence pre-checks. Junk txs are rejected at pool admission
  instead of forcing expensive verify in the block producer. 9 new tests
  in pallets/qutxo/src/tests.rs (all green); 17/17 in the pallet suite.
- **Batch B (byte cap)**: `RuntimeBlockLength` 5 MB -> 20 MB
  (normal share = 15 MB). Applied, compiled, live. Bench numbers
  inconclusive at single-node scale — real limiter is upstream of
  block weight (blocks show refTime 12.8% / bytes 27.7% used, yet
  empty blocks appear while txs pend). Logged as a caveat in
  docs/BENCHMARK.md.
- **Batch C (slot time)**: 6 s -> 1 s tested and REVERTED. 167 TPS
  became 57 TPS — per-block fixed overhead (~0.7 s) dominates at 1 s
  slots. Reverted to 6000 ms. Keep 6 s until overhead is attacked.

## Next (priorities)
1. Batch B: proof_size 5→15 MB. Expected ~480 TPS.
2. Batch C: slot 6s→1s. Combined with B, expected ~2880 TPS.
3. Multi-validator Docker: 7 vals on 3 Linux machines, staged LAN → WAN.
4. Anti-spam for execute_utxo_tx (validate_unsigned doesn't verify sigs).
5. Session key management.
6. 7.6 Android (Ada).

## Working commands and rules
See the SESSION HANDOFF message in the previous chat.
