//! Native-side RISC Zero verifier, exposed to the runtime via
//! `sp_runtime_interface`.
//!
//! Verification runs in the node (native), not in the runtime (WASM).
//! The WASM side gets extern declarations that dispatch to the node.

#![cfg_attr(not(feature = "std"), no_std)]

#[sp_runtime_interface::runtime_interface]
pub trait CalibreRisc0 {
    /// Verify a RISC Zero receipt against the given image ID and journal.
    fn verify_receipt(vk: [u8; 32], receipt: &[u8], journal: &[u8]) -> bool {
        let proof: risc0_verifier::Proof = match ciborium::from_reader(receipt) {
            Ok(p) => p,
            Err(_) => return false,
        };
        let vk_typed = risc0_verifier::Vk::from(vk);
        let j = risc0_verifier::Journal::new(journal.to_vec());
        risc0_verifier::verify(
            &risc0_verifier::v3_0(),
            vk_typed,
            proof,
            j,
        ).is_ok()
    }
}
