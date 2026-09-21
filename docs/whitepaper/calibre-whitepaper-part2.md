# Calibre Whitepaper — Part 2: Cryptographic Specification

## 3. Cryptographic Primitives

### 3.1 ML-DSA-44 (FIPS 204)

Calibre's value path uses **ML-DSA-44** (Module-Lattice Digital Signature
Algorithm, security level 2), standardized by NIST in August 2024 as
FIPS 204.

| Property | Value |
|:---|:---|
| Public key size | 1,312 bytes |
| Signature size | 2,420 bytes |
| Security level | NIST level 2 |
| Underlying hardness | Module-LWE, Module-SIS |
| Quantum resistance | Safe against Shor's and Grover's algorithms |

The implementation is the **dilithium-rs 0.4.1** crate — pure Rust, no_std,
compiling to wasm32-unknown-unknown. No C bindings, no getrandom, no
ambient authority.

### 3.2 Lock derivation

A UTXO's lock is derived from its intended spender's public key:

    lock = blake2b_256(public_key_bytes)

This produces a 32-byte commitment. A spend must satisfy:

    blake2b_256(witness.pub_keys[0]) == UTXO.lock

This binding is enforced inside verify_aegis_transaction(). A valid
ML-DSA-44 signature over the payload is necessary but not sufficient —
the signing key must also be the key that the UTXO was locked to.

### 3.3 Transaction hash and UTXO ID

A transaction's hash is blake2b_256(inputs_SCALE || outputs_SCALE) where
the payload is SCALE-encoded as a tuple of input and output vectors.

A UTXO's ID is blake2b_256(tx_hash || u32_le(output_index)). This mirrors
Bitcoin's txid:vout scheme but uses Blake2b for speed and length-extension
resistance.

### 3.4 QR-CKD (Quantum-Resilient Contextual Key Derivation)

*Roadmap.* For future MPC-based custody:

    share_i = PQ-KDF( SHAKE256(E_i || A_i || T_i) )

where:
- E_i — biometric entropy (FaceID / TouchID / device PUF)
- A_i — hardware attestation (Apple AppAttest / Google Play Integrity)
- T_i — time drift (block timestamp, ±5 min window)

These three inputs are hashed and passed through a Kyber-based KDF. The
resulting shares are held across three devices; a 2-of-3 threshold
reconstructs the signing key **only in volatile memory**, milliseconds
before use.

This mechanism is described for completeness but is **not yet implemented**.
The current testnet uses a single ML-DSA-44 key per user.

### 3.5 Device attestation

*Roadmap.* Real A.E.G.I.S. custody requires verifying that the signing
device is genuinely an Apple or Google device, running an unmodified build
of the Calibre wallet. This is done via DCAppAttestService (iOS) or
Play Integrity API (Android), whose root certificates would be held by the
chain's attestation oracle.

In the current testnet, hardware attestation is a no-op.

---

## 4. Verification Paths

### 4.1 Standard spend

A standard spend of a UTXO locked with AegisThreshold(hash) requires:

1. A witness containing the ML-DSA-44 public key and signature
2. A transaction whose SCALE-encoded payload is the signed message
3. That the transaction's inputs reference existing, unspent UTXOs
4. That the signer's public key hash equals the UTXO's lock

### 4.2 Forgery resistance

An attacker who observes a UTXO (and can see its lock hash) cannot spend
it without the private key. Two attack surfaces are tested:

- **Signature forgery** — a random 2,420-byte signature is submitted. It
  is accepted by the mempool (which does not verify cryptography) and
  rejected at block execution.
- **Key substitution** — a valid ML-DSA-44 signature from a different
  keypair is submitted. It is rejected at Gate 3 (lock binding) before
  the signature is even checked.

### 4.3 Replay resistance

After a UTXO is spent and included in a block, its ID is removed from
UtxoSet. Re-submitting the same transaction returns
InvalidTransaction::Stale from the mempool, without consuming block
space or execution time.

### 4.4 Double-spend resistance

Two conflicting transactions submitted simultaneously (same input,
different outputs) are detected at the mempool's conflict graph. One
is accepted; the other is rejected with Priority is too low.

### 4.5 Conservation of mass

Every spend satisfies:

    Σ input_values ≥ Σ output_values

The difference is the implicit transaction fee. No spend can create value.
This invariant is enforced in execute_utxo_tx before any state mutation.

---

*Continue to Part 3: Fast-Path Mempool and Interoperability.*
