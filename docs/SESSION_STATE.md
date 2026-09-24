## Where we are

- **Baseline:** `ec400e4` on `phase9.4-babe` (BABE runtime and node integration).
- **Phase:** 9.4 validation in progress. Earlier 9.1-9.3 work remains the baseline.
- **Verified here:** release node with embedded WASM builds; native workspace tests pass;
  a temporary one-validator BABE node advanced from best block 7 to 10 and
  GRANDPA-finalized block 5 to 7. The runtime election guard has three passing
  tests for complete, undersized, and missing-key candidate sets.
- **Seven-validator smoke:** all seven temporary local nodes connected to six
  peers, advanced from best block 2 to 5 and finalized block 0 to 2, and
  agreed on the finalized hash at block 2. Test identity creation and BABE/
  GRANDPA keystore checks passed with a disposable local identity.
- **Reported by `ec400e4` commit message:** seven local validators agreed
  through finalized block 612 and crossed the first 600-slot BABE boundary.
  This was not reproduced in this validation pass.
- **Next gate:** validator replacement through the six-session election boundary,
  multi-computer operation, and the planned 21-validator/two-week soak.

## Phase 9 progress

| Sub-phase | Status | Commit |
|-----------|--------|--------|
| 9.1a — calibre-keygen (init/show/check) | done | 2bafab3 |
| 9.1b — export-chain-spec | done | 1e01be0 |
| 9.2 — pallet-session wired | done | 431679f |
| 9.3 — epoch machinery + candidate registry | done | 0b61624 |
| 9.4 — Aura to BABE migration | integrated; validation open | ec400e4 |
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
2. The new election guard keeps the current set unless the top seven candidates have
   registered session keys. A real replacement across the election boundary
   remains untested.
3. Mid-epoch stake changes take effect at next epoch boundary only. Document.
4. No ejection rule — candidate stays elected until leave_candidates. Add in 9.5/9.6.
5. `MaxActiveValidators` is 7, so the planned 21-validator soak needs a
   separate configuration and deployment test before 9.4 can be marked done.
6. Existing active validators can still purge their own session keys, which may
   reduce the next queued authority set; this requires a separate liveness fix.
7. In-place upgrade of an existing Aura chain to BABE was not tested. These
   checks used fresh test chain state.
