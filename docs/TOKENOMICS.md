# Calibre Tokenomics

**Status:** Final. Derived from tools/tokenomics/model.py.
**Last updated:** 2026-09-23.

## The headline

**777M CAL at genesis. 223M CAL emitted over 30+ years. Fixed 1B cap.**

77.7% of supply exists at launch. The remaining 22.3% is distributed
through Bitcoin-style halvings over more than three decades. No tail
emission. The 777M genesis is a deliberate brand signal and funds the
project, team, treasury, and ecosystem for decades.

## Total supply

1,000,000,000 CAL, fixed cap, 18 decimals.

## Emission schedule

Halving every 4 years, 6-second blocks, starting at 5.3 CAL/block.

| Epoch | Years | Rate/block | Yearly emission | Cumulative |
|-------|-------|------------|-----------------|------------|
| 1     | 1-4   | 5.3 CAL    | 27.86M          | 111.43M    |
| 2     | 5-8   | 2.65 CAL   | 13.93M          | 167.14M    |
| 3     | 9-12  | 1.325 CAL  | 6.96M           | 195.00M    |
| 4     | 13-16 | 0.6625 CAL | 3.48M           | 208.93M    |
| 5     | 17-20 | 0.331 CAL  | 1.74M           | 215.89M    |
| 6     | 21-24 | 0.166 CAL  | 0.87M           | 219.37M    |
| 7     | 25-28 | 0.083 CAL  | 0.44M           | 221.11M    |
| 8     | 29-32 | 0.041 CAL  | 0.22M           | 221.99M    |
| 9+    | 33+   | geometric  | -> 0            | -> 222.85M |

**Asymptotic emission:** 222.85M CAL (22.29% of cap).
**Year 1 emission:** 27.86M CAL (2.79% of cap).

Marketing: **"777M genesis. 223M over 30 years."**

## Genesis allocation

**777.15M CAL (77.72% of cap).** Market as 777M.

| Bucket | Share of genesis | Amount | Vesting | Purpose |
|--------|------------------|--------|---------|---------|
| Team & founders | 15% | 116.57M | 4y linear, 1y cliff | Long-term alignment |
| Investors | 15% | 116.57M | 3y linear, 1y cliff | Seed + strategic |
| On-chain treasury | 30% | 233.15M | Unlocked, governance | Dev, ops, grants |
| Ecosystem fund | 15% | 116.57M | Milestone-gated | Partners, integrations |
| Public distribution | 15% | 116.57M | TGE + airdrop | Community, listing |
| Staking bootstrap | 10% | 77.72M | Programmatic (SFDP) | Seed early validators |
| **Total** | 100% | **777.15M** | | |

## Circulating supply

| Year | Cumulative emission | Circulating (approx) | % of cap |
|------|---------------------|----------------------|----------|
| 1    | 27.86M              | ~118M                | 11.8%    |
| 4    | 111.43M             | ~270M                | 27.0%    |
| 5    | 125.36M             | ~298M                | 29.8%    |
| 10   | 181.78M             | ~380M                | 38.0%    |
| 20   | 215.89M             | ~435M                | 43.5%    |
| 32   | 221.99M             | ~500M                | 50.0%    |

Year 1 TGE float: ~11.8%. Full unlock asymptotes to ~1B CAL over 30+
years as team, investor, ecosystem, and staking vesting completes.

## Reward and fee splits

| Source | Producer | Treasury | Burn |
|--------|----------|----------|------|
| Block reward (minted) | 70% | 30% | 0% |
| Transaction fees | 50% | 30% | 20% |
| Priority fee (excess) | 100% | 0% | 0% |

## Dynamic base fee

| Parameter | Value |
|-----------|-------|
| Target block fullness | 50% of max refTime |
| Max change per block | 12% |
| Floor | 500 units |
| Ceiling | 1,000,000 units |
| Genesis seed | 1,000 units |

## Validator economics

Year 1 producer pool: 19.50M CAL (70% of 27.86M).

| N validators | CAL/yr | @ $0.05 | Break-even price |
|--------------|--------|---------|-------------------|
| 7            | 2,785,714 | $139,286 | $0.00151 |
| 21           | 928,571   | $46,429  | $0.00452 |
| 50           | 390,000   | $19,500  | $0.01077 |
| 100          | 195,000   | $9,750   | $0.02154 |

Validators profitable across all tiers at $0.05. Break-even at N=21 is
$0.0045 — 33× below launch price target.

## Governance

Phase 8: root multisig. Phase 9: on-chain governance via pallet-democracy.

**Adjustable by governance:** block reward rate, fee splits, treasury lock, base fee parameters, validator admission.
**Immutable:** 1B total cap, halving schedule, genesis allocation percentages.

## Genesis UTXO seeding

At launch, genesis allocation becomes spendable UTXOs at block 0 via
`pallet_qutxo::GenesisConfig` extension. Flagged for Phase 8.8.

## Verification

`python3 tools/tokenomics/model.py` reproduces every number.

Self-checks:
- genesis + asymptotic == cap (exact)
- sum of 32y emissions <= asymptotic
- circulating never exceeds cap
- deterministic across runs

All checks pass.
