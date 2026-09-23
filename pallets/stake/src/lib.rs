#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

#[cfg(test)]
mod tests;

pub mod weights;
pub use weights::WeightInfo;

#[frame_support::pallet]
pub mod pallet {
    use crate::WeightInfo;
    use calibre_primitives::{FeeMinter, TransactionInput, UtxoConsumer};
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::AtLeast32BitUnsigned;
    use sp_runtime::Saturating;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type Balance: Member
            + Parameter
            + AtLeast32BitUnsigned
            + Default
            + Copy
            + MaxEncodedLen
            + From<u128>
            + Into<u128>;

        #[pallet::constant]
        type MaxBondInputs: Get<u32>;

        /// Consumes UTXOs at bond time (qutxo in the runtime; () in tests).
        type UtxoConsumer: UtxoConsumer<Self::Balance>;

        /// Mints UTXOs at unbond time (qutxo in the runtime; () in tests).
        type UtxoMinter: FeeMinter<Self::Balance, [u8; 32]>;

        type WeightInfo: WeightInfo;
    }

    /// Per-account staked balance. This is a pallet-internal ledger: the
    /// value is not represented by any UTXO while bonded. Upgrade path to
    /// Cardano-style liquid staking (payment key + staking key split) is a
    /// future redesign — see docs/SESSION_STATE.md.
    #[pallet::storage]
    #[pallet::getter(fn stake_of)]
    pub type Stake<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, T::Balance, ValueQuery>;

    /// Sum of all bonded balances. Stats only; does not gate consensus yet.
    #[pallet::storage]
    #[pallet::getter(fn total_staked)]
    pub type TotalStaked<T: Config> = StorageValue<_, T::Balance, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        Bonded { who: T::AccountId, amount: T::Balance },
        Unbonded { who: T::AccountId, amount: T::Balance, lock: [u8; 32] },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Unbond amount exceeds the account's bonded balance.
        InsufficientStake,
        /// Too many inputs in a single bond call.
        BondTooManyInputs,
        /// The UTXO consumer rejected the inputs or witness.
        UtxoConsumeFailed,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Bond UTXOs into the staking ledger. The witness must sign
        /// `(inputs, [])` against the first input's lock. On success, the
        /// UTXOs are consumed (removed from the set, TotalIssuance
        /// decremented) and `Stake[who]` is credited.
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::bond())]
        pub fn bond(
            origin: OriginFor<T>,
            inputs: sp_std::vec::Vec<TransactionInput>,
            witness: sp_std::vec::Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                inputs.len() <= T::MaxBondInputs::get() as usize,
                Error::<T>::BondTooManyInputs
            );
            let amount = T::UtxoConsumer::consume_with_witness(&inputs, &witness)
                .ok_or(Error::<T>::UtxoConsumeFailed)?;
            Stake::<T>::mutate(&who, |s| *s = s.saturating_add(amount));
            TotalStaked::<T>::mutate(|t| *t = t.saturating_add(amount));
            Self::deposit_event(Event::Bonded { who, amount });
            Ok(())
        }

        /// Unbond `amount` from the caller's staked balance. Mints a fresh
        /// UTXO locked to `lock`. No unbonding period yet — added when
        /// staking gates validator participation.
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::unbond())]
        pub fn unbond(
            origin: OriginFor<T>,
            amount: T::Balance,
            lock: [u8; 32],
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let current = Stake::<T>::get(&who);
            ensure!(current >= amount, Error::<T>::InsufficientStake);
            Stake::<T>::insert(&who, current.saturating_sub(amount));
            TotalStaked::<T>::mutate(|t| *t = t.saturating_sub(amount));
            T::UtxoMinter::mint_to_lock(amount, lock);
            Self::deposit_event(Event::Unbonded { who, amount, lock });
            Ok(())
        }
    }
}
