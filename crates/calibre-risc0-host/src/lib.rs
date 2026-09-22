//! Native-side RISC Zero verifier, exposed to the runtime via
//! `sp_runtime_interface`.
//!
//! Verification runs in the node (native), not in the runtime (WASM).
//! We use `risc0-zkvm` (the same crate that generates the receipt) so the
//! claim digest algorithm matches the prover by construction.

#![cfg_attr(not(feature = "std"), no_std)]

#[sp_runtime_interface::runtime_interface]
pub trait CalibreRisc0 {
    /// Verify a RISC Zero receipt against the given image ID and journal.
    ///
    /// `vk` is the 32-byte image ID as printed by the guest build (8 u32 words,
    /// each written big-endian: word value 0xf774fe2d becomes bytes
    /// f7 74 fe 2d). risc0-zkvm's `Digest::from([u8; 32])` reinterprets bytes
    /// as little-endian u32s, so we must convert explicitly to recover the
    /// intended word values.
    fn verify_receipt(vk: [u8; 32], receipt: &[u8], journal: &[u8]) -> bool {
        let r: risc0_zkvm::Receipt = match ciborium::from_reader(receipt) {
            Ok(r) => r,
            Err(_) => return false,
        };
        if r.journal.bytes.as_slice() != journal {
            return false;
        }
        // Big-endian word interpretation: bytes [f7,74,fe,2d] -> u32 0xf774fe2d
        let mut vk_words = [0u32; 8];
        for i in 0..8 {
            vk_words[i] = u32::from_be_bytes([vk[4*i], vk[4*i+1], vk[4*i+2], vk[4*i+3]]);
        }
        r.verify(vk_words).is_ok()
    }
}
