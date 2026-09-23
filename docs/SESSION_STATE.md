## Where we are

- **Commit:** `0b61624` (tag `v0.9.3-epochs`, pushed to origin/main)
- **Phase:** 9.1a, 9.1b, 9.2, 9.3 complete. Phase 8 fully closed.
- **Build:** native + WASM clean. SKIP_WASM_BUILD=1 cargo test --workspace -> 97 passed.
- **Next:** Phase 9.4 (BABE migration). 2-week scope. Not started.

## Phase 9 progress

| Sub-phase | Status | Commit |
|-----------|--------|--------|
| 9.1a — calibre-keygen (init/show/check) | done | 2bafab3 |
| 9.1b — export-chain-spec | done | 1e01be0 |
| 9.2 — pallet-session wired | done | 431679f |
| 9.3 — epoch machinery + candidate registry | done | 0b61624 |
| 9.4 — Aura to BABE migration | next | — |
| 9.5 — stake-weighted leader election | — | — |
| 9.6 — slashing + equivocation | — | — |
| 9.7 — on-chain governance | — | — |
| 9.8 — forkless runtime upgrades | — | — |

## What 9.1-9.3 shipped

- calibre-keygen binary: init/show/check/export-chain-spec
- pallet-session at pallet_index(12), 1-hour sessions
- Candidate registry in pallet-stake (Candidates storage, join/leave, MinValidatorStake=1000 CAL)
- elect_top_n selection, CalibreSessionManager rotates set every 6 sessions (6 hours)

## Open follow-ups

1. Session key registration not tested end-to-end. Add integration test.
2. Elected set can be smaller than MaxActiveValidators. No minimum-set enforcement.
3. Mid-epoch stake changes take effect at next epoch boundary only. Document.
4. No ejection rule — candidate stays elected until leave_candidates. Add in 9.5/9.6.

