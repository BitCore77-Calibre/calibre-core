//! pallet-zk-verifier — records RISC Zero proof attestations on-chain.
//!
//! The actual verification runs in the node (see `calibre-risc0-host`).
//! This pallet:
//!   1. Receives a serialized receipt + journal from the user.
//!   2. Calls the host verifier via the runtime interface.
//!   3. On success, stores a record keyed by blake2_256(journal).

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_core::hashing::blake2_256;
    use sp_std::vec::Vec;
    use frame_support::traits::ConstU32;

    pub type JournalHash = [u8; 32];

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The RISC Zero guest image ID we accept proofs for.
        #[pallet::constant]
        type GuestImageId: Get<[u8; 32]>;

    }


    #[derive(
        Clone, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, PartialEq, Eq,
    )]
    pub struct AttestationRecord<BlockNumber> {
        pub journal: BoundedVec<u8, ConstU32<512>>,
        pub block_number: BlockNumber,
    }

    #[pallet::storage]
    pub type Attestations<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        JournalHash,
        AttestationRecord<BlockNumberFor<T>>,
        OptionQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ProofVerified { journal_hash: JournalHash, block_number: BlockNumberFor<T> },
        ProofRejected { journal_hash: JournalHash },
    }

    #[pallet::error]
    pub enum Error<T> {
        ProofVerificationFailed,
        JournalTooLarge,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Submit a RISC Zero receipt. If it verifies against the
        /// configured `GuestImageId`, store a record. Otherwise, reject.
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(100_000_000_000, 5_000_000))]
        pub fn submit_proof(
            origin: OriginFor<T>,
            receipt: Vec<u8>,
            journal: Vec<u8>,
        ) -> DispatchResult {
            ensure_signed(origin)?;

            let vk = T::GuestImageId::get();
            let ok = calibre_risc0_host::calibre_risc_0::verify_receipt(
                vk,
                &receipt,
                &journal,
            );
            let journal_hash = blake2_256(&journal);

            ensure!(ok, Error::<T>::ProofVerificationFailed);

            let bounded: BoundedVec<u8, ConstU32<512>> = journal
                .try_into()
                .map_err(|_| Error::<T>::JournalTooLarge)?;

            let block_number = frame_system::Pallet::<T>::block_number();
            Attestations::<T>::insert(
                journal_hash,
                AttestationRecord { journal: bounded, block_number },
            );
            Self::deposit_event(Event::ProofVerified { journal_hash, block_number });
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Query whether a journal has been verified on-chain.
        pub fn is_verified(journal_hash: &JournalHash) -> bool {
            Attestations::<T>::contains_key(journal_hash)
        }
    }
}
