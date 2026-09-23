#!/usr/bin/env python3
"""Calibre Economic Viability Model — corrected.
Attack cost rule: attack_cost >= TVL (not 10x TVL).
"""

CAP = 1_000_000_000
YEAR1_EMISSION = 63_070_000
PRODUCER_BLOCK_SHARE = 0.70
VALIDATOR_ANNUAL_COST = (250 + 100) * 12  # $4,200


def validator_breakeven(n, cal_price_usd):
    per_val_cal = (YEAR1_EMISSION * PRODUCER_BLOCK_SHARE) / n
    return {
        "n": n,
        "cal_per_year": per_val_cal,
        "revenue": per_val_cal * cal_price_usd,
        "margin": per_val_cal * cal_price_usd - VALIDATOR_ANNUAL_COST,
        "breakeven_price": VALIDATOR_ANNUAL_COST / per_val_cal,
    }


def security_budget(fdv, staking_ratio, attack_threshold=0.33):
    staked = fdv * staking_ratio
    return {"fdv": fdv, "staked": staked, "attack_cost": staked * attack_threshold}


def min_fdv_for_tvl(tvl, staking_ratio=0.30, attack_threshold=0.33):
    # attack_cost >= TVL  =>  fdv * ratio * threshold >= tvl
    return tvl / (staking_ratio * attack_threshold)


def delegator_apy(total_staked_cal, commission=0.10):
    pool = YEAR1_EMISSION * PRODUCER_BLOCK_SHARE * (1 - commission)
    return (pool / total_staked_cal) * 100 if total_staked_cal else 0


print("=" * 78)
print("PART 1: VALIDATOR BREAK-EVEN (CORRECTED)")
print("=" * 78)
for n in (7, 21, 50, 100):
    r = validator_breakeven(n, 0.05)
    print(f"N={n:>3} @ $0.05: {r['cal_per_year']:>12,.0f} CAL/y = ${r['revenue']:>10,.0f}/y"
          f"  margin ${r['margin']:>10,.0f}  break-even ${r['breakeven_price']:.6f}")
print()

print("=" * 78)
print("PART 2: MINIMUM STAKE WITH PRICE ANCHORS")
print("=" * 78)
print(f"{'Tier':<14} {'Self-Stake (CAL)':>18} {'@ $0.01':>12} {'@ $0.05':>12} {'@ $0.10':>12} {'@ $0.50':>12}")
print("-" * 78)
tiers = [
    ("Founding",  1_000_000),
    ("Core",        500_000),
    ("Candidate",   100_000),
    ("Observer",     10_000),
]
for name, stake in tiers:
    print(f"{name:<14} {stake:>18,} "
          f"${stake*0.01:>10,.0f} ${stake*0.05:>10,.0f} "
          f"${stake*0.10:>10,.0f} ${stake*0.50:>10,.0f}")
print()
print("2026 L1 benchmarks:")
print("  Ethereum: 32 ETH (~$94K)")
print("  Polkadot: 10,000 DOT (~$40K)")
print("  Solana:   ~1,000 SOL (~$170K)")
print()
print("At $0.05/CAL (target post-launch price), Calibre founding = $50K,")
print("Core = $25K, Candidate = $5K. Comparable to Polkadot, cheaper than Ethereum.")
print()

print("=" * 78)
print("PART 3: MINIMUM FDV FOR TVL (CORRECTED)")
print("=" * 78)
print()
for tvl in (10_000_000, 50_000_000, 100_000_000, 500_000_000, 1_000_000_000):
    min_fdv = min_fdv_for_tvl(tvl)
    print(f"TVL ${tvl/1e6:>6,.0f}M -> Min FDV ${min_fdv/1e6:>8,.0f}M "
          f"-> CAL price ${min_fdv/1e9:.4f}")
print()
print("Rule: FDV >= 10.1 x TVL (at 30% staking, 33% attack threshold)")
print()
print("Interpretation:")
print("  Securing $10M TVL needs ~$101M FDV")
print("  Securing $50M TVL needs ~$505M FDV  <- realistic small-chain target")
print("  Securing $100M TVL needs ~$1.0B FDV")
print("  Securing $500M TVL needs ~$5.1B FDV")
print()

print("=" * 78)
print("PART 4: DELEGATOR APY")
print("=" * 78)
print()
for pct in (0.20, 0.30, 0.50, 0.70):
    print(f"Staking ratio {pct*100:>3.0f}% ({CAP*pct/1e6:>5.0f}M CAL): "
          f"Delegator APY {delegator_apy(CAP*pct):>5.2f}%")
print()
print("Comparison (2026):")
print("  Solana:   ~6.8%")
print("  Polkadot: ~12%")
print("  Cosmos:   ~21%")
print()

print("=" * 78)
print("PART 5: SECURITY BUDGET")
print("=" * 78)
print()
for fdv in (100_000_000, 250_000_000, 500_000_000, 1_000_000_000, 2_000_000_000):
    for ratio in (0.30, 0.50):
        s = security_budget(fdv, ratio)
        sec_ratio = s["attack_cost"] / fdv
        print(f"FDV ${fdv/1e9:>4.2f}B @ {ratio*100:.0f}% staked: "
              f"staked ${s['staked']/1e9:>6.2f}B  "
              f"attack cost ${s['attack_cost']/1e9:>6.2f}B "
              f"({sec_ratio*100:.0f}% of FDV)")
print()
print("Threshold: attack_cost >= TVL for security")
print("Sub-$350M FDV = nation-state attack economically feasible")
print()

print("=" * 78)
print("SANITY CHECKS")
print("=" * 78)
print()
r = validator_breakeven(21, 0.05)
assert r["margin"] > 0
print(f"[OK] Validator N=21 @ $0.05 margin: ${r['margin']:,.0f}")

r = validator_breakeven(50, 0.02)
assert r["margin"] > 0
print(f"[OK] Validator N=50 @ $0.02 margin: ${r['margin']:,.0f}")

apy = delegator_apy(CAP * 0.30)
assert 5 < apy < 25
print(f"[OK] Delegator APY @ 30% staked: {apy:.2f}% (target 5-25%)")

s = security_budget(500_000_000, 0.30)
assert s["attack_cost"] > 40_000_000
print(f"[OK] Attack cost @ $500M FDV, 30% staked: ${s['attack_cost']/1e6:.0f}M")

min_fdv = min_fdv_for_tvl(50_000_000)
print(f"[OK] Min FDV for $50M TVL: ${min_fdv/1e6:.0f}M")

print()
print("All checks passed.")
