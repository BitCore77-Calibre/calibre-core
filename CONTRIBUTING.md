# Contributing to Calibre Protocol

Thank you for helping build a quantum-resistant monetary network.

## Getting started

1. Fork the repository and clone your fork.
2. Install Docker (any recent version).
3. Install Rust 1.85 (see rust-toolchain.toml).
4. Read README.md and docs/whitepaper/README.md.
5. Run `docker compose up` to see the full stack running locally.

## Development workflow

    cargo test --workspace                                  # all tests
    cargo build --release -p solochain-template-runtime     # WASM runtime
    docker compose build node                               # rebuild image

## Before opening a pull request

- Run `cargo fmt --all` and `cargo clippy --all -- -D warnings`.
- Ensure `cargo test --workspace` passes.
- Add a test if you fix a bug or add a feature.
- Keep commits focused; one logical change per commit.

## Commit message style

- `phase N.X: <short summary>` for milestone work
- `fix: <what was broken>` for bug fixes
- `docs: <what changed>` for documentation
- `chore: <what and why>` for tooling and housekeeping

## Reporting bugs

For non-security bugs, open a GitHub issue. For security issues, see SECURITY.md.

## Code of conduct

Be precise, be kind, be honest. Attack ideas, not people.