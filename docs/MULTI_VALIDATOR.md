# Calibre Multi-Validator Test — Docker Localhost

**Date:** 2026-09-22
**Commit:** faed3309
**Status:** Verified

## Purpose

Prove that Calibre consensus (Aura block production + GRANDPA finality)
works with a real multi-authority validator set, and that finality
survives realistic network latency. Prior to this test only single-
validator (`--dev`) and 2-authority (`local`) configurations had been
exercised.

## Topology

- 7 validators, one Docker container each
- All on a single Docker bridge network (`calibre-net`)
- Host: single Linux workstation
- Node image: `calibre-node:staging` (247 MB, Ubuntu 24.04 base)
- Resource limits per validator: 1 CPU, 1 GB RAM

### Authority set

| Container   | Authority |
|-------------|-----------|
| calibre-v1  | Alice (bootnode) |
| calibre-v2  | Bob       |
| calibre-v3  | Charlie   |
| calibre-v4  | Dave      |
| calibre-v5  | Eve       |
| calibre-v6  | Ferdie    |
| calibre-v7  | One       |

Keys are `sp_keyring` test identities. NOT for production.

### Chain parameters

| Parameter | Value |
|-----------|-------|
| Chain spec | `staging` |
| Aura slot | 6 s |
| Block weight cap | 2 s refTime, u64::MAX proof_size |
| Block byte cap | 20 MB (15 MB normal share) |
| GRANDPA quorum | 5 of 7 |
| Sessions | not enabled |

## Results

### Baseline (localhost, no latency injection)

| Metric | Value |
|--------|-------|
| Peers per validator | 6 (full mesh) |
| Block interval | ~6 s |
| Finality lag | 2 blocks (12 s) |
| Containers healthy | 7/7 |
| CPU per validator | ~1.1% |
| Memory per validator | ~46 MB |

### 100 ms delay / 10 ms jitter (200 ms effective RTT)

| Metric | Value |
|--------|-------|
| Peers per validator | 6 |
| Block interval | ~6 s |
| Finality lag | 2-3 blocks |
| Finality advancing | yes |

### 500 ms delay / 50 ms jitter (1 s effective RTT)

| Metric | Value |
|--------|-------|
| Peers per validator | 6 |
| Block interval | ~6 s |
| Finality lag | 2 blocks |
| Finality advancing | yes |
| Slot skips | none |

1 second RTT exceeds any physically possible internet link (NYC-Sydney
is ~220 ms, Mumbai-Sao Paulo ~330 ms). We tested at 3x the practical
worst case with no degradation.

## Findings

1. Multi-authority consensus works. 7 authorities, full mesh, GRANDPA
   engaged at 5/7 threshold, no forks, no stalls.

2. Finality is latency-tolerant to at least 1 s RTT. Breaking point is
   above 1 s. GRANDPA round timeout (3 slots = 18 s at 6 s blocks) is
   generous relative to gossip latency at this scale.

3. Validator hardware requirements are trivial at current load: ~1.1%
   CPU, ~46 MB RAM per node. This excludes execute_utxo_tx cost — the
   real CPU consumer under transaction load.

4. Bootnode determinism is required for reproducible startup. v1's
   libp2p key is seeded from a fixed file; without it the peer ID
   changes on restart and bootnode multiaddrs baked into v2-v7 break.

5. Docker base image must match host glibc. Host binary needs
   glibc 2.39 (Ubuntu 24.04); Debian bookworm's 2.36 fails.

6. `tc netem` inside containers requires CAP_NET_ADMIN. Added to
   compose defaults.

## Reproducing

    cargo build --release -p solochain-template-node
    cp target/release/solochain-template-node deploy/bin/node
    docker compose -f deploy/docker-compose.yml build
    docker compose -f deploy/docker-compose.yml up -d
    # wait ~60s, then:
    docker logs calibre-v1 --tail 20

Latency injection:

    for i in 1 2 3 4 5 6 7; do
      docker exec calibre-v$i tc qdisc add dev eth0 root \
        netem delay 100ms 10ms distribution normal
    done

Tear down:

    docker compose -f deploy/docker-compose.yml down       # keep data
    docker compose -f deploy/docker-compose.yml down -v    # wipe data


## Throughput benchmark (2026-09-22)

5000 ML-DSA-44-signed UTXO spends submitted to v1's RPC via
`calibre-bench burst`, concurrency 40. Preceded by 5000 `sudo_mint`
calls to seed the UTXO set.

### Chain-level result

| Metric | Value |
|--------|-------|
| Peak block capacity | ~1014 txs / 6 s block = **169 TPS** |
| Consecutive full blocks | 4 |
| Finality lag under load | 2 blocks (unchanged from idle) |
| Pool rejections | 0 |
| Inclusion rate | 5000/5000 |

Single-validator baseline was 167 TPS. Multi-validator is 169 TPS.
**The 7-validator network does not degrade throughput.**

### Per-block weight (full blocks)

- refTime: 1.002e12 / 1.5e12 cap = 67% used
- proof_size (bytes): 4.19e6 / 15.7e6 cap = 27% used
- Neither weight nor byte limit saturated

There is a soft cap around ~1014 txs/block that is not the configured
weight or byte limit. Likely a block producer time budget or extrinsic
count constraint in the current SDK version. Flagged for follow-up;
not blocking.

### Resource use

| State | CPU | RAM |
|-------|-----|-----|
| Idle | ~1.1% | ~46 MB |
| Under load | ~0.5-1.0% | ~150 MB |

Memory tripled under load but remained under 200 MB per validator.

## What this does NOT cover

- Cross-host WAN (this was one host with injected latency)
- Real key generation (uses sp_keyring test identities)
- Byzantine behavior / slashing / equivocation
- Authority set rotation (sessions not enabled)

## Next steps

1. Throughput benchmark on the 7-validator set
2. Cross-host WAN split across 3 machines
3. Real keygen tooling (calibre-keygen + chain-spec generator)
4. Sessions + authority rotation

## Artifacts

- deploy/Dockerfile
- deploy/docker-compose.yml
- deploy/entrypoint.sh
- deploy/secrets/v1-network-key (not in git)
- Image: calibre-node:staging (247 MB)
