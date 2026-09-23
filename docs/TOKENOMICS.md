# Calibre Tokenomics

**Status:** Final. Derived from tools/tokenomics/model.py.
**Last updated:** 2026-09-23.

## Total supply

1,000,000,000 CAL, fixed cap, 18 decimals.

## Emission schedule

Halving every 4 years, 6-second blocks, starting at 12 CAL/block.

| Epoch | Years | Rate/block | Yearly emission | Cumulative |
|-------|-------|------------|-----------------|------------|
| 1     | 1-4   | 12 CAL     | 63.07M          | 252.29M    |
| 2     | 5-8   | 6 CAL      | 31.54M          | 378.43M    |
| 3     | 9-12  | 3 CAL      | 15.77M          | 441.50M    |
| 4     | 13-16 | 1.5 CAL    | 7.88M           | 473.04M    |
| 5     | 17-20 | 0.75 CAL   | 3.94M           | 488.81M    |
| 6     | 21-24 | 0.375 CAL  | 1.97M           | 496.69M    |
| 7     | 25-28 | 0.1875 CAL | 0.99M           | 500.63M    |
| 8     | 29-32 | 0.09375 CAL| 0.49M           | 502.61M    |
| ->    | 33+   | geometric  | -> 0            | -> 504.58M |

Asymptotic emission: 504.58M CAL (50.46% of cap).
Year 1 emission: 63.07M CAL (6.31% of cap).

## Genesis allocation

Genesis bucket = cap - asymptotic emission = 495.42M CAL (49.54%).

Split within the genesis bucket:

| Bucket | Share | Amount | Vesting |
|--------|-------|--------|---------|
| Team & founders | 15% | 74.31M | 4y linear, 1y cliff |
| Investors | 15% | 74.31M | 3y linear, 1y cliff |
| On-chain treasury | 30% | 148.63M | Unlocked, governance |
| Ecosystem fund | 15% | 74.31M | Milestone-gated |
| Public distribution | 15% | 74.31M | TGE + airdrop |
| Staking bootstrap | 10% | 49.54M | Programmatic |

## Circulating supply

| Year | Cumulative emission | Circulating | % of cap |
|------|---------------------|-------------|----------|
| 1    | 63.07M              | 100.23M     | 10.02%   |
| 4    | 252.29M             | 456.65M     | 45.67%   |
| 5    | 283.82M             | 519.15M     | 51.92%   |
| 10   | 409.97M             | 682.45M     | 68.25%   |
| 20   | 488.81M             | 761.29M     | 76.13%   |
| 32   | 502.61M             | 775.09M     | 77.51%   |

Year 1 TGE float: 10.02%.

## Reward and fee splits

Block rewards (minted): 70% producer, 30% treasury, 0% burn.
Transaction fees: 50% producer, 30% treasury, 20% burn.
Priority fees (excess over minimum): 100% producer.

## Dynamic base fee

| Parameter | Value |
|-----------|-------|
| Target block fullness | 50% |
| Max change per block | 12% |
| Floor | 500 |
| Ceiling | 1,000,000 |
| Seed | 1,000 |

## Governance

Phase 8: root multisig. Phase 9: on-chain governance.

Adjustable: block reward rate, fee splits, treasury lock, base fee parameters, validator admission.
Immutable: 1B cap, halving schedule, genesis percentages.

## Genesis UTXO seeding

At launch, genesis allocation becomes spendable UTXOs at block 0 via
pallet_qutxo::GenesisConfig extension. Flagged for Phase 8.8.

## Verification

Run python3 tools/tokenomics/model.py to reproduce. Self-checks:
- genesis + asymptotic == cap (exact)
- sum of 32y emissions <= asymptotic
- circulating never exceeds cap
- deterministic across runs
