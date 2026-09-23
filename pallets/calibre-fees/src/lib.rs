#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

#[cfg(test)]
mod tests;

pub mod weights;
pub use weights::WeightInfo;

/// Interface used by pallet-qutxo to route fees without a hard dependency.
pub trait FeeHandler<AccountId, Balance> {
    /// Split and record a fee. Returns (producer_cut, treasury_cut, burn_cut).
    fn charge_fee(fee: Balance, producer: Option<AccountId>) -> (Balance, Balance, Balance);
    /// Minimum acceptable fee for a tx with N inputs, M outputs.
    fn minimum_fee(inputs: u32, outputs: u32) -> Balance;
}

#[frame_support::pallet]
pub mod pallet {
    use crate::{FeeHandler, WeightInfo};
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::AtLeast32BitUnsigned;
    use sp_runtime::Saturating;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Balance type — must match pallet-qutxo's Balance for the FeeHandler to bind.
        type Balance: Member
            + Parameter
            + AtLeast32BitUnsigned
            + Default
            + Copy
            + MaxEncodedLen
            + From<u128>
            + Into<u128>;

        /// Minimum base fee per transaction.
        #[pallet::constant]
        type BaseTxFee: Get<Self::Balance>;

        /// Additional fee per input + output.
        #[pallet::constant]
        type PerInOutFee: Get<Self::Balance>;

        /// Percent of the fee routed to the block producer (0-100).
        #[pallet::constant]
        type ProducerFeeShare: Get<u8>;

        /// Percent of the fee routed to the treasury (0-100).
        #[pallet::constant]
        type TreasuryFeeShare: Get<u8>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;
    }

    #[pallet::storage]
    #[pallet::getter(fn treasury_lock)]
    pub type TreasuryLock<T: Config> = StorageValue<_, [u8; 32], ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn producer_lock)]
    pub type ProducerLocks<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, [u8; 32]>;

    #[pallet::storage]
    #[pallet::getter(fn treasury_accumulated)]
    pub type TreasuryAccumulated<T: Config> = StorageValue<_, T::Balance, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn burn_counter)]
    pub type BurnCounter<T: Config> = StorageValue<_, T::Balance, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A tx fee was split. Amounts are producer / treasury / burned.
        FeeCharged {
            producer: T::Balance,
            treasury: T::Balance,
            burned: T::Balance,
        },
        /// Treasury ML-DSA lock was updated.
        TreasuryLockSet { lock: [u8; 32] },
        /// A validator's producer lock was registered.
        ProducerLockRegistered {
            account: T::AccountId,
            lock: [u8; 32],
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Fee split percentages do not sum to 100.
        FeeSplitsInvalid,
        /// Producer has not registered a UTXO lock yet.
        ProducerLockNotRegistered,
    }

    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T: Config> {
        pub treasury_lock: [u8; 32],
        #[serde(skip)]
        pub _phantom: sp_std::marker::PhantomData<T>,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            // Validate split config at genesis. Producer + Treasury must be <= 100.
            let p = T::ProducerFeeShare::get() as u32;
            let t = T::TreasuryFeeShare::get() as u32;
            assert!(p + t <= 100, "ProducerFeeShare + TreasuryFeeShare must be <= 100");
            TreasuryLock::<T>::put(self.treasury_lock);
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Root sets the treasury ML-DSA UTXO lock.
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::set_treasury_lock())]
        pub fn set_treasury_lock(origin: OriginFor<T>, lock: [u8; 32]) -> DispatchResult {
            ensure_root(origin)?;
            TreasuryLock::<T>::put(lock);
            Self::deposit_event(Event::TreasuryLockSet { lock });
            Ok(())
        }

        /// Root registers a validator's UTXO lock for fee routing.
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::register_producer_lock())]
        pub fn register_producer_lock(
            origin: OriginFor<T>,
            account: T::AccountId,
            lock: [u8; 32],
        ) -> DispatchResult {
            ensure_root(origin)?;
            ProducerLocks::<T>::insert(&account, lock);
            Self::deposit_event(Event::ProducerLockRegistered { account, lock });
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Compute the fee split for a given amount.
        /// Burn absorbs the remainder, so shares must satisfy producer+treasury <= 100.
        pub fn split(fee: T::Balance) -> (T::Balance, T::Balance, T::Balance) {
            let fee_u128: u128 = fee.into();
            let producer_pct = T::ProducerFeeShare::get() as u128;
            let treasury_pct = T::TreasuryFeeShare::get() as u128;

            let producer_u128 = fee_u128.saturating_mul(producer_pct) / 100;
            let treasury_u128 = fee_u128.saturating_mul(treasury_pct) / 100;
            let burn_u128 = fee_u128.saturating_sub(producer_u128).saturating_sub(treasury_u128);

            (producer_u128.into(), treasury_u128.into(), burn_u128.into())
        }
    }

    impl<T: Config> FeeHandler<T::AccountId, T::Balance> for Pallet<T> {
        fn charge_fee(
            fee: T::Balance,
            producer: Option<T::AccountId>,
        ) -> (T::Balance, T::Balance, T::Balance) {
            let (producer_cut, treasury_cut, burn_cut) = Self::split(fee);

            TreasuryAccumulated::<T>::mutate(|t| *t = t.saturating_add(treasury_cut));
            BurnCounter::<T>::mutate(|b| *b = b.saturating_add(burn_cut));

            // Producer routing is recorded but not yet booked to a UTXO.
            // Phase 8.2 wires this into a producer UTXO at block finalize.
            let _ = producer;

            Self::deposit_event(Event::FeeCharged {
                producer: producer_cut,
                treasury: treasury_cut,
                burned: burn_cut,
            });

            (producer_cut, treasury_cut, burn_cut)
        }

        fn minimum_fee(inputs: u32, outputs: u32) -> T::Balance {
            let base: u128 = T::BaseTxFee::get().into();
            let per: u128 = T::PerInOutFee::get().into();
            let total = base.saturating_add(per.saturating_mul((inputs as u128).saturating_add(outputs as u128)));
            total.into()
        }
    }
}
