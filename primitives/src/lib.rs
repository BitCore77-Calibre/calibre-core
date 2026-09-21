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
