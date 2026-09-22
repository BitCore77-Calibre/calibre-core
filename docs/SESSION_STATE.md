# Calibre Session State — d7d61fe2

See the SESSION HANDOFF document in the chat history (or the commit messages
since 28498766) for the full picture.

## Just shipped
- **Phase 7.4 unblocked**: on-chain RISC Zero verifier works.
  - crates/calibre-risc0-host uses risc0-zkvm 3.0.6 (same crate as prover).
  - vk conversion [u8; 32] → [u32; 8] BE before verify.
  - Verified end-to-end against a fresh 3.0.6 receipt.
  - Standalone test: /tmp/pallet-verify/target/release/pallet-verify

## Measured
- 167 TPS single-validator, 1006 qutxo/6s block
- Bottleneck: proof_size (99.9% used), NOT crypto (14% ref_time)
- ML-DSA-44 verify: 208 µs/tx, ~$1e-9 electricity
- See docs/BENCHMARK.md

## Next (priorities)
1. Batch B: proof_size 5→15 MB. Expected ~480 TPS.
2. Batch C: slot 6s→1s. Combined with B, expected ~2880 TPS.
3. Multi-validator Docker: 7 vals on 3 Linux machines, staged LAN → WAN.
4. Anti-spam for execute_utxo_tx (validate_unsigned doesn't verify sigs).
5. Session key management.
6. 7.6 Android (Ada).

## Working commands and rules
See the SESSION HANDOFF message in the previous chat.
