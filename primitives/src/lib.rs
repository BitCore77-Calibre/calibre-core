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
pub trait FeeHandler<Balance> {
    /// Split a fee into (producer, treasury, burn) and record it.
    fn charge_fee(fee: Balance) -> (Balance, Balance, Balance);

    /// Minimum acceptable fee for a tx with N inputs and M outputs.
    fn minimum_fee(inputs: u32, outputs: u32) -> Balance;

    /// Charge a priority fee (tip above base fee). Routes 100% to producer.
    fn charge_priority_fee(fee: Balance) -> Balance;
}

impl<Balance: Default + Copy> FeeHandler<Balance> for () {
    fn charge_fee(_fee: Balance) -> (Balance, Balance, Balance) {
        let z = Balance::default();
        (z, z, z)
    }
    fn minimum_fee(_inputs: u32, _outputs: u32) -> Balance {
        Balance::default()
    }
    fn charge_priority_fee(_fee: Balance) -> Balance {
        Balance::default()
    }
}

/// Mint `value` into a UTXO locked to `lock`. Implemented by pallet-qutxo,
/// consumed by pallet-calibre-fees to settle producer payouts at block end.
pub trait FeeMinter<Balance, Lock> {
    fn mint_to_lock(value: Balance, lock: Lock);
}

/// Consume UTXOs (verify witness, remove from set, decrement TotalIssuance).
/// Implemented by pallet-qutxo; used by pallet-stake to burn user funds into
/// the staking ledger.
pub trait UtxoConsumer<Balance> {
    /// Verify the AEGIS witness against the first input's lock, then remove
    /// every input from the UTXO set. Returns the total consumed value, or
    /// `None` if any input is missing or the witness fails verification.
    /// On success, decrements `TotalIssuance` by the consumed amount.
    fn consume_with_witness(
        inputs: &[TransactionInput],
        witness: &[u8],
    ) -> Option<Balance>;
}

impl<Balance: Default + Copy> UtxoConsumer<Balance> for () {
    fn consume_with_witness(_inputs: &[TransactionInput], _witness: &[u8]) -> Option<Balance> {
        Some(Balance::default())
    }
}

impl<Balance, Lock> FeeMinter<Balance, Lock> for () {
    fn mint_to_lock(_value: Balance, _lock: Lock) {}
}
