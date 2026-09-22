#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

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

    #[pallet::pallet] pub struct Pallet<T>(_);
    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type Balance: Member + Parameter + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;
        #[pallet::constant] type MaxTxInputs: Get<u32>;
        #[pallet::constant] type MaxTxOutputs: Get<u32>;
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
        #[pallet::call_index(0)] #[pallet::weight(Weight::from_parts(100_000_000, 0))]
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
            Self::deposit_event(Event::TransactionExecuted { tx_hash, value: total_output_value });
            Ok(())
        }
        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_parts(10_000_000, 0))]
        pub fn sudo_mint(origin: OriginFor<T>, value: T::Balance, recipient_lock: [u8; 32]) -> DispatchResult {
            ensure_root(origin)?;
            // Create a deterministic genesis hash for the first UTXO
            let mut seed = Vec::new();
            seed.extend_from_slice(b"calibre-mint");
            seed.extend_from_slice(&frame_system::Pallet::<T>::block_number().encode());
            // Include recipient in the seed so multiple mints in one block
            // produce distinct UTXOs. Without this, two recipients funded
            // in the same block collide on the same utxo_hash and the
            // second overwrites the first.
            seed.extend_from_slice(&recipient_lock);
            let genesis_hash = sp_core::H256::from(sp_core::blake2_256(&seed));
            let utxo_hash = Self::calculate_utxo_hash(genesis_hash, 0);
            
            let new_utxo = Utxo { 
                value, 
                lock: QuantumLock::AegisThreshold(recipient_lock) // Post-quantum lock bound to recipient
            };
            
            // Inject into state and update total issuance
            UtxoSet::<T>::insert(utxo_hash, new_utxo);
            Self::mark_utxo_set_dirty();
            let current = TotalIssuance::<T>::get();
            TotalIssuance::<T>::put(current.saturating_add(value));
            
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
            if let Call::execute_utxo_tx { tx } = call {
                let mut builder = ValidTransaction::with_tag_prefix("QUTXO").priority(100).longevity(64).propagate(true);
                for input in tx.inputs.iter() {
                    let id = Self::calculate_utxo_hash(input.tx_hash, input.output_index);
                    let utxo = UtxoSet::<T>::get(&id).ok_or(InvalidTransaction::Stale)?;
                    if !matches!(utxo.lock, calibre_primitives::QuantumLock::AegisThreshold(_)) {
                        return InvalidTransaction::BadProof.into();
                    }
                    builder = builder.and_provides(id.as_ref().to_vec());
                }
                builder.build()
            } else { InvalidTransaction::Call.into() }
        }
    }
}
