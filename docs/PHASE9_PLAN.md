# Phase 9 — Decentralization

**Status:** Plan. Not started.
**Baseline:** commit 81e0fff, tag v0.8.10-debt-closed, 78 tests green.
**Target:** permissioned validator set with on-chain rotation, stake-weighted election, real keygen, and governance.

## What Phase 9 is

Phase 8 made the economic layer real. Phase 9 removes the static genesis-
configured validator set. After Phase 9, validators can be added, removed,
and rotated by protocol rules. The chain stops being a single-operator
testnet and becomes a network.

This is the largest phase. Eight sub-phases, 10-12 weeks.

## Sub-phase map

| # | Sub-phase | Effort | Depends on |
|---|-----------|--------|------------|
| 9.1 | Real keygen + validator onboarding | 2-3 days | — |
| 9.2 | Session pallet + key rotation | 1 week | 9.1 |
| 9.3 | Epoch machinery | 1 week | 9.2 |
| 9.4 | Aura to BABE migration | 2 weeks | 9.3 |
| 9.5 | Stake-weighted leader election | 2 weeks | 9.4 |
| 9.6 | Slashing + equivocation | 2 weeks | 9.5 |
| 9.7 | On-chain governance | 2 weeks | 9.3 |
| 9.8 | Forkless runtime upgrades | 3 days | 9.7 |

## 9.1 — Real keygen + validator onboarding (first)

Problem: all 7 staging validators use sp_keyring test identities. No
external operator can join without a known-weak key.

Solution: new calibre-keygen subcommand that generates real ML-DSA
validator identities.

Deliverables:
- calibre-keygen init — generate validator identity (ML-DSA-44 + Aura
  sr25519 + GRANDPA ed25519), write to base-path.
- calibre-keygen show — print pubkeys for chain-spec inclusion.
- calibre-keygen export-chain-spec --validators <list> — merge N
  validator pubkeys into fresh genesis config.
- docs/VALIDATOR_ONBOARDING.md — step-by-step operator guide.
- Retire sp_keyring from production path.

Why first: every other sub-phase assumes distinct real keys. Building on
sp_keyring means rebuilding when we switch. Build once, on real keys.

## 9.2 — Session pallet + key rotation

Add pallet-session. Each validator registers session keys (Aura +
GRANDPA) separate from account keys. Session keys rotate every N blocks.

Numbers: session length 1 hour (600 blocks at 6s). Rotation every session.

Deliverables:
- pallet-session wired into runtime.
- calibre-node session-key subcommand.
- Runtime NextAuthorities populated at session boundaries.
- Tests: key rotation, session rollover, misbehaving validator ejection.

## 9.3 — Epoch machinery

Epochs (eras) are longer than sessions. Validator set changes happen
only at epoch boundaries.

Numbers: epoch length 6 hours (6 sessions). Unbonding period 7 days
(28 epochs).

Deliverables:
- Staking-style era boundary logic.
- Reward distribution at epoch end.
- Slashing window: exposure for UnbondingDuration.

## 9.4 — Aura to BABE migration

Why: Aura is round-robin. Every validator gets a slot regardless of
stake. That caps the validator set at ~12 (the O(n^2) inflection).
BABE is slot-lottery weighted by stake. Scales to 100+.

What changes:
- Remove pallet-aura, add pallet-babe.
- Wire epoch-change hooks to session manager.
- Update service.rs: BABE block authoring.
- Update FindAuthor: BABE author resolution.

What stays: GRANDPA, pallet-session, fee market, stake, treasury.

Risks: BABE allows forks (multiple VRF winners per slot). GRANDPA
resolves, but mempool and gossip need to handle fork conditions Aura
never exercised. 2-week testnet soak required.

## 9.5 — Stake-weighted leader election

Connect stake ledger to consensus. Babe uses stake weight for slot
probability.

Deliverables:
- pallet-staking wired to pallet-babe epoch trigger.
- Validator election: top-N by stake active, others candidates.
- 21-validator testnet, weighted block production verified.

## 9.6 — Slashing + equivocation

Slashing conditions:
- GRANDPA equivocation (double vote): 100% self-stake
- BABE equivocation (two blocks in one slot): 100% self-stake
- Unresponsiveness (offline 1 epoch): small, gradual

Deliverables:
- pallet-grandpa equivocation reporting wired.
- pallet-babe equivocation reporting wired.
- Reporter bounty (10% of slashed amount).
- Offense storage + era pruning.
- Testnet: deliberate slashing triggered, stake reduction verified.

## 9.7 — On-chain governance

Replace root multisig with token-holder voting.

Deliverables:
- pallet-democracy for referenda.
- pallet-collective for technical council.
- pallet-treasury wired to TreasuryAccumulated + TreasuryLock.
- Governance over: fee params, validator admission, treasury spend,
  runtime upgrade scheduling.

Transition: root retains emergency powers 6 months post-launch. Then
governance takes over. Then root disabled.

## 9.8 — Forkless runtime upgrades

Forkless upgrades via set_code. Two testnet upgrades completed before
mainnet.

Deliverables:
- frame-system set_code enabled (currently disabled).
- Signed upgrade via governance.
- try-runtime pre-upgrade validation.
- Two upgrades completed on testnet.

## Dependency graph

9.1 (keygen) -> 9.2 (sessions) -> 9.3 (epochs)
                                     |
                      +--------------+--------------+
                      |                             |
                 9.4 (BABE)                    9.7 (governance)
                      |                             |
                 9.5 (stake-weighted)          9.8 (upgrades)
                      |
                 9.6 (slashing)

## Ordering

Ship 9.1 first, in isolation. Only sub-phase with no runtime changes.
Unblocks testnet participation immediately. Every operator onboarded
becomes a future validator with a known-good identity.

After 9.1: 9.2 + 9.3 in one push. Then 9.4 (the big one). Then 9.5-9.6.
Governance (9.7) runs parallel to 9.4-9.5.

## Not in Phase 9

- Cardano-style liquid staking (future redesign)
- MEV / block building market
- Cross-chain bridges
- ZK private transfers
- Regulated-entity compliance tooling

## Testnet strategy

| Phase | Testnet |
|-------|---------|
| 9.1 | 7 validators, real keys, public RPC |
| 9.2 | 7 validators with session rotation |
| 9.3 | 14 validators, 6-hour epochs |
| 9.4 | 21 validators, BABE, 2-week soak |
| 9.5 | 21 validators, stake-weighted |
| 9.6 | 21 validators, slashing triggered |
| 9.7 | 21 validators, governance live |
| 9.8 | Two runtime upgrades completed |

## Success criteria

1. Real keys. No sp_keyring in production path.
2. Rotating validator set. Add/remove without chain restart.
3. BABE in production. Aura gone.
4. Stake-weighted consensus. 21 validators, weighted.
5. Slashing proven. Double-signer loses stake.
6. Governance live. Treasury spend via token vote.
7. Runtime upgrades work. Two forkless upgrades.
8. Public testnet stable. 30 days continuous, no stalls >60s.

## Related

- docs/BOOTSTRAP_PLAN.md
- docs/SESSION_STATE.md
- docs/CALIBRE_MAP.md
- docs/ECONOMIC_MODEL.md
