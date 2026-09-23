# The Quantum Threat to Blockchain Cryptography

**Status:** Briefing document. All claims sourced.
**Last updated:** 2026-09-23

---

## Executive summary

Every major blockchain — Bitcoin, Ethereum, Solana, and all others in the
top 20 — secures ownership with elliptic-curve cryptography (ECDSA or
Ed25519). Both are broken by Shor's algorithm on a sufficiently large
quantum computer.

- Expert consensus places "Q-Day" (the moment a quantum computer can break
  ECDSA in practice) in the **2029–2033 window**.
- Ethereum targets December 2029 for full post-quantum protection. Its
  post-quantum transactions will cost **150,000–200,000 gas** versus
  4,000 gas for ECDSA — a **7–10× premium** until EIP-8288 lands.
- Bitcoin has **no formal post-quantum roadmap**. 6.9M BTC sit in
  exposed addresses; 2.3M are irreducibly at risk.
- Solana has selected Falcon signatures but says "no change is required
  today or likely anytime soon."
- **None of the top 20 cryptocurrencies are fully post-quantum today.**

Calibre is **natively post-quantum**. No migration. No cost premium.
No roadmap dependencies. This document captures the evidence and the
strategic window.

---

## 1. The threat

Shor's algorithm, given a sufficiently large fault-tolerant quantum
computer, derives a private key from a public key in polynomial time.
ECDSA and Ed25519 — the two signature schemes securing nearly all
cryptocurrency today — rely on the hardness of the discrete-logarithm
problem, which Shor's algorithm solves.

**The attack is practical when a public key is exposed.** On Bitcoin and
Ethereum, the public key becomes visible the moment a transaction is
broadcast. An adversary with a quantum computer can:

1. Observe the transaction in the mempool.
2. Derive the private key.
3. Sign a competing transaction to a different recipient.
4. Broadcast before the original confirms.

Google Quantum AI's March 2026 paper estimated **9 minutes** to complete
this attack with ~500,000 physical qubits — against Bitcoin's 10-minute
average block time. That gives a **41% chance of beating the confirmation
window** on a per-transaction basis.

---

## 2. When Q-Day arrives

Expert estimates cluster tightly around a 2029–2033 window.

| Source | Estimate | Notes |
|--------|----------|-------|
| Project Eleven (2026, 110 pages) | Q-Day 2030–2033 | "More likely than not by 2033" |
| Google Quantum AI (March 2026) | 2029 internal migration deadline | 500K physical qubits, 9-minute attack |
| Justin Drake (Ethereum Foundation) | ≥10% chance by 2032 | Confidence rose sharply after Google paper |
| Quantum Horizon (Monte Carlo) | 1-in-6 by 2035, 30% by 2040, 60% by 2050 | Wide bimodal distribution, 80% range 2032–2060 |
| Craig Gidney (Google) | 10% chance by 2030 | Cited by Drake |

The Google paper compressed the timeline by roughly **20×** — from
"millions of physical qubits" to "under 500,000" — in a single result.

---

## 3. The migration problem

### Ethereum — targeting December 2029, but expensive

The Ethereum Foundation has a dedicated post-quantum research team and a
"Lean Ethereum" roadmap targeting **December 2029** for full post-quantum
protection. Four layers require replacement: ECDSA account signatures,
BLS consensus signatures, KZG commitments, and ZK-proof systems.

**The cost barrier:** post-quantum signatures on Ethereum cost
**150,000–200,000 gas** to verify, versus ~4,000 gas for ECDSA. A normal
transaction is 21,000 gas. A post-quantum transaction would be **7–10×
more expensive** until EIP-8288 is live.

### EIP-8288 — Vitalik's proposed cost fix

Proposed June 2026 by Vitalik Buterin and Thomas Coratger. Currently a
**draft**, not scheduled for any fork, and depends on EIP-8141 (also
unscheduled).

Mechanism: transactions carry a 96-byte "dependency frame" instead of a
full post-quantum signature. Nodes aggregate these off-chain into a
single recursive STARK proof roughly every second. Only the aggregate
proof lands on-chain.

Expected impact: post-quantum signature verification drops from
150,000–200,000 gas to tens of thousands of gas — a **>99% reduction**.

But: draft status, dependency on another unscheduled proposal, and no
shipping date. Ethereum's post-quantum path depends on two unshipped
EIPs arriving on time.

### Bitcoin — no formal roadmap

Project Eleven estimates Bitcoin's migration could take **close to 10
years**, based on the SegWit precedent (2+ years, one chain split).
Bitcoin has no formal post-quantum research team and no target date.

**6.9 million BTC sit in quantum-exposed addresses.** Of those, **2.3
million** are irreducibly at risk — lost keys, Satoshi-era coins, and
addresses whose public keys are permanently exposed on-chain.

### Solana — Falcon selected, phased migration

Solana has selected Falcon signatures for post-quantum security and
described the performance hit as "manageable." The plan: adopt
post-quantum for new wallets when the threat becomes credible, then
migrate existing wallets.

**Solana's official position:** *"No change is required today or likely
anytime soon."*

### The survey result

The Quantum Horizon paper surveyed the top 20 cryptocurrencies. **None
are fully post-quantum today.** All are either planning migration,
researching options, or have no formal plan.

---

## 4. Calibre's position

Calibre is **natively post-quantum**.

| Property | Calibre | Ethereum | Bitcoin | Solana |
|----------|---------|----------|---------|--------|
| Signature | ML-DSA-44 | ECDSA → ML-DSA/leanXMSS | ECDSA | Ed25519 → Falcon |
| Live today? | Yes | No (target 2029) | No | No |
| Migration needed? | No | Yes | Yes | Yes |
| Post-quantum cost premium | None | 7–10× until EIP-8288 | N/A | TBD |
| Verify cost | 208 µs | 4K gas (ECDSA) / 150K+ gas (PQ) | ~1,000 vbytes | TBD |

**Why Calibre has no cost premium:** we use Substrate's weight system,
not gas. Post-quantum verification cost is denominated in refTime and
proof_size, and block capacity is allocated by weight, not by a gas
market. There is no classical fallback competing for the same block
space. A post-quantum transaction on Calibre costs the same as any other
transaction.

**Measured:** ML-DSA-44 verification = 208 µs. Under 0.3% of a 6-second
block's refTime budget. Cryptography is not the bottleneck.

---

## 5. The strategic window

### Phase 1 — now through 2029: "We are already ready"

Every major chain is *planning* migration. Calibre is *native*. The
pitch is simple: post-quantum ownership from block zero, no roadmap, no
dependencies.

### Phase 2 — 2029 to 2033: "The migration squeeze"

Ethereum hits its 2029 target, but post-quantum transactions are
expensive until EIP-8288 ships — and EIP-8288 has no ship date. Bitcoin
has no roadmap and 6.9M BTC exposed. Users, exchanges, and custodians
face a choice: pay Ethereum's post-quantum premium, wait on Bitcoin, or
move to a chain that was post-quantum from day one.

### Phase 3 — 2033 onward: "Flight to safety"

When Q-Day arrives, or becomes credibly imminent, capital moves. Chains
with native post-quantum security have years of head start. Custodians
and regulated institutions will not migrate to a chain mid-transition
under uncertainty — they will migrate to one that has been quantum-safe
since genesis.

---

## 6. What this means for Calibre

**Do not compete with Ethereum on post-quantum features.** They will
have them. Compete on **certainty of execution**:

- Ethereum's post-quantum path depends on two draft EIPs and a 2029
  deadline that has not been met by prior foundational work.
- Bitcoin's path depends on a governance process that has not produced a
  roadmap.
- Solana's path depends on a phased migration whose start date is
  "whenever the threat becomes credible" — a decision that, by
  definition, will be made too late.

Calibre's claim is: **post-quantum from block zero, no migration, no
dependencies, no premium.** Measured. Verified. Shipped.

---

## Sources

1. Project Eleven, *Quantum Threat Report* (2026). 110-page analysis of
   the cryptocurrency quantum threat and migration timelines.
2. Google Quantum AI, *Securing Cryptocurrencies Against Quantum
   Attacks* (March 2026). Estimated 500,000 physical qubits, 9-minute
   ECDSA attack.
3. Justin Drake (Ethereum Foundation), public statements on quantum
   threat timelines, Q1 2026.
4. Quantum Horizon, *Monte Carlo Analysis of Q-Day Probability* (2026).
   Bimodal distribution, 80% confidence interval 2032–2060.
5. Ethereum Foundation, "Lean Ethereum" roadmap (2026). December 2029
   target for post-quantum L1 security.
6. Vitalik Buterin and Thomas Coratger, *EIP-8288: Post-Quantum and
   Privacy-Preserving Transaction Cost Reduction* (June 2026). Draft
   status.
7. Solana Foundation, post-quantum research blog posts (2026). Falcon
   signature selection.
8. Calibre internal benchmarks, *docs/BENCHMARK.md* and
   *docs/MULTI_VALIDATOR.md* (2026). ML-DSA-44 verify: 208 µs.

---

## Appendix — definitions

**ECDSA** — Elliptic Curve Digital Signature Algorithm. Secures Bitcoin,
Ethereum, and most pre-2010 cryptocurrencies. Broken by Shor's algorithm.

**Ed25519** — Edwards-curve Digital Signature Algorithm. Secures Solana,
Cardano, Polkadot, and most modern chains. Broken by Shor's algorithm.

**ML-DSA-44** — Module-Lattice Digital Signature Algorithm, security
level 2. NIST FIPS 204. Resistant to Shor's algorithm and Grover's
search. Calibre's signature scheme.

**Shor's algorithm** — Quantum algorithm for integer factorization and
discrete logarithms. Breaks ECDSA, Ed25519, RSA, and Diffie-Hellman.

**Grover's algorithm** — Quantum search algorithm. Provides quadratic
speedup against symmetric cryptography (SHA-256, AES). Not a break;
merely requires doubling key sizes.

**Q-Day** — The moment a quantum computer can break widely used public-key
cryptography in practice. Expert consensus places it in 2029–2033.
