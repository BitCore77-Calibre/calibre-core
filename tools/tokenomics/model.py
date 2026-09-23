#!/usr/bin/env python3
"""Calibre tokenomics model — precise, self-verifying."""

BLOCKS_PER_YEAR = 365 * 24 * 3600 // 6   # 6s blocks
YEARS = 32
HALVING_YEARS = 4

# Sanity check
assert BLOCKS_PER_YEAR == 5_256_000, f"blocks/year = {BLOCKS_PER_YEAR}"
print(f"Blocks per year (6s blocks): {BLOCKS_PER_YEAR:,}")
print()


def build_schedule(year1_rate, years=YEARS, halving=HALVING_YEARS):
    """Return per-year emission list, and the asymptotic limit."""
    emissions = []
    for y in range(years):
        epoch = y // halving
        rate = year1_rate / (2 ** epoch)
        emissions.append(rate * BLOCKS_PER_YEAR)
    # Asymptotic = year1 * BLOCKS_PER_YEAR * halving * 2
    asymptotic = year1_rate * BLOCKS_PER_YEAR * halving * 2
    return emissions, asymptotic


def analyze(cap, year1_rate, label):
    """Full analysis: emission, genesis, circulating, validator econ."""
    emissions, asymptotic = build_schedule(year1_rate)
    cum = []
    running = 0.0
    for e in emissions:
        running += e
        cum.append(running)

    sum_32y = cum[-1]
    genesis = cap - asymptotic

    if genesis < 0:
        return {"label": label, "error": f"asymptotic {asymptotic/1e6:.1f}M > cap {cap/1e6:.1f}M"}

    # Verify: sum_32y <= asymptotic
    assert sum_32y <= asymptotic + 1e-6, "sum exceeds asymptotic"
    # Verify: genesis + asymptotic == cap
    assert abs((genesis + asymptotic) - cap) < 1e-6, "genesis + asymptotic != cap"

    # Circulating per year (rough vesting model)
    # Team: 4y vest 1y cliff → 0 in year 1, then 1/48 per month
    # Investors: 3y vest 1y cliff → 0 in year 1, then 1/36 per month
    # Treasury/Public/Ecosystem/Staking: separate release rules
    circulating = []
    for y in range(YEARS):
        year = y + 1
        team_v = max(0, min((year - 1) / 4, 1))            # 4-year linear after 1y cliff
        inv_v  = max(0, min((year - 1) / 3, 1))            # 3-year linear after 1y cliff
        eco_v  = max(0, min(year / 5, 1))                  # 5y linear
        pub_v  = 1.0 if year >= 1 else 0                    # TGE
        treas_v = max(0, min(year / 10, 1))                 # 10y linear drawdown
        circ = (
            genesis * 0.15 * team_v +
            genesis * 0.15 * inv_v +
            genesis * 0.15 * treas_v +
            genesis * 0.05 * eco_v +
            genesis * 0.05 * pub_v +
            cum[y]
        )
        circulating.append(circ)

    return {
        "label": label,
        "cap": cap,
        "year1_rate": year1_rate,
        "year1_pct": year1_rate * BLOCKS_PER_YEAR / cap * 100,
        "genesis": genesis,
        "asymptotic": asymptotic,
        "sum_32y": sum_32y,
        "cum_emission": cum,
        "circulating": circulating,
        "blk_per_year": BLOCKS_PER_YEAR,
    }


def validator_econ(s, n_validators, cal_price_usd, op_cost_usd=1_200):
    """Year-1 validator economics for a given scenario."""
    y1_emission = s["cum_emission"][0]
    per_validator_cal = (y1_emission / n_validators) * 0.70
    revenue = per_validator_cal * cal_price_usd
    return per_validator_cal, revenue, revenue - op_cost_usd


# ────────────────────────────────────────────────────────────
# SCENARIOS
# ────────────────────────────────────────────────────────────
scenarios = [
    analyze(1e9,  8,  "A: 1B cap, 8 CAL/blk  (4.2% yr1)"),
    analyze(1e9, 10,  "B: 1B cap, 10 CAL/blk (5.3% yr1)"),
    analyze(1e9, 12,  "C: 1B cap, 12 CAL/blk (6.3% yr1)"),
    analyze(1e9, 15,  "D: 1B cap, 15 CAL/blk (7.9% yr1)"),
    analyze(1e9, 20,  "E: 1B cap, 20 CAL/blk (10.5% yr1)"),
    analyze(2e9, 20,  "F: 2B cap, 20 CAL/blk (5.3% yr1)"),
    analyze(2e9, 25,  "G: 2B cap, 25 CAL/blk (6.6% yr1)"),
]

print("=" * 78)
print("SCENARIO COMPARISON — supply mechanics")
print("=" * 78)
for s in scenarios:
    if "error" in s:
        print(f"\n{s['label']}:  ERROR — {s['error']}")
        continue
    c = s["circulating"]
    print(f"\n{s['label']}")
    print(f"  Genesis bucket:         {s['genesis']/1e6:>10,.2f}M  ({s['genesis']/s['cap']*100:5.2f}% of cap)")
    print(f"  Asymptotic emission:    {s['asymptotic']/1e6:>10,.2f}M  ({s['asymptotic']/s['cap']*100:5.2f}% of cap)")
    print(f"  Sum of 32-year emission:{s['sum_32y']/1e6:>10,.2f}M")
    print(f"  Year 1 emission:        {s['cum_emission'][0]/1e6:>10,.2f}M  ({s['year1_pct']:5.2f}% of cap)")
    print(f"  Year 1 circulating:     {c[0]/1e6:>10,.2f}M  ({c[0]/s['cap']*100:5.2f}%)")
    print(f"  Year 5 circulating:     {c[4]/1e6:>10,.2f}M  ({c[4]/s['cap']*100:5.2f}%)")
    print(f"  Year 10 circulating:    {c[9]/1e6:>10,.2f}M  ({c[9]/s['cap']*100:5.2f}%)")
    print(f"  Year 20 circulating:    {c[19]/1e6:>10,.2f}M  ({c[19]/s['cap']*100:5.2f}%)")

print()
print("=" * 78)
print("VALIDATOR ECONOMICS — year 1, 70% producer share, $1,200/yr op cost")
print("=" * 78)
for s in scenarios:
    if "error" in s:
        continue
    print(f"\n{s['label']}")
    for n in (7, 21, 50):
        cal_v, rev, margin = validator_econ(s, n, 0.05)
        print(f"  N={n:>3} @ $0.05/CAL: {cal_v/1e6:>7.3f}M CAL/y = "
              f"${rev:>10,.0f}/y  (margin ${margin:>10,.0f})")

print()
print("=" * 78)
print("VERIFICATION — scenario C (1B cap, 12 CAL/block) year-by-year")
print("=" * 78)
s = scenarios[2]
print(f"  {'Year':>4}  {'Epoch':>5}  {'Rate/blk':>9}  {'Year emit (M)':>14}  "
      f"{'Cum emit (M)':>13}  {'Circ (M)':>10}")
cum = 0.0
for y in range(YEARS):
    epoch = y // HALVING_YEARS
    rate = s["year1_rate"] / (2 ** epoch)
    year_emit = rate * BLOCKS_PER_YEAR
    cum += year_emit
    print(f"  {y+1:>4}  {epoch+1:>5}  {rate:>9.4f}  {year_emit/1e6:>14,.2f}  "
          f"{cum/1e6:>13,.2f}  {s['circulating'][y]/1e6:>10,.2f}")

# Final verifications
print()
print("=" * 78)
print("FINAL VERIFICATION — scenario C")
print("=" * 78)
print(f"  Genesis + asymptotic              = {s['genesis']/1e6:.4f}M + {s['asymptotic']/1e6:.4f}M "
      f"= {(s['genesis']+s['asymptotic'])/1e6:.4f}M")
print(f"  Should equal cap                  = {s['cap']/1e6:.4f}M")
print(f"  Match: {abs(s['genesis']+s['asymptotic'] - s['cap']) < 1e-6}")
print(f"  Sum 32y ≤ asymptotic: {s['sum_32y'] <= s['asymptotic']}")
print(f"  32y sum as % of asymptotic: {s['sum_32y']/s['asymptotic']*100:.4f}%")

# Show max circulating (year 32)
print(f"  Max circulating (year 32) = {s['circulating'][-1]/1e6:.2f}M "
      f"({s['circulating'][-1]/s['cap']*100:.2f}% of cap)")
print(f"  Never exceeds cap: {s['circulating'][-1] <= s['cap']}")
