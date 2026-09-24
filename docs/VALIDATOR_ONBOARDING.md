# Validator Onboarding

**Status:** Testnet tooling; Phase 9.4 BABE integration under validation.
**Tool:** `calibre-keygen` (shipped alongside `solochain-template-node`).

## Overview

`calibre-keygen` generates a validator identity: keys that let a validator
produce blocks, vote in finality, and receive payouts. An identity consists
of four key files and one public metadata file.

Everything is local. Nothing is sent over the network during keygen.
Public parts go into a chain spec; private parts stay on the validator's
machine.

The `//Alice` example below is for an isolated testnet. The current tool passes
`--suri` through process arguments, where other local processes may see it.
Production key provisioning needs a safer input path.

## Prerequisites

1. `solochain-template-node` — build: `cargo build --release -p solochain-template-node`
2. `calibre-keygen` — build: `cargo build --release -p pq-signer`
3. A base path directory for the validator

`calibre-keygen` calls the node binary for BABE, GRANDPA, and node network
key generation. It finds the node via `--node-bin <path>`, the current
directory (`target/release/`), or `PATH`.

## Generating an identity

    calibre-keygen init \
        --base-path /var/lib/calibre \
        --name validator-1 \
        --suri "//Alice" \
        --chain staging

Flags:

| Flag | Required | Purpose |
|------|----------|---------|
| `--base-path` | yes | Where the validator's keys and chain data live |
| `--name` | yes | Short validator name (used for directory naming) |
| `--suri` | yes* | Secret URI for BABE and GRANDPA keys |
| `--chain` | no | Chain alias. Default `staging` |
| `--node-bin` | no | Path to `solochain-template-node` |

*The current tool requires a SURI. Use only disposable test identities with
this command-line flow; `//Alice` is a public test identity.

## Files written

    /var/lib/calibre/validator-1/
    ├── identity.json                                       (public)
    ├── pq.key                                              (SECRET)
    ├── pq.pub                                              (public)
    └── chains/
        └── calibre_staging/
            ├── keystore/
            │   ├── 62616265...    (BABE sr25519 — SECRET)
            │   └── 6772616e...    (GRANDPA ed25519 — SECRET)
            └── network/
                └── secret_ed25519 (libp2p network — SECRET)

If any SECRET file is lost, the validator loses its on-chain identity.
There is no recovery. Back up `pq.key` and the `keystore` directory
offline.

## Inspecting

    calibre-keygen show --base-path /var/lib/calibre --name validator-1

Prints name, version, pq_public_key, pq_lock_hash.

## Verifying

    calibre-keygen check --base-path /var/lib/calibre --name validator-1 --chain staging

Checks all six required files exist. Output:

      [  ok] identity.json
      [  ok] pq.key
      [  ok] pq.pub
      [  ok] babe keystore
      [  ok] gran keystore
      [  ok] node network key

    All checks passed.

Run after every backup restore and before starting the node.

## Starting the node

    solochain-template-node \
        --base-path /var/lib/calibre/validator-1 \
        --chain staging \
        --validator \
        --name validator-1 \
        --port 30333 \
        --rpc-port 9944

The node reads the keystore from the base path on startup.

## Legacy pq-signer commands

`pq-signer` binary name still works with the original four commands
(used by docker-compose for the faucet):

    pq-signer keygen
    pq-signer pubkey
    pq-signer sign <hex>
    pq-signer verify <pk> <msg> <sig>

Both `pq-signer` and `calibre-keygen` ship the same code.

## FAQ

**What is pq_lock_hash?** `blake2_256(pq_public_key)`. Goes into
`QuantumLock::AegisThreshold(...)` for the validator's on-chain UTXO.

**Can two validators share a --name?** No. Name determines the base-path
subdirectory.

**What is --suri?** Secret URI. `//Alice` derives Alice's publicly known
test keys deterministically. The current command-line input path is for
isolated testnet use.

**Where does --chain staging map on disk?** `chains/calibre_staging/`.
CLI alias is `staging`; on-disk chain ID is `calibre_staging`.
`calibre-keygen` handles the mapping.

## Related

- `docs/PHASE9_PLAN.md` — Phase 9.1 context
- `docs/MULTI_VALIDATOR.md` — 7-validator deployment
- `deploy/entrypoint.sh` — container keygen automation
