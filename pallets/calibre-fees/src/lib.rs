#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

#[cfg(test)]
mod tests;

pub mod weights;
pub use weights::WeightInfo;

#[frame_support::pallet]
pub mod pallet {
    use crate::WeightInfo;
    use calibre_primitives::{FeeHandler, FeeMinter};
    use frame_support::traits::FindAuthor;
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

        /// Target block fullness as a percent of max block weight (e.g. 50).
        /// Blocks above this raise the base fee; blocks below lower it.
        #[pallet::constant]
        type TargetBlockFullnessPct: Get<u8>;

        /// Maximum base fee change per block, in percent (e.g. 12 = 12%).
        #[pallet::constant]
        type MaxBaseFeeChangePct: Get<u8>;

        /// Minimum base fee. Base fee will never decay below this.
        #[pallet::constant]
        type MinBaseFee: Get<Self::Balance>;

        /// Maximum base fee. Base fee will never rise above this.
        #[pallet::constant]
        type MaxBaseFee: Get<Self::Balance>;

        /// Block reward minted per block. Split between producer and treasury
        /// by the same percentages used for fees (ProducerFeeShare /
        /// TreasuryFeeShare). Burn share is not applied to rewards — all
        /// reward value is distributed.
        #[pallet::constant]
        type BlockRewardPerBlock: Get<Self::Balance>;

        /// Percent of the fee routed to the block producer (0-100).
        #[pallet::constant]
        type ProducerFeeShare: Get<u8>;

        /// Percent of the fee routed to the treasury (0-100).
        #[pallet::constant]
        type TreasuryFeeShare: Get<u8>;

        /// Resolves the block author for producer payout routing.
        type FindAuthor: frame_support::traits::FindAuthor<Self::AccountId>;

        /// Mints the settled producer payout as a UTXO. pallet-qutxo in the
        /// runtime; `()` in tests.
        type FeeMinter: calibre_primitives::FeeMinter<Self::Balance, [u8; 32]>;

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

    /// Producer's accumulated base fee cut (from 50/30/20 split). Minted as UTXO in on_finalize.
    #[pallet::storage]
    #[pallet::getter(fn producer_accumulated)]
    pub type ProducerAccumulated<T: Config> = StorageValue<_, T::Balance, ValueQuery>;

    /// Priority fee accumulator (100% to producer). Minted as UTXO in on_finalize.
    #[pallet::storage]
    #[pallet::getter(fn priority_fee_accumulated)]
    pub type PriorityFeeAccumulated<T: Config> = StorageValue<_, T::Balance, ValueQuery>;

    /// Current dynamic base fee. Adjusted in `on_finalize` based on the
    /// previous block's consumed weight. Read by `minimum_fee`.
    #[pallet::storage]
    #[pallet::getter(fn current_base_fee)]
    pub type CurrentBaseFee<T: Config> = StorageValue<_, T::Balance, ValueQuery>;

    /// Cumulative block rewards minted since genesis. Stats only.
    #[pallet::storage]
    #[pallet::getter(fn total_rewards_minted)]
    pub type TotalRewardsMinted<T: Config> = StorageValue<_, T::Balance, ValueQuery>;

    /// Block author captured in `on_initialize`. Drives producer payout routing.
    #[pallet::storage]
    pub type CurrentAuthor<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

    /// Producer payouts awaiting a registered lock (Q2(a): never burned).
    /// Bounded by validator-set size. Cleared on successful mint.
    #[pallet::storage]
    #[pallet::getter(fn producer_pending)]
    pub type ProducerPending<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, T::Balance, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A tx fee was split. Amounts are producer / treasury / burned.
        FeeCharged {
            producer: T::Balance,
            treasury: T::Balance,
            burned: T::Balance,
        },
        /// Priority fee charged (100% to producer).
        PriorityFeeCharged { amount: T::Balance },
        /// Block reward was minted and split.
        BlockRewardMinted {
            producer: T::Balance,
            treasury: T::Balance,
        },
        /// Treasury ML-DSA lock was updated.
        TreasuryLockSet { lock: [u8; 32] },
        /// A validator's producer lock was registered.
        ProducerLockRegistered {
            account: T::AccountId,
            lock: [u8; 32],
        },
        /// Producer payout settled and minted as a UTXO.
        ProducerPaid { account: T::AccountId, amount: T::Balance },
        /// Producer payout deferred — no lock registered yet. Held pending.
        ProducerPayoutDeferred { account: T::AccountId, amount: T::Balance },
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
            // Seed the dynamic base fee from the genesis constant.
            CurrentBaseFee::<T>::put(T::BaseTxFee::get());
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(_n: BlockNumberFor<T>) -> Weight {
            let author = T::FindAuthor::find_author(
                frame_system::Pallet::<T>::digest()
                    .logs
                    .iter()
                    .filter_map(|d| d.as_pre_runtime()),
            );
            CurrentAuthor::<T>::set(author);
            T::DbWeight::get().reads_writes(1, 1)
        }

        fn on_finalize(_n: BlockNumberFor<T>) {
            Self::adjust_base_fee();
            Self::distribute_block_reward();
            Self::settle_producer_payouts();
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
        /// Mint and split the block reward. Called in `on_finalize`.
        ///
        /// Producer share is recorded via the same accounting used for fees,
        /// but routed to `ProducerFeeShare` instead of being tracked as a cut.
        /// Phase 8.2b wires the actual UTXO creation via pallet-authorship.
        pub fn distribute_block_reward() {
            let reward = T::BlockRewardPerBlock::get();
            if reward == T::Balance::default() {
                return;
            }

            let reward_u128: u128 = reward.into();
            let treasury_pct = T::TreasuryFeeShare::get() as u128;

            let treasury_u128 = reward_u128.saturating_mul(treasury_pct) / 100;

            // Remaining % stays as producer cut in practice — all reward value
            // is distributed (no burn on issuance).
            let producer_total_u128 = reward_u128.saturating_sub(treasury_u128);

            let treasury_cut: T::Balance = treasury_u128.into();
            TreasuryAccumulated::<T>::mutate(|t| *t = t.saturating_add(treasury_cut));
            TotalRewardsMinted::<T>::mutate(|m| *m = m.saturating_add(reward));

            let producer_cut: T::Balance = producer_total_u128.into();
            ProducerAccumulated::<T>::mutate(|p| *p = p.saturating_add(producer_cut));

            Self::deposit_event(Event::BlockRewardMinted {
                producer: producer_cut,
                treasury: treasury_cut,
            });
        }

        /// Adjust the dynamic base fee based on the current block's weight.
        /// Called in `on_finalize`; sets the fee for the *next* block.
        pub fn adjust_base_fee() {
            let max_block = <T as frame_system::Config>::BlockWeights::get().max_block.ref_time();
            if max_block == 0 {
                return;
            }

            let actual = frame_system::Pallet::<T>::block_weight().total().ref_time();
            let target_pct = T::TargetBlockFullnessPct::get() as u64;
            let target = max_block.saturating_mul(target_pct) / 100;
            if target == 0 {
                return;
            }

            let current: u128 = CurrentBaseFee::<T>::get().into();
            let step_pct = T::MaxBaseFeeChangePct::get() as u128;

            let new_u128: u128 = if actual > target {
                // Block over target: raise, scaled by how far over.
                let over = (actual - target) as u128;
                let over_ratio = (over.saturating_mul(100) / (target as u128)).min(100);
                let change = step_pct.saturating_mul(over_ratio) / 100;
                current.saturating_mul(100u128.saturating_add(change)) / 100
            } else if actual < target {
                // Block under target: lower, scaled by how far under.
                let under = (target - actual) as u128;
                let under_ratio = (under.saturating_mul(100) / (target as u128)).min(100);
                let change = step_pct.saturating_mul(under_ratio) / 100;
                current.saturating_mul(100u128.saturating_sub(change)) / 100
            } else {
                current
            };

            let floor: u128 = T::MinBaseFee::get().into();
            let ceil: u128 = T::MaxBaseFee::get().into();
            let clamped = new_u128.max(floor).min(ceil);

            CurrentBaseFee::<T>::put(T::Balance::from(clamped));
        }

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

    impl<T: Config> FeeHandler<T::Balance> for Pallet<T> {
        fn charge_fee(fee: T::Balance) -> (T::Balance, T::Balance, T::Balance) {
            let (producer_cut, treasury_cut, burn_cut) = Self::split(fee);

            TreasuryAccumulated::<T>::mutate(|t| *t = t.saturating_add(treasury_cut));
            BurnCounter::<T>::mutate(|b| *b = b.saturating_add(burn_cut));
            ProducerAccumulated::<T>::mutate(|p| *p = p.saturating_add(producer_cut));

            Self::deposit_event(Event::FeeCharged {
                producer: producer_cut,
                treasury: treasury_cut,
                burned: burn_cut,
            });

            (producer_cut, treasury_cut, burn_cut)
        }

        fn minimum_fee(inputs: u32, outputs: u32) -> T::Balance {
            let base: u128 = CurrentBaseFee::<T>::get().into();
            let per: u128 = T::PerInOutFee::get().into();
            let total = base
                .saturating_add(per.saturating_mul((inputs as u128).saturating_add(outputs as u128)));
            total.into()
        }

        fn charge_priority_fee(fee: T::Balance) -> T::Balance {
            if fee == T::Balance::default() {
                return fee;
            }
            PriorityFeeAccumulated::<T>::mutate(|p| *p = p.saturating_add(fee));
            Self::deposit_event(Event::PriorityFeeCharged { amount: fee });
            fee
        }
    }

    impl<T: Config> Pallet<T> {
        /// Settle all producer earnings for this block into a single UTXO.
        ///
        /// Drains the two global accumulators (`ProducerAccumulated` from the
        /// 50/30/20 split and block reward, `PriorityFeeAccumulated` from
        /// priority tips) plus any carried `ProducerPending` for the author.
        ///
        /// Q2(a): if the author has no registered lock, the whole amount is
        /// consolidated back into `ProducerPending[author]` and minted on a
        /// later block once `register_producer_lock` has been called. Value is
        /// never burned. Pending is keyed by account; the lock used at mint
        /// time is whatever `ProducerLocks[author]` currently holds.
        ///
        /// Treasury stays accounting-only — no mint path for it in 8.6.
        pub fn settle_producer_payouts() {
            let author = match CurrentAuthor::<T>::get() {
                Some(a) => a,
                // No resolvable author this block: leave accumulators intact.
                None => return,
            };

            let producer = ProducerAccumulated::<T>::take();
            let priority = PriorityFeeAccumulated::<T>::take();
            let carried = ProducerPending::<T>::get(&author);
            let total = producer.saturating_add(priority).saturating_add(carried);
            if total == T::Balance::default() {
                return;
            }

            match ProducerLocks::<T>::get(&author) {
                Some(lock) => {
                    T::FeeMinter::mint_to_lock(total, lock);
                    ProducerPending::<T>::remove(&author);
                    Self::deposit_event(Event::ProducerPaid { account: author, amount: total });
                }
                None => {
                    ProducerPending::<T>::insert(&author, total);
                    Self::deposit_event(Event::ProducerPayoutDeferred {
                        account: author,
                        amount: total,
                    });
                }
            }
        }
    }
}
