#![cfg_attr(not(feature = "std"), no_std)]
use parity_scale_codec::{Decode, Encode, MaxEncodedLen, DecodeWithMemTracking};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

#[derive(Clone, Encode, Decode, DecodeWithMemTracking, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum QuantumLock { SingleSig([u8; 32]), AegisThreshold([u8; 32]) }

#[derive(Clone, Encode, Decode, DecodeWithMemTracking, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct Utxo<Balance> { pub value: Balance, pub lock: QuantumLock }

#[derive(Clone, Encode, Decode, DecodeWithMemTracking, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct TransactionInput { pub tx_hash: sp_core::H256, pub output_index: u32 }

#[derive(Clone, Encode, Decode, DecodeWithMemTracking, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct TransactionOutput<Balance> { pub value: Balance, pub lock: QuantumLock }

/// One step in a Merkle inclusion path.
/// `current_is_left == true` means the running node is the LEFT child,
/// so the fold is `inner_hash(current, sibling)`; otherwise
/// `inner_hash(sibling, current)`.
/// Must match `calibre_merkle::merkle_path` and the RISC Zero guest.
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct MerkleStep { pub sibling: sp_core::H256, pub current_is_left: bool }

/// Everything a light client needs to prove one UTXO is in the set:
/// the root it commits against, the leaf, and the path between them.
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, PartialEq, Eq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct InclusionProof {
    pub root: sp_core::H256,
    pub utxo_id: sp_core::H256,
    pub value_hash: sp_core::H256,
    pub path: sp_std::vec::Vec<MerkleStep>,
}

/// Interface for routing UTXO transaction fees.
///
/// Consumed by `pallet-qutxo` in `execute_utxo_tx`. Implemented by
/// `pallet-calibre-fees` in the runtime. `()` is a no-op that discards
/// fees — used in unit tests that don't care about routing.
pub trait FeeHandler<AccountId, Balance> {
    /// Split a fee into (producer, treasury, burn) and record it.
    fn charge_fee(fee: Balance, producer: Option<AccountId>) -> (Balance, Balance, Balance);

    /// Minimum acceptable fee for a tx with N inputs and M outputs.
    fn minimum_fee(inputs: u32, outputs: u32) -> Balance;
}

impl<AccountId, Balance: Default + Copy> FeeHandler<AccountId, Balance> for () {
    fn charge_fee(_fee: Balance, _producer: Option<AccountId>) -> (Balance, Balance, Balance) {
        let z = Balance::default();
        (z, z, z)
    }
    fn minimum_fee(_inputs: u32, _outputs: u32) -> Balance {
        Balance::default()
    }
}
