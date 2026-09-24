## Current repair and publication status — 2026-09-24

- Baseline: BABE commit `ec400e4`; repairs prepared for GitHub publication.
- Repairs are isolated on `fix/github-review-repairs`; original checkout preserved.
- Every-input ownership validation now covers pool admission, execution and
  staking. Runtime specification version is 102. Bond signing bytes now bind
  the staking domain, chain genesis hash, beneficiary and exact inputs;
  old bond signatures are rejected. Call encoding and storage layout are
  unchanged. This is not an approved deployment or live-chain migration.
- Seven-validator epoch evidence applies to baseline `ec400e4`, not to this
  newly repaired runtime. See [repair record](REVIEW_REPAIRS.md).
- CALIBRE-owned code uses Unlicense. Third-party notices are preserved.
- The user authorized Git commits and GitHub/source-site publication after
  local review. No validator deployment, live-chain migration or next phase is started.
- Local validation: 127 native test executions; default/all-features Clippy
  and native + Wasm build passed with warnings. Hosted CI results are separate.

## Historical checkpoint (superseded)

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
