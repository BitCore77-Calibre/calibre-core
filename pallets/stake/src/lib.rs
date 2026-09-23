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

        /// Minimum stake required to join the candidate registry. Below this
        /// amount, `join` fails. Governance-adjustable.
        #[pallet::constant]
        type MinValidatorStake: Get<Self::Balance>;

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

    /// Account IDs that have opted in to validator election. Election
    /// considers only accounts in this set with a positive `Stake` balance.
    /// `join` and `leave` maintain it. Bounded iteration — one entry per
    /// declared candidate, not per staker.
    #[pallet::storage]
    #[pallet::getter(fn candidates)]
    pub type Candidates<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, (), OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        Bonded { who: T::AccountId, amount: T::Balance },
        Unbonded { who: T::AccountId, amount: T::Balance, lock: [u8; 32] },
        Joined { who: T::AccountId, stake: T::Balance },
        Left { who: T::AccountId },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Unbond amount exceeds the account's bonded balance.
        InsufficientStake,
        /// Too many inputs in a single bond call.
        BondTooManyInputs,
        /// The UTXO consumer rejected the inputs or witness.
        UtxoConsumeFailed,
        /// Stake is below the minimum required to be a validator candidate.
        InsufficientStakeForCandidate,
        /// Caller is already a candidate.
        AlreadyCandidate,
        /// Caller is not a candidate.
        NotCandidate,
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

        /// Opt in to validator election. Caller must hold at least
        /// `MinValidatorStake` bonded. Idempotent failure if already joined.
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::bond())]  // same shape as bond: O(1) write
        pub fn join_candidates(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                !Candidates::<T>::contains_key(&who),
                Error::<T>::AlreadyCandidate
            );
            let stake = Stake::<T>::get(&who);
            ensure!(
                stake >= T::MinValidatorStake::get(),
                Error::<T>::InsufficientStakeForCandidate
            );
            Candidates::<T>::insert(&who, ());
            Self::deposit_event(Event::Joined { who, stake });
            Ok(())
        }

        /// Withdraw from the candidate registry. Fails if not a candidate.
        /// Does not require unbonding first — stake can remain and be
        /// re-joined later.
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::bond())]
        pub fn leave_candidates(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                Candidates::<T>::contains_key(&who),
                Error::<T>::NotCandidate
            );
            Candidates::<T>::remove(&who);
            Self::deposit_event(Event::Left { who });
            Ok(())
        }
    }

    // ── Election helpers (not extrinsics) ──
    impl<T: Config> Pallet<T> {
        /// Return up to `n` candidates with the highest `Stake` balance.
        /// Only candidates with positive stake are considered.
        pub fn elect_top_n(n: u32) -> sp_std::vec::Vec<T::AccountId> {
            let mut ranked: sp_std::vec::Vec<(T::AccountId, T::Balance)> =
                Candidates::<T>::iter()
                    .filter_map(|(who, _)| {
                        let stake = Stake::<T>::get(&who);
                        if stake > T::Balance::default() {
                            Some((who, stake))
                        } else {
                            None
                        }
                    })
                    .collect();
            ranked.sort_by(|a, b| b.1.cmp(&a.1));
            ranked.into_iter().take(n as usize).map(|(who, _)| who).collect()
        }

        /// Current candidate count. Stats only.
        pub fn candidate_count() -> u32 {
            Candidates::<T>::iter().count() as u32
        }
    }
}
