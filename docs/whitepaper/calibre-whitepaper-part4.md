# Calibre Whitepaper — Part 4: Status, Roadmap, and References

## 8. Current Status

### 8.1 What is implemented (testnet)

| Component | Status |
|:---|:---|
| Q-UTXO state model | ✅ pallet-qutxo |
| Real ML-DSA-44 verification | ✅ crates/aegis-crypto |
| Lock binding (pubkey hash) | ✅ verify_aegis_transaction |
| Fast-Path conflict tags | ✅ validate_unsigned |
| Stale gate | ✅ UtxoSet::contains_key |
| Conservation of mass | ✅ execute_utxo_tx invariant |
| Named testnet | ✅ Calibre Testnet 1 |
| Network fingerprint | ✅ SHA-256 7b7ba4d1... |
| HTTP faucet | ✅ tools/faucet |
| Docker deployment | ✅ docker compose up |
| Prometheus metrics | ✅ :9615 scraped by Prometheus |
| Whitepaper + docs | ✅ this repository |

### 8.2 What is not implemented (roadmap)

| Component | Target phase |
|:---|:---|
| Multi-validator testnet | Phase 6.6 |
| Benchmark-derived weights | Phase 8 |
| Fuzz targets for crypto & UTXO | Phase 8 |
| Third-party audit | Phase 8 |
| Light nodes with ZK state proofs | Phase 7 |
| A.E.G.I.S. threshold MPC | Phase 9 |
| Hardware attestation verification | Phase 9 |
| ZK Intent Bridge | Phase 9 |
| ZkShield private UTXOs | Phase 10 |
| Mainnet launch | Phase 11 |

### 8.3 Testnet limitations

The current testnet is intended for research and demonstration, not for
value transfer. Specifically:

- **One validator** — Alice is the only authority.
- **No economics** — minting is unrestricted via sudo.
- **No attestation** — hardware attestation is a no-op.
- **No audit** — code has not been externally reviewed.
- **Unstable API** — state may be wiped between releases.

---

## 9. Roadmap

### Phase 6.6 — Multi-validator testnet
- Add 3 additional authorities (Bob, Charlie, Dave)
- Regenerate chain spec + fingerprint
- Publish bootnode addresses

### Phase 7 — Light nodes
- Generate succinct state proofs per block
- Publish proofs over libp2p for mobile verification

### Phase 8 — Audit preparation
- cargo benchmark for all pallets
- Fuzz targets for QR-CKD, verify_aegis_transaction, execute_utxo_tx
- try-runtime migrations
- Third-party audit
- Bug bounty (Immunefi)

### Phase 9 — ZK Intent Bridge
- SP1 circuits for Ethereum light-client verification
- Solver network with $CAL bonding and slashing

### Phase 10 — ZkShield private UTXOs
- Pedersen commitments for hidden values
- Nullifier sets for double-spend prevention
- ZK-SNARK verifier pallet

### Phase 11 — Mainnet
- Finalize tokenomics
- Decentralize the Guardian multi-sig

---

## 10. Security Analysis

### 10.1 Threat model

Calibre's security is evaluated against:

- **Quantum adversaries** with BQP computational power
- **Classical adversaries** with polynomial resources
- **Malicious validators** up to the BFT threshold (⅓ of stake)
- **Network adversaries** capable of eclipse attacks

### 10.2 Assumptions

- ML-DSA-44 is unforgeable under chosen-message attack
- Blake2b-256 is collision-resistant
- SHAKE256 is a secure extendable-output function
- Aura + GRANDPA provide eventual finality under ≥⅔ honest validators

### 10.3 Known limitations

- Attestation is a no-op
- A.E.G.I.S. threshold is unimplemented
- No ZK proofs are verified on-chain
- Weights are hardcoded
- No try-runtime migrations

---

## 11. References

1. NIST FIPS 204 — Module-Lattice-Based Digital Signature Standard, 2024.
2. NIST SP 800-227 — Recommendations for Key-Encapsulation Mechanisms, draft.
3. NIST IR 8547 — Transition to Post-Quantum Cryptography Standards, draft.
4. Bernstein, D. J., & Lange, T. — Post-quantum cryptography, Nature, 2017.
5. IETF — Composite ML-DSA for use in X.509 and TLS, draft.
6. Substrate — Runtime Development Documentation, Parity Technologies.
7. dilithium-rs 0.4.1 — crates.io
8. Lamport, L. — Constructing Digital Signatures from a One-Way Function, 1979.

---

## Appendix A — Glossary

| Term | Meaning |
|:---|:---|
| A.E.G.I.S. | Attestation-Enhanced Quantum-Resilient Identity System |
| Aura | Substrate's block production algorithm (slot-based) |
| GRANDPA | Substrate's finality gadget |
| ML-DSA-44 | NIST FIPS 204 signature standard, security level 2 |
| PQ-KDF | Post-quantum key derivation function |
| Q-UTXO | Quantum-UTXO |
| QR-CKD | Quantum-Resilient Contextual Key Derivation |
| SCALE | Substrate's compact encoding |
| UTXO | Unspent Transaction Output |

---

*End of whitepaper.*
