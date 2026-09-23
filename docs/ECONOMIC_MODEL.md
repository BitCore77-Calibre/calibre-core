# Calibre Economic Model

**Status:** Derived from `tools/tokenomics/economics.py` (deterministic).
**Last updated:** 2026-09-23.

---

## Executive summary

| Metric | Value |
|--------|-------|
| Fixed supply | 1,000,000,000 CAL |
| Year 1 emission | 63,070,000 CAL (6.31% of cap) |
| Halving cadence | Every 4 years |
| Asymptotic emission | 504,580,000 CAL (50.46%) |
| Genesis bucket | 495,420,000 CAL (49.54%) |
| Target FDV (launch → mature) | $500M → $2B |
| Minimum viable FDV | $350M |
| Delegator APY target | 8–15% |
| Validator self-stake (Founding) | 1,000,000 CAL |
| Validator self-stake (Candidate) | 100,000 CAL |
| Delegator minimum | 1,000 CAL |

---

## 1. Validator economics

Annual operating cost assumed: **$4,200/year** ($250/mo cloud node + $100/mo ops).

Block reward: 12 CAL/block year 1, 70% to producer. Producer pool per year: 44.15M CAL.

| N validators | CAL/validator/yr | Revenue @ $0.05 | Margin @ $0.05 | Break-even CAL price |
|--------------|------------------|-----------------|----------------|----------------------|
| 7            | 6,307,000        | $315,350        | $311,150       | $0.000666            |
| 21           | 2,102,333        | $105,117        | $100,917       | $0.001998            |
| 50           | 882,980          | $44,149         | $39,949        | $0.004757            |
| 100          | 441,490          | $22,074         | $17,874        | $0.009513            |

**Interpretation:** even at 100 validators and $0.01/CAL, per-validator revenue is $4,415/year — just above cost. At $0.05, all tiers have healthy margins. The break-even CAL price is 10–100× below any credible post-launch price.

---

## 2. Stake requirements with price anchors

Self-stake amounts (CAL) mapped to USD at various price points:

| Tier | Self-stake (CAL) | @ $0.01 | @ $0.05 | @ $0.10 | @ $0.50 |
|------|------------------|---------|---------|---------|---------|
| Founding | 1,000,000 | $10K | $50K | $100K | $500K |
| Core | 500,000 | $5K | $25K | $50K | $250K |
| Candidate | 100,000 | $1K | $5K | $10K | $50K |
| Observer | 10,000 | $100 | $500 | $1K | $5K |
| Delegator min | 1,000 | $10 | $50 | $100 | $500 |

**2026 benchmark comparison:**

| Chain | Minimum validator stake | USD (2026) |
|-------|-------------------------|------------|
| Ethereum | 32 ETH | ~$94,000 |
| Polkadot | 10,000 DOT | ~$40,000 |
| Solana | ~1,000 SOL | ~$170,000 |
| Monad | 100,000 MON (self) | TBD |
| **Calibre (Founding)** | **1,000,000 CAL** | **$50K @ $0.05** |

Calibre Founding sits between Polkadot and Ethereum. Candidate tier ($5K) is deliberately accessible — small validators can participate without $100K+ capital.

---

## 3. Minimum FDV for economic security

**Rule:** `attack_cost >= TVL`. Attacker must be unable to profit by acquiring 1/3 of staked tokens and reversing transactions.

At 30% staking ratio and 33% attack threshold:

    attack_cost = FDV × 0.30 × 0.33 = FDV × 0.099
    attack_cost >= TVL  ⟹  FDV >= TVL / 0.099 = 10.1 × TVL

| TVL to secure | Minimum FDV | CAL price @ 1B supply |
|---------------|-------------|----------------------|
| $10M          | $101M       | $0.101               |
| $50M          | $505M       | $0.505               |
| $100M         | $1,010M     | $1.01                |
| $500M         | $5,051M     | $5.05                |
| $1,000M       | $10,101M    | $10.10               |

**2026 real-world ratios:**

| Chain | FDV | TVL | Ratio |
|-------|-----|-----|-------|
| Ethereum | $333B | ~$60B | 5.5× |
| Solana | $68B | ~$10B | 6.8× |
| BNB | ~$90B | ~$5B | 18× |

Calibre's 10.1× sits in the middle. Conservative enough to be defensible, not so high that launch FDV becomes unachievable.

---

## 4. Security budget scenarios

Attack cost (cost to acquire 33% of staked tokens at current price):

| FDV | Staking ratio | Staked value | Attack cost | % of FDV |
|-----|---------------|--------------|-------------|----------|
| $100M | 30% | $30M | $10M | 10% |
| $250M | 30% | $75M | $25M | 10% |
| $500M | 30% | $150M | $50M | 10% |
| $1B | 30% | $300M | $99M | 10% |
| $2B | 30% | $600M | $198M | 10% |
| $500M | 50% | $250M | $83M | 16% |
| $1B | 50% | $500M | $165M | 16% |
| $2B | 50% | $1B | $330M | 16% |

**Thresholds:**
- **$100M FDV:** attack cost ~$10M — feasible for a well-funded private group
- **$500M FDV:** attack cost ~$50M — feasible for a large hedge fund, not retail
- **$1B+ FDV:** attack cost $100M+ — outside most private actors' reach
- **$2B+ FDV:** attack cost $200M+ — nation-state territory

**The $350M floor is where private-actor attacks stop being economically rational.** Below that, chain security depends on the assumption that no attacker exists. Above it, security is enforced by economics.

---

## 5. Delegator APY

Annual reward pool (year 1): 44.15M CAL (producer share, net of commission at 10% delegated to delegators).

| Staking ratio | Total staked | Delegator APY |
|---------------|--------------|---------------|
| 20% | 200M CAL | 19.87% |
| 30% | 300M CAL | 13.24% |
| 50% | 500M CAL | 7.95% |
| 70% | 700M CAL | 5.68% |

**2026 comparison:**
- Solana: ~6.8% APY
- Polkadot: ~12% APY
- Cosmos: ~21% APY

Calibre at typical 30–50% staking sits at **8–13% APY** — competitive with mature chains, sustainable long-term. Higher than Ethereum, lower than Cosmos. This is the right range for institutional and retail delegators.

---

## 6. Target metrics at maturity

| Metric | Target | Notes |
|--------|--------|-------|
| FDV at launch | $500M | Below = attack vulnerability |
| FDV at year 2 | $1B | Path to security independence |
| FDV at year 5 | $2B+ | Nation-state-resistant |
| Staking ratio | 30–50% | Higher = more security, lower APY |
| Delegator APY | 8–15% | Competitive with Polkadot |
| Validator count | 21 → 50 → 100 | Empirical sweet spot |
| TVL secured | $50M → $500M | Follows FDV by 10× |

---

## 7. Bootstrap path

**Phase 1 (launch):** 7 founding validators, all self-staked. FDV target $500M. Attack cost $50M. Security provided by the founders' reputational stake more than economic stake.

**Phase 2 (year 1–2):** Expand to 21 validators. Add staking bootstrap program (SFDP-style) — treasury delegates to qualifying external validators. FDV grows toward $1B.

**Phase 3 (year 3–5):** Expand to 50 candidates. Bootstrap program shrinks as external delegation grows. FDV $1–2B. Attack cost $100–200M.

**Phase 4 (year 5+):** 100+ validators possible (requires Phase 9 BABE migration). FDV $2B+. Fully economic security.

---

## 8. What this model does NOT cover

- **Token price prediction.** Price is discovered by the market. This model shows what price is needed for security; not what price will happen.
- **Emission-to-FDV ratio drift.** If FDV rises much faster than emission, year-1 staking APY drops and delegators may need adjustment.
- **Real-world attacker capabilities.** Nation-states, malicious collusion, and coerced operators violate the profit-maximization assumption.
- **Regulatory costs.** KYC/KYB infrastructure, legal entity setup, and compliance are not included in the $4,200 validator cost.
- **Fee revenue.** Year-1 fee revenue is negligible compared to block rewards. Post-year-5, fee revenue may exceed rewards and change the economics.

---

## 9. Verification

Run `python3 tools/tokenomics/economics.py` to reproduce all numbers.

Self-checks enforced:
- Validator margin > 0 at N=21 @ $0.05
- Validator margin > 0 at N=50 @ $0.02
- Delegator APY in [5%, 25%] at 30% staked
- Attack cost at $500M FDV, 30% staked > $40M
- Minimum viable FDV for $50M TVL is $505M
- Deterministic across runs

---

## 10. Related documents

- `docs/TOKENOMICS.md` — supply, emission, allocation
- `docs/FEE_MARKET.md` — fee mechanism, dynamic base fee
- `tools/tokenomics/model.py` — supply model
- `tools/tokenomics/economics.py` — this document's source

---

## Sources

- 2026 L1 benchmark data: Ethereum Foundation, Web3 Foundation, Solana
  Foundation, Cosmos Hub, Monad Labs public documentation.
- Hardware cost assumptions: Hetzner, DigitalOcean, AWS cloud pricing Q2
  2026.
- Attack cost rule: standard PoS economic security framework
  (Buterin, "On Settlement Finality", 2016; Ethereum Foundation research
  blog).
