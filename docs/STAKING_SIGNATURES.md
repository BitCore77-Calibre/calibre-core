# Staking bond signatures — runtime 102 evidence, runtime 103 integration

## Decision and compatibility

Bond authorization now signs:

```text
SCALE(
    domain: byte slice = "CALIBRE::stake::bond::v1",
    genesis_hash: runtime Hash,
    beneficiary: runtime AccountId,
    inputs: ordered TransactionInput sequence
)
```

The 24-byte domain is SCALE length-prefixed (first byte 0x60). On the
CALIBRE runtime, the genesis hash and AccountId32 are each 32 raw bytes.
Input count is compact-encoded; each input is tx_hash[32] followed by its
u32 little-endian output_index. There is no output list in this payload.
The ML-DSA context remains empty; domain separation is in the signed message.

One shared helper in `calibre-primitives` defines these bytes for both the
runtime and offline encoder. `Stake::bond` obtains the beneficiary from
`ensure_signed(origin)`; QUTXO obtains the genesis hash from System block 0.
Neither context field is accepted from witness metadata.

The call arguments and AegisWitness container are unchanged, but signatures
over the old `(inputs, [])` payload are intentionally invalid. **Regenerate
every outstanding bond witness. There is no legacy fallback.** Normal QUTXO
transfer signatures are unchanged. The repair was first checked in runtime
spec 102 and is now included in the untested combined BABE runtime 103. No
storage layout migration or live network upgrade was performed.

Binding just the account would still permit reuse across chains or actions.
The explicit domain and genesis hash avoid those ambiguities. Consumed input
availability rejects repeated bonding; networks sharing identical genesis
and state are not distinguished by this domain.

## Offline encoder

From the repository root, build the local signer:

```bash
cargo build --locked -p pq-signer --bin pq-signer
```

The following public-data fixture is copy-pasteable; it does not authorize a
real bond or access any key:

```bash
./target/debug/pq-signer bond-payload 1111111111111111111111111111111111111111111111111111111111111111 2222222222222222222222222222222222222222222222222222222222222222 '[{"tx_hash":"3333333333333333333333333333333333333333333333333333333333333333","output_index":7}]'
```

It emits canonical payload hex only. For actual local test signing, use the
verified chain's genesis hash, the intended signer's beneficiary account
(raw AccountId32 hex, **not SS58 text**), and the exact unspent inputs.
Confirm those values before using the existing offline `sign <message-hex>`
command. Do not send a private key or seed phrase to a service.

As before, put the resulting signature and matching public key into the
SCALE AegisWitness, then construct `Stake::bond(inputs, witness)` with the
same beneficiary as the outer signed caller. The encoder neither builds the
outer extrinsic nor broadcasts it, and does not prove inputs exist or are
owned by the key. Runtime validation remains authoritative.

## Regression coverage

Runtime-102 validation on 2026-09-24: PASS, 127 native workspace test executions
(five signer tests run under each of the two binary names), zero failures
or ignored tests. Standard and all-features Clippy passed with warnings.
Runtime 102 native + Wasm build passed with the existing Wasm-target advisory.
These results do not validate the combined runtime 103 source.

- Legacy unbound witness: rejected (reproduced as accepted before this repair).
- Recipient substituted: rejected without consumption; original recipient
  can still bond the same inputs afterward.
- Other chain, reordered inputs, mixed owners: rejected.
- Bond witness used for a transfer/burn: rejected in pool and execution.
- Valid bond: exact credit; repeated bond cannot credit twice.
- CLI byte layout matches an independent golden vector and AccountId32.
- Real ML-DSA signature from encoder bytes verifies for the intended runtime
  account and fails for a different account.
- Missing consumer implementation fails closed.

Run from the repository root:

```bash
SKIP_WASM_BUILD=1 cargo test --locked -p pallet-qutxo -p pallet-stake -p calibre-primitives -p pq-signer
SKIP_WASM_BUILD=1 cargo test --locked --workspace
cargo build --locked -p solochain-template-runtime
```

Validation uses generated in-memory test keys, not user keys. Hosted CI,
runtime-102 multi-validator testing, external audit and live deployment are
separate checks. The original one-hour BABE report remains evidence only for
its original binary.
