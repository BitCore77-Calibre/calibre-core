# Calibre Launch Economics

**Status:** Recommended structure for 777M genesis.
**Last updated:** 2026-09-23.

---

## Summary

| Metric | Value |
|--------|-------|
| Total raise | $11.8M |
| Launch price | $0.15/CAL |
| Launch FDV | $150M |
| TGE float | ~53M CAL (5.3%) |
| Runway | 3 years at $11M spend |
| Security verdict | Private-actor attack feasible at launch |

---

## 1. Capital requirement — 3-year runway

| Item | Amount |
|------|--------|
| Team (6 people x 3 yrs @ $180K avg) | $3,240,000 |
| Audits (3 x $500K: crypto + ZK + consensus) | $1,500,000 |
| Legal + regulatory (3 jurisdictions) | $800,000 |
| Infrastructure (3 yrs cloud + tooling) | $600,000 |
| Marketing + BD | $1,500,000 |
| Initial DEX/CEX liquidity | $1,500,000 |
| Buffer (20%) | $1,828,000 |
| **Total 3-year runway** | **$10,968,000** |

**Minimum viable raise: $11M.**

---

## 2. Recommended raise structure

| Round | Tokens | Price | Raise | FDV |
|-------|--------|-------|-------|-----|
| Seed | 30,000,000 | $0.04 | $1,200,000 | $40M |
| Private | 40,000,000 | $0.09 | $3,600,000 | $90M |
| Public | 46,570,000 | $0.15 | $6,985,500 | $150M |
| **Total** | **116,570,000** | | **$11,785,500** | |

All three rounds consume the investor bucket (116.57M CAL, 15% of
genesis).

Runway coverage: 107% of $11M requirement. No treasury sales needed
for 3 years.

---

## 3. Vesting

| Round | Cliff | Vest |
|-------|-------|------|
| Seed | 12 months | 24-month linear |
| Private | 12 months | 24-month linear |
| Public | 0 | 50% at TGE, 50% over 12 months |
| Team | 12 months | 48-month linear |
| Ecosystem | 0 | Milestone-gated |
| Treasury | 0 | Governance-controlled |
| Staking bootstrap | 0 | Programmatic release |

**TGE float: ~53M CAL = 5.3% of cap.**

---

## 4. FDV trajectory and security

| Stage | Target FDV | Attack cost | Verdict |
|-------|------------|-------------|---------|
| Launch | $150M | $15M | Private actor feasible |
| Year 1 | $300M | $30M | Private actor feasible |
| Year 2 | $500M | $50M | Institutional only |
| Year 3 | $800M | $79M | Institutional only |
| Year 5 | $2,000M | $198M | Nation-state only |
| Year 10 | $5,000M | $495M | Nation-state only |

Attack cost = 33% of staked value, at 30% staking ratio = 9.9% of FDV.
Rule: FDV >= 10x TVL for security.

---

## 5. Investor returns

| Round | Entry | @ $0.15 | @ $0.50 | @ $1.00 | @ $5.00 |
|-------|-------|---------|---------|---------|---------|
| Seed | $0.04 | 3.8x | 12.5x | 25x | 125x |
| Private | $0.09 | 1.7x | 5.6x | 11x | 56x |
| Public | $0.15 | 1.0x | 3.3x | 6.7x | 33x |

Seed discount: 73% off launch. Private: 40% off launch.

---

## 6. Exchange strategy

**Phase 1 (TGE):** DEX listing (Uniswap/Curve/PancakeSwap). Initial
liquidity $1.5M. Target $10M/day volume.

**Phase 2 (year 1):** Tier-2 CEXs (Gate, KuCoin, Bitget, MEXC).
Professional market maker. Target $50M/day.

**Phase 3 (year 2):** Tier-1 CEXs (Binance, Coinbase, Kraken). Target
$200M/day.

---

## 7. Risk factors

- **Price discovery.** If CAL launches at $0.08, seed sees 2x, public sees 0.9x.
- **Runway finite.** 3 years at current spend. Treasury sales may be needed after year 3.
- **FDV growth not guaranteed.** $500M year-2 target assumes traction.
- **Exchange timelines.** Tier-1 listings can take 6-18 months.
- **Regulatory risk.** Jurisdiction choices affect listings and investor access.

---

## 8. Recommendation

| Metric | Value |
|--------|-------|
| Total raise | $11.8M |
| Launch FDV | $150M |
| Launch price | $0.15/CAL |
| TGE float | 53M CAL (5.3%) |
| Year 1 unlock | ~170M CAL (17%) |
| Runway | 3 years |
| Security at launch | Private-actor attack feasible |

Deliberately conservative launch FDV. $150M is achievable for a
pre-mainnet L1 with strong technical differentiation (native post-quantum).

---

## Related

- docs/TOKENOMICS.md — supply, emission, allocation
- docs/ECONOMIC_MODEL.md — validator and delegator economics
- docs/QUANTUM_THREAT.md — market timing argument
