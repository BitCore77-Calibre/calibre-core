#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

pub mod weights;
pub use weights::WeightInfo;

#[cfg(test)]
mod tests;
#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::{AtLeast32BitUnsigned, BlakeTwo256, Hash, Saturating};
    use sp_std::prelude::*;
    use calibre_primitives::{QuantumLock, Utxo, TransactionInput, TransactionOutput};
    use frame_system::ensure_root;
    use calibre_aegis_crypto::AegisWitness;
    use crate::weights::WeightInfo;
    use calibre_primitives::FeeHandler;

    #[pallet::pallet] pub struct Pallet<T>(_);
    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type Balance: Member + Parameter + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen
            + serde::Serialize + serde::de::DeserializeOwned;
        #[pallet::constant] type MaxTxInputs: Get<u32>;
        #[pallet::constant] type MaxTxOutputs: Get<u32>;
		type WeightInfo: WeightInfo;
		/// Fee router: () in tests, pallet-calibre-fees in the runtime.
		type FeeHandler: FeeHandler<Self::Balance>;
	}
    #[pallet::storage] #[pallet::getter(fn utxo_set)]
    pub type UtxoSet<T: Config> = StorageMap<_, Blake2_128Concat, sp_core::H256, Utxo<T::Balance>, OptionQuery>;
    #[pallet::storage] #[pallet::getter(fn total_issuance)]
    pub type TotalIssuance<T: Config> = StorageValue<_, T::Balance, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn utxo_set_root)]
    pub type UtxoSetRoot<T: Config> = StorageValue<_, sp_core::H256, ValueQuery>;

    #[pallet::storage]
    pub type UtxoSetDirty<T: Config> = StorageValue<_, bool, ValueQuery>;

    #[derive(Clone, Encode, Decode, parity_scale_codec::DecodeWithMemTracking, PartialEq, Eq, RuntimeDebug, TypeInfo)]
    pub struct Transaction<Balance> {
        pub inputs: Vec<TransactionInput>, pub outputs: Vec<TransactionOutput<Balance>>,
        pub pq_signature: Vec<u8>, pub witness: Vec<u8>, 
    }
    #[pallet::event] #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> { TransactionExecuted { tx_hash: sp_core::H256, value: T::Balance } }
    #[pallet::error]
    pub enum Error<T> { UtxoDoesNotExist, ValueMismatch, TransactionTooLarge, InvalidPqSignature }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_finalize(_n: BlockNumberFor<T>) {
            if UtxoSetDirty::<T>::take() {
                let root = Self::compute_utxo_set_root();
                UtxoSetRoot::<T>::put(root);
            }
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::execute_utxo_tx(
            tx.inputs.len() as u32,
            tx.outputs.len() as u32,
        ))]
        pub fn execute_utxo_tx(origin: OriginFor<T>, tx: Transaction<T::Balance>) -> DispatchResult {
            ensure_none(origin)?;
            ensure!(tx.inputs.len() <= T::MaxTxInputs::get() as usize, Error::<T>::TransactionTooLarge);
            ensure!(tx.outputs.len() <= T::MaxTxOutputs::get() as usize, Error::<T>::TransactionTooLarge);
            let tx_payload = (tx.inputs.clone(), tx.outputs.clone());
            let tx_hash = BlakeTwo256::hash(&tx_payload.encode());
            let witness: AegisWitness = Decode::decode(&mut &tx.witness[..]).map_err(|_| Error::<T>::InvalidPqSignature)?;
            // ── REAL POST-QUANTUM VERIFICATION ──
            let first_lock = UtxoSet::<T>::get(Self::calculate_utxo_hash(tx.inputs[0].tx_hash, tx.inputs[0].output_index))
                .ok_or(Error::<T>::UtxoDoesNotExist)?.lock;
            calibre_aegis_crypto::AegisCryptoCore::verify_aegis_transaction(
                &first_lock,
                &tx_payload.encode(),
                &witness,
            ).map_err(|_| Error::<T>::InvalidPqSignature)?;
            let mut total_input_value = T::Balance::default();
            for input in &tx.inputs {
                let utxo_hash = Self::calculate_utxo_hash(input.tx_hash, input.output_index);
                let utxo = UtxoSet::<T>::get(utxo_hash).ok_or(Error::<T>::UtxoDoesNotExist)?;
                total_input_value = total_input_value.saturating_add(utxo.value);
                UtxoSet::<T>::remove(utxo_hash);
                    Self::mark_utxo_set_dirty();
            }
            let mut total_output_value = T::Balance::default();
            for (idx, output) in tx.outputs.iter().enumerate() {
                total_output_value = total_output_value.saturating_add(output.value);
                let new_utxo_hash = Self::calculate_utxo_hash(tx_hash, idx as u32);
                UtxoSet::<T>::insert(new_utxo_hash, Utxo { value: output.value, lock: output.lock.clone() });
            Self::mark_utxo_set_dirty();
            }
            ensure!(total_input_value >= total_output_value, Error::<T>::ValueMismatch);
            let total_fee = total_input_value.saturating_sub(total_output_value);
            if total_fee > T::Balance::default() {
                let base_fee = T::FeeHandler::minimum_fee(tx.inputs.len() as u32, tx.outputs.len() as u32);
                let priority_fee = total_fee.saturating_sub(base_fee);
                
                // Route base fee through 50/30/20 split
                let _ = T::FeeHandler::charge_fee(base_fee);
                
                // Route priority fee 100% to producer
                if priority_fee > T::Balance::default() {
                    let _ = T::FeeHandler::charge_priority_fee(priority_fee);
                }
            }
            Self::deposit_event(Event::TransactionExecuted { tx_hash, value: total_output_value });
            Ok(())
        }
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::sudo_mint())]
        pub fn sudo_mint(origin: OriginFor<T>, value: T::Balance, recipient_lock: [u8; 32]) -> DispatchResult {
            ensure_root(origin)?;
            let genesis_hash = Self::mint_utxo(value, recipient_lock);
            Self::deposit_event(Event::TransactionExecuted { tx_hash: genesis_hash, value });
            Ok(())
        }

    }
    impl<T: Config> Pallet<T> {
        /// Recompute the UTXO set root from scratch.
        ///
        /// O(n log n) in the number of UTXOs. Acceptable for testnet.
        /// Phase 7.2 will replace the body with an incremental SMT update
        /// while keeping this call site intact.
        /// Build an inclusion proof for one UTXO.
        /// Reads the whole UTXO set (O(n log n)) — acceptable for
        /// testnet. Returns None if `utxo_id` is not in the set.
        pub fn get_inclusion_proof(
            utxo_id: sp_core::H256,
        ) -> Option<calibre_primitives::InclusionProof> {
            let leaves: sp_std::vec::Vec<(calibre_merkle::Hash, calibre_merkle::Hash)> =
                UtxoSet::<T>::iter()
                    .map(|(id, utxo)| {
                        (*id.as_fixed_bytes(), *Self::hash_utxo(&utxo).as_fixed_bytes())
                    })
                    .collect();

            let tid: calibre_merkle::Hash = *utxo_id.as_fixed_bytes();
            let path = calibre_merkle::merkle_path(&leaves, &tid)?;
            let root = calibre_merkle::utxo_set_root(leaves.clone());

            let value_hash = leaves
                .iter()
                .find(|(id, _)| *id == tid)
                .map(|(_, v)| *v)?;

            Some(calibre_primitives::InclusionProof {
                root: sp_core::H256::from(root),
                utxo_id,
                value_hash: sp_core::H256::from(value_hash),
                path: path
                    .into_iter()
                    .map(|(sib, is_left)| calibre_primitives::MerkleStep {
                        sibling: sp_core::H256::from(sib),
                        current_is_left: is_left,
                    })
                    .collect(),
            })
        }

        pub fn compute_utxo_set_root() -> sp_core::H256 {
            sp_core::H256::from(calibre_merkle::utxo_set_root(
                UtxoSet::<T>::iter()
                    .map(|(id, utxo)| (*id.as_fixed_bytes(), *Self::hash_utxo(&utxo).as_fixed_bytes())),
            ))
        }

        /// Hash of a single UTXO's value. Uses SCALE encoding under a
        /// domain separator; do not change without also bumping the
        /// circuit's commitment scheme.
        fn hash_utxo(utxo: &Utxo<T::Balance>) -> sp_core::H256 {
            sp_core::H256::from(sp_core::hashing::blake2_256(&utxo.encode()))
        }

        /// Call from every path that mutates `UtxoSet`.
        pub fn mark_utxo_set_dirty() {
            UtxoSetDirty::<T>::put(true);
        }

        /// Mint a UTXO locked to `recipient_lock` and return its genesis hash.
        /// Used by `sudo_mint` and by pallet-calibre-fees' producer payout settle.
        pub fn mint_utxo(value: T::Balance, recipient_lock: [u8; 32]) -> sp_core::H256 {
            let mut seed = Vec::new();
            seed.extend_from_slice(b"calibre-mint");
            seed.extend_from_slice(&frame_system::Pallet::<T>::block_number().encode());
            seed.extend_from_slice(&recipient_lock);
            let genesis_hash = sp_core::H256::from(sp_core::blake2_256(&seed));
            let utxo_hash = Self::calculate_utxo_hash(genesis_hash, 0);
            let new_utxo = Utxo {
                value,
                lock: QuantumLock::AegisThreshold(recipient_lock),
            };
            UtxoSet::<T>::insert(utxo_hash, new_utxo);
            Self::mark_utxo_set_dirty();
            let current = TotalIssuance::<T>::get();
            TotalIssuance::<T>::put(current.saturating_add(value));
            genesis_hash
        }

        pub fn calculate_utxo_hash(tx_hash: sp_core::H256, output_index: u32) -> sp_core::H256 {
            let mut bytes = Vec::new();
            bytes.extend_from_slice(tx_hash.as_bytes());
            bytes.extend_from_slice(&output_index.to_le_bytes());
            BlakeTwo256::hash(&bytes)
        }
    }
    #[pallet::validate_unsigned]
    impl<T: Config> ValidateUnsigned for Pallet<T> {
        type Call = Call<T>;

        fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
            let Call::execute_utxo_tx { tx } = call else {
                return InvalidTransaction::Call.into();
            };

            // ── BOUNDED-SIZE CHECKS (cheapest, run first) ──
            // Reject oversized inputs before any allocation / iteration.
            if tx.inputs.is_empty() || tx.inputs.len() > T::MaxTxInputs::get() as usize {
                return InvalidTransaction::ExhaustsResources.into();
            }
            if tx.outputs.len() > T::MaxTxOutputs::get() as usize {
                return InvalidTransaction::ExhaustsResources.into();
            }
            // ML-DSA-44 signature ≈ 2.4 KB, pubkey ≈ 1.3 KB → 8 KB is generous.
            const MAX_WITNESS_BYTES: usize = 8 * 1024;
            if tx.witness.is_empty() || tx.witness.len() > MAX_WITNESS_BYTES {
                return InvalidTransaction::BadProof.into();
            }

            // ── INTRA-TX DUPLICATE INPUT DETECTION ──
            // Same UTXO listed twice would pass the pool's per-UTXO dedup but
            // double-count value at dispatch. O(n²) is fine: n ≤ MaxTxInputs.
            for i in 0..tx.inputs.len() {
                for j in (i + 1)..tx.inputs.len() {
                    if tx.inputs[i].tx_hash == tx.inputs[j].tx_hash
                        && tx.inputs[i].output_index == tx.inputs[j].output_index
                    {
                        return InvalidTransaction::BadProof.into();
                    }
                }
            }

            // ── UTXO EXISTENCE + LOCK-TYPE CHECK ──
            let first_id = Self::calculate_utxo_hash(
                tx.inputs[0].tx_hash,
                tx.inputs[0].output_index,
            );
            let first_utxo = UtxoSet::<T>::get(&first_id).ok_or(InvalidTransaction::Stale)?;
            if !matches!(first_utxo.lock, QuantumLock::AegisThreshold(_)) {
                return InvalidTransaction::BadProof.into();
            }
            for input in tx.inputs.iter().skip(1) {
                let id = Self::calculate_utxo_hash(input.tx_hash, input.output_index);
                if UtxoSet::<T>::get(&id).is_none() {
                    return InvalidTransaction::Stale.into();
                }
            }

            // ── FULL POST-QUANTUM VERIFICATION ──
            // Mirror execute_utxo_tx: verify against the first input's lock.
            // Invalid signatures never enter the pool, so a flood of junk
            // costs the attacker one ML-DSA verify per attempt — they pay
            // it, not the block producer.
            let witness: AegisWitness = match Decode::decode(&mut &tx.witness[..]) {
                Ok(w) => w,
                Err(_) => return InvalidTransaction::BadProof.into(),
            };
            let payload = (tx.inputs.clone(), tx.outputs.clone()).encode();
            if calibre_aegis_crypto::AegisCryptoCore::verify_aegis_transaction(
                &first_utxo.lock,
                &payload,
                &witness,
            )
            .is_err()
            {
                return InvalidTransaction::BadProof.into();
            }

            // ── FEE CHECK (pool-side minimum fee) ──
            // Reconstruct what execute_utxo_tx will compute. Reject under-priced
            // txs here so the pool can't be flooded with zero-fee junk.
            let mut total_in = T::Balance::default();
            for input in tx.inputs.iter() {
                let id = Self::calculate_utxo_hash(input.tx_hash, input.output_index);
                if let Some(u) = UtxoSet::<T>::get(&id) {
                    total_in = total_in.saturating_add(u.value);
                }
            }
            let mut total_out = T::Balance::default();
            for out in tx.outputs.iter() {
                total_out = total_out.saturating_add(out.value);
            }
            let fee = total_in.saturating_sub(total_out);
            let min_fee = T::FeeHandler::minimum_fee(
                tx.inputs.len() as u32,
                tx.outputs.len() as u32,
            );
            if fee < min_fee {
                return InvalidTransaction::Payment.into();
            }

            // ── PROVIDES (pool dedup per UTXO) ──
            let mut builder = ValidTransaction::with_tag_prefix("QUTXO")
                .priority(100)
                .longevity(64)
                .propagate(true);
            for input in tx.inputs.iter() {
                let id = Self::calculate_utxo_hash(input.tx_hash, input.output_index);
                builder = builder.and_provides(id.as_ref().to_vec());
            }
            builder.build()
        }
    }

    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T: Config> {
        pub initial_utxos: Vec<(sp_core::H256, T::Balance, [u8; 32])>,
        #[serde(skip)]
        pub _config: sp_std::marker::PhantomData<T>,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            let mut total = T::Balance::default();
            for (hash, value, lock_bytes) in &self.initial_utxos {
                let utxo = Utxo {
                    value: *value,
                    lock: calibre_primitives::QuantumLock::AegisThreshold(*lock_bytes),
                };
                UtxoSet::<T>::insert(hash, utxo);
                total = total.saturating_add(*value);
            }
            TotalIssuance::<T>::put(total);
        }
    }

}

impl<T: pallet::Config> calibre_primitives::FeeMinter<T::Balance, [u8; 32]> for pallet::Pallet<T> {
    fn mint_to_lock(value: T::Balance, lock: [u8; 32]) {
        let _ = Self::mint_utxo(value, lock);
    }
}
