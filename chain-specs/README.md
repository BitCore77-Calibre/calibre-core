# Local BABE launch guide

This guide applies to the BABE/runtime-102 source, not older Aura revisions.
Research/test use only; no real funds.

## Build and run

Run from the repository root. Install Rustup and the native prerequisites
listed in the matching `.github/actions/*-dependencies/action.yml` first
(including a C/C++ toolchain, CMake and Protobuf). The repository pins
Rust 1.88.0, Rust sources and the Wasm target. The initial build may take
considerably longer than a few minutes.

```bash
cargo build --locked --release -p solochain-template-node
./target/release/solochain-template-node --dev --tmp --no-telemetry
```

The default RPC endpoint is `http://127.0.0.1:9944`. Keep it loopback-only.
The development identity is public and insecure; never use it for real value.
Stop with Ctrl-C. Temporary chain state is disposable.

In another terminal, confirm that best and finalized block numbers advance:

```bash
curl -sS -H 'Content-Type: application/json' --data '{"jsonrpc":"2.0","id":1,"method":"chain_getHeader","params":[]}' http://127.0.0.1:9944
curl -sS -H 'Content-Type: application/json' --data '{"jsonrpc":"2.0","id":2,"method":"chain_getFinalizedHead","params":[]}' http://127.0.0.1:9944
```

## Generate a matching specification

Use the same freshly built binary as the validators. This command prints a
new development specification; save it to a new file if needed, without
overwriting an existing network configuration:

```bash
./target/release/solochain-template-node build-spec --chain dev --raw --disable-default-bootnode
```

Committed older `calibre-testnet*.json` files are historical Aura artifacts.
Do not launch this BABE runtime with them, reuse an Aura database, or run the
legacy multi-validator cleanup scripts. The Aura-to-BABE changes were
validated on a fresh local chain, not as an in-place network migration.

Six seconds is the BABE slot duration, not a finality guarantee. Mempool
acceptance does not settle a transaction; inclusion and GRANDPA finalization
must be checked separately. See [repair scope](../docs/REVIEW_REPAIRS.md).
