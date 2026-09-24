## Where we are

- **Baseline:** `b932a70` on `phase9.4-babe` (BABE runtime and node integration).
- **Phase:** 9.4 validation in progress. Earlier 9.1-9.3 work remains the baseline.
- **Prepared security integration:** the QUTXO input-ownership and chain-bound
  staking bond repairs from `main` commit `4991959` are included with BABE in
  runtime spec 103. The combined source has not been built or tested. Earlier
  runtime-101 BABE and runtime-102 security results apply to their original
  binaries, not to this combined runtime.
- **Verified here:** release node with embedded WASM builds; native workspace tests pass;
  a temporary one-validator BABE node advanced from best block 7 to 10 and
  GRANDPA-finalized block 5 to 7. The runtime election guard has three passing
  tests for complete, undersized, and missing-key candidate sets.
- **Seven-validator smoke:** the updated release binary (runtime spec 101)
  ran seven temporary local nodes. All connected to six peers, advanced from
  best block 2 to 5 and finalized block 0 to 2, and agreed on the finalized
  hash at block 2. Test identity creation and BABE/GRANDPA keystore checks
  passed earlier with a disposable local identity.
- **Validator replacement test:** a signed `session.set_keys` call and signed
  candidate registrations with seeded test stake replaced one of seven
  validators after the queued session delay. Runtime tests checked the active
  BABE and GRANDPA authority sets, the missing-key fallback, and the fee author
  mapping. These tests simulate session transitions.
- **Accelerated live handoff:** a separate eight-node local lab (seven active
  validators and one standby) used a three-slot BABE epoch to reach session 6.
  One validator was replaced while the active, BABE, and GRANDPA sets stayed
  at seven. All eight nodes advanced to best block 19, finalized block 17,
  and agreed on its hash after the handoff. This disposable lab forced the
  replacement in its session manager; it did not exercise live stake election
  or the production 600-slot session duration.
- **Liveness guard:** signed `session.purge_keys` calls are filtered for all
  accounts until a validator-aware exit policy exists. `session.set_keys` remains
  available. This behavior began in runtime 101 and is retained in 103.
- **Lab scope:** keep the current seven-validator cap for this validation. The
  planned 21-validator/two-week soak is deferred and has not passed.
- **Prepared next run:** a user-started single-computer, normal-duration
  seven-validator election with one standby, synthetic genesis stake, and
  read-only Observatory evidence (`docs/PHASE9_4_LOCAL_RUN.md`). It has not run.
- **Deferred:** multi-computer operation until the additional computers are
  available. Signed live candidate registration remains untested.

## Phase 9 progress

| Sub-phase | Status | Commit |
|-----------|--------|--------|
| 9.1a — calibre-keygen (init/show/check) | done | 2bafab3 |
| 9.1b — export-chain-spec | done | 1e01be0 |
| 9.2 — pallet-session wired | done | 431679f |
| 9.3 — epoch machinery + candidate registry | done | 0b61624 |
| 9.4 — Aura to BABE migration | integrated; validation open | b932a70 |
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

1. The runtime integration test covers signed key registration, election, and
   simulated activation at session 6. The accelerated local lab covers a forced
   replacement with continued block production and GRANDPA finality. Live
   stake-driven election at the normal session duration remains untested.
2. The election guard keeps the current set unless the top seven candidates have
   registered session keys. The test covers the missing-key fallback.
3. Mid-epoch stake changes take effect at next epoch boundary only. Document.
4. No ejection rule — candidate stays elected until leave_candidates. Add in 9.5/9.6.
5. `MaxActiveValidators` remains 7 by user choice for the current private lab.
   The planned 21-validator soak needs a separate configuration and deployment
   test before that plan criterion can be marked done.
6. `session.purge_keys` is filtered for all signed accounts as a temporary
   liveness guard. A future exit policy must allow safe cleanup after a
   validator leaves the active and queued sets.
7. In-place upgrade of an existing Aura chain to BABE was not tested. These
   checks used fresh test chain state.
