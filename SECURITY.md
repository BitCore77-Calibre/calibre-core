# Security Policy

## Reporting a Vulnerability

**Do not open a public GitHub issue for security vulnerabilities.**

Please contact the maintainers privately first. Until a dedicated security email is published, use the repository"s issue tracker to request a private disclosure channel.

Include in your report:

- A description of the issue and its impact
- Steps to reproduce (code, transaction bytes, or a minimal test case)
- The commit hash or release tag affected
- Whether you intend to publish, and if so, when

## Scope

In scope:

- `pallet-qutxo` - UTXO state machine, conflict detector, validation gates
- `crates/aegis-crypto` - ML-DSA-44 verification, QR-CKD
- `runtime` - composed runtime, pallet wiring
- `tools/pq-signer` - ML-DSA-44 CLI
- `tools/faucet` - HTTP faucet
- `Dockerfile` and `docker-compose.yml`

Out of scope:

- Bugs in upstream Substrate or dilithium-rs (report upstream)
- Testnet-only issues that do not affect mainnet safety
- DoS via public RPC (rate limiting is not a security bug)

## Rewards

Calibre Protocol does not currently operate a bug bounty program. Once mainnet is launched, a formal program will be published. Until then, we credit reporters in release notes if they wish.

## Cryptographic Assumptions

Calibre"s security relies on:

- **ML-DSA-44** (FIPS 204) - signature unforgeability under quantum attack
- **Blake2b-256** - collision resistance for UTXO ID and lock hash
- **SHAKE256** - QR-CKD context derivation
- **Substrate consensus** - Aura + GRANDPA for block production and finality

A break in any of these primitives is a critical issue. Please report it.