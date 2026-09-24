# Local review repairs — 2026-09-24

Status: locally validated; source publication authorized after review.
Not a validator deployment or security audit. Review branch:
`fix/github-review-repairs`, based on BABE commit `ec400e4`.
The original checkout and user files are preserved.

## Changes

- QUTXO now validates every input before consuming any: nonempty/bounded,
  unique IDs, existing outputs, identical owner lock, checked value sum and
  a real ML-DSA signature. Pool admission, direct execution and staking's
  input consumer share that validator.
- Dispatch also enforces output conservation, overflow rejection and the
  minimum fee. Rejected inputs leave UTXOs, issuance, stake and events unchanged
  in the regression scenarios.
- The single-witness wire format is preserved. Combining different owners
  is rejected rather than silently authorized by the first owner.
- Runtime spec version was 101 for these initial repairs; the authorized
  [staking follow-up](STAKING_SIGNATURES.md) advances it to 102.
  Storage layout and transaction encoding are unchanged. No live upgrade or
  Aura-to-BABE state migration is authorized or validated.
- Rust 1.88 explicitly includes Clippy and rustfmt; macOS CI installs components
  for the pinned toolchain. CI build, tests and documentation use the lockfile.
  The existing tested SDK versions are retained; no speculative Git overrides
  or lockfile deletion were applied.
- CALIBRE-owned license text and workspace metadata use Unlicense.
  Third-party notices, including `zk/LICENSE` and the template notice, remain.
- Website source preserves the later published feature sections and design,
  corrects BABE/GRANDPA, slot/finality and mempool/settlement wording, removes
  unsupported performance promises, and adds a [fresh-chain guide](../chain-specs/README.md).
  Legacy Aura raw specs and launch scripts are preserved as historical artifacts.

## Initial repair validation (runtime 101)

Before repair, three newly added mixed-owner regression tests failed because
pool admission, execution and the staking consumer accepted unauthorized
later inputs. After repair:

- PASS: 40 QUTXO tests, including actual staking bond integration.
- PASS: 21 staking tests and 27 fee tests.
- PASS: full native workspace suite, 109 tests; no failures or ignored tests.
- PASS with warnings: standard and all-features all-targets workspace Clippy.
- PASS with a target advisory: native + Wasm runtime build (spec version 101).
  The existing `wasm32-unknown-unknown` target was retained; no cache cleanup
  or target migration was performed.
- PASS: local website viewed at desktop (1280), laptop (1024) and mobile (390)
  widths; no document-level horizontal overflow; local section anchors resolve;
  keyboard focus is visible; no captured console errors.
- NOT RUN: hosted GitHub CI/macOS, Docker rebuild, external audit, and a new
  seven-validator run of runtime 101. The existing epoch report belongs to
  baseline `ec400e4` and is not relabeled as evidence for this repair.

From the repository root:

```bash
SKIP_WASM_BUILD=1 cargo test --locked --workspace
SKIP_WASM_BUILD=1 cargo clippy --all-targets --locked --workspace --quiet
SKIP_WASM_BUILD=1 cargo clippy --all-targets --all-features --locked --workspace --quiet
cargo build --locked -p solochain-template-runtime
```

Local checks reused the existing build cache through `CARGO_TARGET_DIR`.
The existing node executable was not replaced by these native checks.

## Boundaries and next decision

These fixes address the reviewed ownership, CI setup, license and documentation
findings, not every security property of the chain. The subsequently authorized
staking follow-up replaces the unsafe `(inputs, [])` bond payload with a
versioned chain- and beneficiary-bound payload. See
[STAKING_SIGNATURES.md](STAKING_SIGNATURES.md) for compatibility and validation.
The prior test counts above belong to runtime 101. Do not use this prototype
for real funds; no external audit or live upgrade has been performed.

The user subsequently authorized committing and publishing this work to GitHub.
Source and website publication do not deploy the runtime to validators.
No next phase, live-chain migration, signing or broadcast is started by publication.
