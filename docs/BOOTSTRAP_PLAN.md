# Calibre Bootstrap Plan

**Status:** Recommended strategy. Self-funded start, phased to decentralization.
**Last updated:** 2026-09-23.
**Depends on:** docs/TOKENOMICS.md, docs/ECONOMIC_MODEL.md, docs/LAUNCH_ECONOMICS.md.

---

## Executive summary

Calibre starts **self-funded**. No investor raise. No pre-mine sale.
No token distribution event.

The founder funds 7 validators on Hetzner for **€32/month** (~€500 for
the first 6 months). Testnet runs until the protocol is proven. Mainnet
launches with 14 validators (7 founder + 7 invited operators). Treasury
sales fund everything after month 6. An investor raise is optional, and
if taken, happens at **$500M+ FDV with a working product** — not $150M
pre-product.

**Total founder capital required before treasury self-funds: €13,000.**
Spread over 12 months. Fully recoverable if the project fails (only
sunk cost is the audit).

---

## Why this approach

We evaluated three launch strategies:

| Strategy | Raise amount | FDV at launch | Founder ownership | Speed |
|----------|--------------|---------------|-------------------|-------|
| **A. Pre-product VC raise** | $11.8M | $150M | ~60% after dilution | 6 months to raise, then build |
| **B. Token pre-sale (Cardano model)** | $30M+ | $200M | ~55% after sale | 16 months of tranches |
| **C. Self-funded bootstrap** | €13K | No launch price | 100% + treasury | 6 months to testnet, 12 to mainnet |

**C wins on three axes:**
1. **Ownership.** Founder keeps team allocation (15%) plus treasury (30%)
   plus ecosystem fund (15%). Effective control: 60% of genesis.
2. **Optionality.** No obligation to raise. Can wait for mainnet traction
   and raise at $500M–$1B FDV if desired.
3. **Correctness.** Build first, price later. No investor pressure to
   ship before ready.

**C loses on:** speed. Raising capital accelerates hiring and auditing.
But Calibre does not need capital to ship — it needs time.

---

## The cheapest viable validator network

### Cost comparison (7 validators, 12 months)

| Provider | Instance | 7× Monthly | 7× Yearly | Notes |
|----------|----------|------------|-----------|-------|
| **Hetzner CX22** | 2 vCPU / 4 GB / 40 GB | €31.57 | **€378.84** | Cheapest viable |
| Vultr | 1 vCPU / 1 GB / 25 GB | $35.00 | $420.00 | Similar, slightly more |
| LightNode | 1 vCPU / 2 GB / 50 GB | $53.97 | $647.64 | Global locations |
| Oracle Cloud Free | ARM Ampere | $0 | $0 | Availability lottery |
| Solana (reference) | 12 vCPU / 256 GB | $5,000+ | $60,000+ | 158× more expensive |

**Calibre's cost advantage is structural.** Post-quantum signatures are
larger (2.4 KB vs 64 B) but verify in 208 µs — under 0.3% of block
budget. The 750× cost gap versus Solana reflects a different consensus
model (Aura+GRANDPA vs Turbine+Gulf Stream), not a hardware compromise.

### Minimum spec per validator

Our measured Docker test showed **1.1% CPU and 46 MB RAM** at idle,
150 MB under load. The CX22 has 20× headroom on CPU and 26× on RAM.

| Resource | Needed | CX22 provides | Headroom |
|----------|--------|---------------|----------|
| CPU | 0.5 vCPU | 2 vCPU | 4× |
| RAM | 512 MB | 4 GB | 8× |
| Disk | 10 GB | 40 GB | 4× |
| Network | 100 Mbps | 1 Gbps | 10× |

The network is not just adequate — it's over-provisioned for the
protocol's current load.

---

## Phase 0: Self-funded development (Month 0 → Month 6)

### Budget

| Item | Monthly | One-time | Notes |
|------|---------|----------|-------|
| 7 Hetzner CX22 VPS | €32 | — | 3 locations × 2–3 validators |
| Domain (calibre.io) | €1 | — | Namecheap, 3-year upfront |
| Cloudflare (free tier) | €0 | — | DDoS protection, DNS |
| GitHub (public repos) | €0 | — | Already free |
| Prometheus + Grafana Cloud | €0 | — | Free tier covers 7 nodes |
| Email (Protonmail) | €5 | — | Business address |
| **Founder's time** | €0 | — | Unpaid — this is the investment |
| **Total monthly** | **€38/month** | | |
| **6-month total** | **€228** | | |

### What is built

1. **Devnet** — 7 validators on 3 Hetzner locations (1 in FSN, 3 in NBG,
   3 in HEL). Same Docker setup as the local 7-validator test, but
   geographically distributed.
2. **Public testnet** — RPC endpoint live, faucet deployed, explorer
   stub. Anyone can connect.
3. **Documentation** — `docs/VALIDATOR_GUIDE.md` with step-by-step
   onboarding. FAQ. Discord for support.
4. **Community** — Discord server, Twitter/X account, GitHub public
   repos. Target: 100 Discord members, 50 testnet participants by
   month 6.
5. **Phase 8 completion** — priority fee, stake pallet, genesis UTXO
   seeding, valid-witness test harness.

### Why this works

Calibre at month 6 has:
- Working consensus (Aura + GRANDPA, 7 validators, distributed)
- Live post-quantum signatures on every transaction
- Real ZK light-client verification
- Docker deployment reproducible by anyone
- Documentation for external operators

**This is investor-ready material.** It's also operational — the network
runs, blocks finalize, nothing needs to be fixed.

### Founder cost: €228

Six months of infrastructure for the price of a decent dinner.

---

## Phase 1: Permissioned growth (Month 6 → Month 12)

### Budget

| Item | Amount | Timing |
|------|--------|--------|
| Hetzner × 7 (continued) | €194 | Ongoing |
| Legal entity (Estonia e-Residency) | €2,500 | Month 6 |
| Audit #1 (community-grade, fixed scope) | €10,000 | Month 8–10 |
| Security review tooling | €500 | Month 8 |
| Domain + email (continued) | €36 | Ongoing |
| **Total Phase 1** | **~€13,230** | |

**Cumulative founder investment by end of Phase 1: ~€13,500.**

### Legal structure

- **Jurisdiction:** Estonia e-Residency → Estonian OÜ.
  - Minimal bureaucracy, EU jurisdiction, crypto-friendly.
  - Setup: €2,500 (legal + registered agent).
  - Annual: ~€1,000 (accounting + filing).
  - Alternatives: Switzerland (foundation, expensive), Cayman (offshore,
    opaque), Singapore (good but slow setup).

- **Entity purpose:** holding the treasury multisig, issuing grants,
  signing exchange listings, hiring contractors.

- **Not required:** full compliance stack, MSB licence, or regulated
  entity status. Those come later if exchange listings demand them.

### Audit scope

**Audit #1 covers:**
- ML-DSA-44 signature implementation (`calibre-aegis-crypto`)
- UTXO spend semantics (`pallet-qutxo`)
- RISC Zero guest program correctness
- Consensus integration (Aura + GRANDPA)

**Audit #1 does NOT cover:**
- Fee market (Phase 8.4+)
- Stake pallet (Phase 8.7)
- Governance (Phase 9)

**Firms we've considered:**
- Trail of Bits — top-tier, $75K–$150K per engagement
- Zellic — strong ZK focus, $30K–$60K
- Oak Security — mid-tier, $20K–$50K
- Community-grade (independent researchers via Immunefi) — $5K–$15K

We budget €10,000 for a **community-grade audit**: a fixed-scope review
by 1–2 well-known researchers, producing a public report. Not a
Tier-1 audit. That runs later, post-raise or post-treasury-liquidity.

### Operators invited in Phase 1

We invite **7 external operators** to run validators alongside ours.
Selection criteria:

1. **Technical competence** — running production infrastructure today
2. **Reputation** — someone whose name we can attach to the validator set
3. **Geographic diversity** — at least 3 continents represented
4. **Alignment** — believe in post-quantum security, willing to wait for
   rewards to materialize

**Compensation in Phase 1:** staking bootstrap program. Treasury
delegates 5M CAL to each operator over 4 years. At $0.05/CAL, that's
$250K per operator over the vest period. Enough to justify
participation, not enough to be a security concern.

**Primary targets:**
- Figment (staking-as-a-service)
- Helius (Solana RPC provider)
- Chorus One (multi-chain validator)
- Kiln (institutional staking)
- Individual operators from Polkadot and Cosmos communities

### Mainnet launch at month 12

At mainnet, Calibre has:
- 14 validators (7 founder, 7 invited)
- Live token (CAL, DEX tradeable)
- Working treasury (5% annual release mechanism live)
- First use cases (testnet DEX, wallets, light clients)
- Public presence (website, docs, Discord, Twitter)

**Token at launch is worth $0 until listed.** Listing happens at mainnet
launch via DEX (Uniswap, Curve, or a native Calibre DEX if ready).

---

## Phase 2: Treasury self-funding (Month 12 → Month 24)

### Treasury release mechanism

The treasury lock contract allows **5% annual release** starting at
mainnet. This is the operational budget for everything after launch.

| CAL price | Treasury value | 5% annual release | Monthly budget |
|-----------|----------------|-------------------|----------------|
| $0.001 | $233K | $11.6K | $970 |
| $0.01 | $2.33M | $116.5K | $9,700 |
| $0.05 | $11.66M | $582.8K | $48,600 |
| $0.10 | $23.31M | $1.16M | $97,100 |
| $0.15 | $34.97M | $1.75M | $145,800 |

**At any CAL price above $0.001, the treasury funds operations.** At
$0.01, you can hire 3–4 contractors. At $0.05, you can hire 5 full-time
engineers.

### What Phase 2 does

| Month | Activity | Cost source |
|-------|----------|-------------|
| 12–15 | Mainnet stabilization, first DEX liquidity | Treasury release |
| 15–18 | Grow from 14 → 21 validators | Treasury + staking bootstrap |
| 18–21 | Tier-2 CEX listings (Gate, KuCoin, MEXC) | Treasury |
| 21–24 | Ecosystem grants, DEX TVL growth | Ecosystem fund |
| 24 | First DAO governance vote | Root → council transition |

**Founder salary:** starts at month 12. $5,000/month, paid from
treasury. Modest, but sufficient.

**Team growth:** 1 → 3 people (founder + 2 contractors) by month 18.
3 → 5 by month 24.

### Why no VC raise at this point

If CAL trades at $0.05 with a $500M FDV, the project is functional and
self-sustaining. External capital would be **acceleration money**, not
**survival money**. We don't need it.

The optional raise can happen at month 24+ at a much higher valuation
with a working product. Or never. Either path works.

---

## Phase 3: Optional investor raise (Month 24+)

**Decision point:** if we want to accelerate 3× — faster hiring, more
audits, more listings — raise capital. If we're happy at 1× growth,
stay self-funded.

### If we raise:

| Metric | Value at raise |
|--------|----------------|
| FDV | $500M–$1B |
| Raise amount | $10M–$30M |
| Dilution | 1–3% of supply (from investor + ecosystem buckets) |
| Valuation multiple vs. self-funding | 3.3–6.6× higher |

Compare:
- Raise at month 0 at $150M FDV: 8% dilution for $12M
- Raise at month 24 at $750M FDV: 1.6% dilution for $12M

**Waiting 24 months costs nothing and preserves 6.4% of supply.**
That's 64M CAL = $3.2M at $0.05. Real money.

### If we don't raise:

- Slower growth (2–3 year roadmap instead of 1–2 year)
- Full founder control
- Treasury funds everything
- No investor obligations or board seats

**My recommendation:** don't raise. Calibre is not a capital-intensive
business. It's a time-intensive one. Treasury sales fund time.

---

## Phase 4: Progressive decentralization (Month 24+)

| Month | Validators | Mechanism |
|-------|-----------|-----------|
| 0–6 | 7 (founder) | Genesis preset |
| 6–12 | 7 (founder) | Still genesis |
| 12–15 | 14 (founder + 7 invited) | Genesis preset upgrade |
| 15–18 | 21 (invited) | Genesis preset + governance approval |
| 18–24 | 21 → 50 | On-chain validator admission (gated) |
| 24–36 | 50 → 100 | Permissionless-with-stake |
| 36+ | 100+ | Full gated permissionless |

**Governance transition:** root multisig at Phase 1, council at Phase 2,
on-chain democracy at Phase 3.

**Founder validator share:** starts at 100% (7/7), shrinks to 33% (7/21)
by month 18, 14% (7/50) by month 24, 7% (7/100) by month 36.

---

## The full timeline and budget

| Period | Founder capital | Treasury available | Milestones |
|--------|-----------------|---------------------|------------|
| Month 0–6 | €228 | €0 | Testnet, 7 validators, docs |
| Month 6–12 | €13,230 | €0 | Mainnet, 14 validators, audit #1 |
| Month 12–18 | €0 | $50K–$600K (dependent on CAL price) | 21 validators, CEX listing |
| Month 18–24 | €0 | $100K–$1.2M | 50 validators, DEX launch |
| Month 24+ | €0 | Sustainable | Optional raise, progressive decentralization |

**Total founder capital: €13,500.** Everything after month 12 is funded
by the treasury from CAL sales.

**Max downside: €13,500** (if project fails completely by month 12). If
it succeeds to month 24, the treasury at $0.01/CAL has generated ~$230K
of liquid value — a 17× return on founder capital, before any token
appreciation.

---

## Comparisons to other launches

| Project | Launch model | Raise | Mainnet validators | Founder cost |
|---------|--------------|-------|--------------------|--------------|
| **Cardano (2017)** | 5-tranche ICO, 16 months | $62M | Federated (1 node) | Minimal |
| **Solana (2020)** | Seed + foundation | $25M+ | 92 validators | $1M+/mo foundation |
| **Polkadot (2020)** | ICO + Web3 Foundation | $143M | ~200 validators | High |
| **Avail (2024)** | VC raise + airdrop | $43M | 45–50 validators | Medium |
| **Monad (2025)** | VC raise | $225M | 200 validators | High |
| **Calibre (2026)** | **Self-funded bootstrap** | **€13.5K** | **14 → 21 → 50** | **€13.5K** |

**Calibre is in a distinct category.** We are the only one of these that
starts with no external capital. The reason is structural: our operating
costs are 100–1000× lower than peers. We can afford to wait for the
right moment instead of selling tokens into a weak market.

---

## Risk analysis

### Risks we avoid by self-funding

- **Investor pressure to list early** — no investors, no pressure
- **Vesting cliffs forcing sell pressure** — treasury releases slowly
- **Product-market-fit misalignment** — build what users want, not VCs
- **Down-round risk** — no prior round to mark down
- **Board seats and control loss** — founder retains full control

### Risks we accept

- **Slower hiring** — cannot compete for senior talent on salary
- **Single-founder risk** — one person, one set of skills
- **Capital-constrained audit** — one community-grade audit only, no
  Tier-1 firm until Phase 2
- **Longer timeline** — 12 months to mainnet, not 6
- **Founder burnout risk** — long timeline, unpaid, single-handed

### Mitigations

| Risk | Mitigation |
|------|-----------|
| Slower hiring | Hire contractors by project, not employee |
| Single-founder | Public documentation lets others contribute |
| Audit constraints | Immunefi bug bounty from month 8 |
| Timeline | Ship testnet in 3 months, mainnet in 12 |
| Burnout | Build in public, celebrate milestones, don't grind alone |

---

## Decision checkpoints

Three moments where we reassess:

**Month 6 — Testnet milestone.** If testnet is stable and external
operators are interested, proceed. If not, extend testnet by 3 months.

**Month 12 — Mainnet milestone.** If mainnet is live and CAL has any
positive market price, proceed with Phase 2. If CAL is unsold or the
treasury cannot release value, consider a small seed round ($500K–$2M)
at a modest valuation.

**Month 24 — Investor decision.** If treasury funds operations
comfortably, never raise. If the project needs acceleration, raise at
$500M+ FDV. If the project is dying, sunset gracefully and return
remaining treasury to holders.

---

## What this looks like from the outside

**Month 0:** Anonymous founder, GitHub repos, unpublished draft docs.

**Month 6:** Public testnet. 7 validators. Docs. Discord. 100 people.
No token. No website beyond a landing page.

**Month 12:** Mainnet. CAL token live. 14 validators. First DEX listing.
1,000 token holders. Public audit report.

**Month 18:** 21 validators. Tier-2 CEX listing. First DEX built on
Calibre. 10,000 holders. Real usage.

**Month 24:** 50 validators. Tier-1 CEX talks. Multiple DApps. 50,000
holders. Recognized post-quantum L1.

**Month 36:** 100 validators. Fully decentralized. Standard reference for
post-quantum blockchain.

**The path is not fast. It is durable.**

---

## Related documents

- `docs/TOKENOMICS.md` — supply, emission, allocation
- `docs/ECONOMIC_MODEL.md` — validator and delegator math
- `docs/LAUNCH_ECONOMICS.md` — VC raise alternative (not chosen)
- `docs/MULTI_VALIDATOR.md` — distributed deployment reference
- `tools/tokenomics/model.py` — supply model
- `tools/tokenomics/economics.py` — economics model
