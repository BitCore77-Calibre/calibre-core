//! Runtime API declaration for Q-UTXO inclusion proofs.
//! Kept in its own crate so both the runtime (implements) and the
//! node (consumes) can name the trait without a circular dependency.

#![cfg_attr(not(feature = "std"), no_std)]

use calibre_primitives::InclusionProof;
use sp_core::H256;

sp_api::decl_runtime_apis! {
    /// Read-only API for fetching Merkle inclusion proofs from the
    /// live UTXO set.
    pub trait QutxoApi {
        /// Returns the inclusion proof for `utxo_id`, or `None` if
        /// the UTXO is not in the current set.
        fn get_inclusion_proof(utxo_id: H256) -> Option<InclusionProof>;
    }
}
