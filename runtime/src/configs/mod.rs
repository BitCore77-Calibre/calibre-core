// This is free and unencumbered software released into the public domain.
//
// Anyone is free to copy, modify, publish, use, compile, sell, or
// distribute this software, either in source code form or as a compiled
// binary, for any purpose, commercial or non-commercial, and by any
// means.
//
// In jurisdictions that recognize copyright laws, the author or authors
// of this software dedicate any and all copyright interest in the
// software to the public domain. We make this dedication for the benefit
// of the public at large and to the detriment of our heirs and
// successors. We intend this dedication to be an overt act of
// relinquishment in perpetuity of all present and future rights to this
// software under copyright law.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
// IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY CLAIM, DAMAGES OR
// OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE,
// ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
// OTHER DEALINGS IN THE SOFTWARE.
//
// For more information, please refer to <http://unlicense.org>

// Substrate and Polkadot dependencies
use frame_support::{
    derive_impl, parameter_types,
    traits::{ConstU128, ConstU32, ConstU64, ConstU8, Contains, VariantCountOf},
    weights::{
        constants::{RocksDbWeight, WEIGHT_REF_TIME_PER_SECOND},
        IdentityFee, Weight,
    },
};
use frame_system::limits::{BlockLength, BlockWeights};
use pallet_transaction_payment::{ConstFeeMultiplier, FungibleAdapter, Multiplier};
use sp_runtime::{traits::One, Perbill};
use sp_version::RuntimeVersion;

// Local module imports
use super::{
    AccountId, Babe, Balance, Balances, Block, BlockNumber, Hash, Nonce, PalletInfo, Runtime,
    RuntimeCall, RuntimeEvent, RuntimeFreezeReason, RuntimeHoldReason, RuntimeOrigin, RuntimeTask,
    SessionKeys, System, EXISTENTIAL_DEPOSIT, SLOT_DURATION, VERSION,
};

const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

parameter_types! {
    pub const BlockHashCount: BlockNumber = 2400;
    pub const Version: RuntimeVersion = VERSION;

    /// We allow for 2 seconds of compute with a 6 second average block time.
    pub RuntimeBlockWeights: BlockWeights = BlockWeights::with_sensible_defaults(
        Weight::from_parts(2u64 * WEIGHT_REF_TIME_PER_SECOND, u64::MAX),
        NORMAL_DISPATCH_RATIO,
    );
    pub RuntimeBlockLength: BlockLength = BlockLength::max_with_normal_ratio(20 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
    pub const SS58Prefix: u8 = 42;
}

/// Until validator exits are coordinated with queued session keys, any signed
/// `purge_keys` could remove an active authority at the next rotation. Keep
/// registrations and key replacement available, but reject key purges.
pub struct SessionKeyLivenessFilter;

impl Contains<RuntimeCall> for SessionKeyLivenessFilter {
    fn contains(call: &RuntimeCall) -> bool {
        !matches!(
            call,
            RuntimeCall::Session(pallet_session::Call::purge_keys { .. })
        )
    }
}

/// The default types are being injected by [`derive_impl`](`frame_support::derive_impl`) from
/// [`SoloChainDefaultConfig`](`struct@frame_system::config_preludes::SolochainDefaultConfig`),
/// but overridden as needed.
#[derive_impl(frame_system::config_preludes::SolochainDefaultConfig)]
impl frame_system::Config for Runtime {
    type BaseCallFilter = SessionKeyLivenessFilter;
    /// The block type for the runtime.
    type Block = Block;
    /// Block & extrinsics weights: base values and limits.
    type BlockWeights = RuntimeBlockWeights;
    /// The maximum length of a block (in bytes).
    type BlockLength = RuntimeBlockLength;
    /// The identifier used to distinguish between accounts.
    type AccountId = AccountId;
    /// The type for storing how many extrinsics an account has signed.
    type Nonce = Nonce;
    /// The type for hashing blocks and tries.
    type Hash = Hash;
    /// Maximum number of block number to block hash mappings to keep (oldest pruned first).
    type BlockHashCount = BlockHashCount;
    /// The weight of database operations that the runtime can invoke.
    type DbWeight = RocksDbWeight;
    /// Version of the runtime.
    type Version = Version;
    /// The data to be stored in an account.
    type AccountData = pallet_balances::AccountData<Balance>;
    /// This is used as an identifier of the chain. 42 is the generic substrate prefix.
    type SS58Prefix = SS58Prefix;
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
    /// BABE epochs are driven by pallet-session, so both boundaries must match.
    pub const EpochDuration: u64 = 600;
    pub const ExpectedBlockTime: u64 = 6000;
}

impl pallet_babe::Config for Runtime {
    type EpochDuration = EpochDuration;
    type ExpectedBlockTime = ExpectedBlockTime;
    type EpochChangeTrigger = pallet_babe::ExternalTrigger;
    type DisabledValidators = ();
    type MaxAuthorities = ConstU32<32>;
    type MaxNominators = ConstU32<0>;
    type KeyOwnerProof = sp_core::Void;
    type EquivocationReportSystem = ();
    type WeightInfo = ();
}

impl pallet_grandpa::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;

    type WeightInfo = ();
    type MaxAuthorities = ConstU32<32>;
    type MaxNominators = ConstU32<0>;
    type MaxSetIdSessionEntries = ConstU64<600>;

    type KeyOwnerProof = sp_core::Void;
    type EquivocationReportSystem = ();
}

impl pallet_timestamp::Config for Runtime {
    /// A timestamp: milliseconds since the unix epoch.
    type Moment = u64;
    type OnTimestampSet = Babe;
    type MinimumPeriod = ConstU64<{ SLOT_DURATION / 2 }>;
    type WeightInfo = ();
}

impl pallet_balances::Config for Runtime {
    type MaxLocks = ConstU32<50>;
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    /// The type for recording an account's balance.
    type Balance = Balance;
    /// The ubiquitous event type.
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU128<EXISTENTIAL_DEPOSIT>;
    type AccountStore = System;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
    type FreezeIdentifier = RuntimeFreezeReason;
    type MaxFreezes = VariantCountOf<RuntimeFreezeReason>;
    type RuntimeHoldReason = RuntimeHoldReason;
    type RuntimeFreezeReason = RuntimeFreezeReason;
    type DoneSlashHandler = ();
}

parameter_types! {
    pub FeeMultiplier: Multiplier = Multiplier::one();
}

impl pallet_transaction_payment::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type OnChargeTransaction = FungibleAdapter<Balances, ()>;
    type OperationalFeeMultiplier = ConstU8<5>;
    type WeightToFee = IdentityFee<Balance>;
    type LengthToFee = IdentityFee<Balance>;
    type FeeMultiplierUpdate = ConstFeeMultiplier<FeeMultiplier>;
    type WeightInfo = pallet_transaction_payment::weights::SubstrateWeight<Runtime>;
}

impl pallet_sudo::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type WeightInfo = pallet_sudo::weights::SubstrateWeight<Runtime>;
}

/// Configure the pallet-template in pallets/template.
impl pallet_template::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = pallet_template::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    /// Minimum base fee per UTXO tx (in smallest CAL unit).
    pub const BaseTxFee: Balance = 1_000;
    /// Additional fee per input + per output.
    pub const PerInOutFee: Balance = 100;
    /// Percentage of fee routed to the block producer.
    pub const ProducerFeeShare: u8 = 50;
    /// Percentage of fee routed to the treasury.
    pub const TreasuryFeeShare: u8 = 30;
    /// Target block fullness: 50% of max weight. Above → fee rises.
    pub const TargetBlockFullnessPct: u8 = 50;
    /// Max change per block: 12% (EIP-1559-flavored).
    pub const MaxBaseFeeChangePct: u8 = 12;
    /// Floor: base fee will never decay below this.
    pub const MinBaseFee: Balance = 500;
    /// Ceiling: base fee will never rise above this.
    pub const MaxBaseFee: Balance = 1_000_000;
    /// Block reward minted per block: 10 CAL = 10_000_000_000_000_000_000 units (18 decimals).
    /// Tunable down over time by governance.
    pub const BlockRewardPerBlock: Balance = 5_300_000_000_000_000_000u128;
    /// Treasury accumulates below this and doesn't mint (1,000 CAL = 1e21 units).
    /// Keeps UTXO set clean — one treasury UTXO per ~60 blocks at launch.
    pub const MinTreasurySettle: Balance = 1_000_000_000_000_000_000_000;
    /// Minimum stake to become a validator candidate. 1,000 CAL.
    pub const MinValidatorStake: Balance = 1_000_000_000_000_000_000_000;
}

/// Decode BABE's pre-runtime digest to an authority index, then resolve that
/// index through pallet-session's active validator list. This keeps producer
/// payouts tied to the same account IDs that own the session keys.
pub type BabeFindAuthor = pallet_session::FindAccountFromAuthorIndex<Runtime, Babe>;

impl pallet_stake::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Balance = Balance;
    type MaxBondInputs = ConstU32<16>;
    type MinValidatorStake = MinValidatorStake;
    type UtxoConsumer = crate::Qutxo;
    type UtxoMinter = crate::Qutxo;
    type WeightInfo = pallet_stake::weights::SubstrateWeight<Runtime>;
}

impl pallet_calibre_fees::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Balance = Balance;
    type BaseTxFee = BaseTxFee;
    type PerInOutFee = PerInOutFee;
    type ProducerFeeShare = ProducerFeeShare;
    type TreasuryFeeShare = TreasuryFeeShare;
    type TargetBlockFullnessPct = TargetBlockFullnessPct;
    type MaxBaseFeeChangePct = MaxBaseFeeChangePct;
    type MinBaseFee = MinBaseFee;
    type MaxBaseFee = MaxBaseFee;
    type BlockRewardPerBlock = BlockRewardPerBlock;
    type MinTreasurySettle = MinTreasurySettle;
    type FindAuthor = BabeFindAuthor;
    type FeeMinter = crate::Qutxo;
    type WeightInfo = pallet_calibre_fees::weights::SubstrateWeight<Runtime>;
}

// ─────────────────────────────────────────────────────────────
// Session pallet — key rotation and validator set updates
// ─────────────────────────────────────────────────────────────
//
// pallet-session owns BABE and GRANDPA authority keys. Session keys rotate at
// each boundary; CalibreSessionManager keeps the active validator set unchanged
// except at the configured stake-election boundaries below.
//
// Session length: 600 BABE slots, approximately 1 hour at 6s per slot.

// ─────────────────────────────────────────────────────────────
// Epoch-driven session manager (Phase 9.3)
// ─────────────────────────────────────────────────────────────
//
// BABE/session epochs rotate every 600 blocks (about 1 hour). Validator
// elections run every 6 BABE epochs (about 6 hours). At each election boundary, the validator set is
// re-elected from the candidate registry in pallet-stake — top-N by
// stake. Between epoch boundaries, the set is left unchanged.
//
// `SessionManager::new_session(N)` plans the set for session N.
// Returning `None` means "keep current set". Returning `Some(vec)`
// makes that vec active starting at session N.
//
// Safety:
//   - Session 0 (genesis) returns `None`. The genesis-set authorities
//     (from pallet_babe::Authorities and pallet_grandpa::Authorities)
//     remain in place.
//   - Keep the existing set unless a full replacement has registered keys.
//     Session filters missing keys, so a partial set could otherwise reduce
//     BABE authors and GRANDPA voters without warning.

parameter_types! {
    /// BABE/session epochs per validator election cycle.
    pub const EpochDurationInSessions: u32 = 6;
    /// Maximum active validators elected per epoch.
    pub const MaxActiveValidators: u32 = 7;
}

/// Return true if the given session index is an epoch boundary —
/// i.e. the first session of a new epoch. Session 0 is the genesis
/// boundary and does NOT trigger rotation (the genesis set is used).
#[inline]
pub(crate) fn is_epoch_boundary(session_index: u32, epoch_len: u32) -> bool {
    if epoch_len == 0 {
        return false;
    }
    session_index != 0 && session_index % epoch_len == 0
}

/// Epoch-driven validator election. Reads `pallet-stake`'s candidate
/// registry at epoch boundaries; returns the top-N by stake.
pub struct CalibreSessionManager;

impl pallet_session::SessionManager<AccountId> for CalibreSessionManager {
    fn new_session(index: sp_staking::SessionIndex) -> Option<alloc::vec::Vec<AccountId>> {
        // Genesis or mid-epoch: keep current set.
        let epoch_len = EpochDurationInSessions::get();
        if !is_epoch_boundary(index, epoch_len) {
            return None;
        }

        // Epoch boundary — re-elect from the candidate registry.
        let elected = crate::Stake::elect_top_n(MaxActiveValidators::get());

        // Session silently drops elected accounts without NextKeys. Do not
        // replace the seven-authority set with fewer usable authorities.
        if elected.len() != MaxActiveValidators::get() as usize
            || elected
                .iter()
                .any(|validator| !pallet_session::NextKeys::<Runtime>::contains_key(validator))
        {
            return None;
        }

        Some(elected)
    }

    fn start_session(_start_index: sp_staking::SessionIndex) {}

    fn end_session(_end_index: sp_staking::SessionIndex) {}
}

impl pallet_session::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type ValidatorId = AccountId;
    type ValidatorIdOf = sp_runtime::traits::ConvertInto;
    // BABE owns the slot-based epoch boundary. A block-count schedule would
    // drift whenever a slot is empty and can desynchronize consensus epochs.
    type ShouldEndSession = Babe;
    type NextSessionRotation = Babe;
    type SessionManager = CalibreSessionManager;
    // BABE + GRANDPA both implement SessionHandler for their key types.
    type SessionHandler = <SessionKeys as sp_runtime::traits::OpaqueKeys>::KeyTypeIdProviders;
    type Keys = SessionKeys;
    // No validators are disabled in this phase.
    type DisablingStrategy = ();
    type WeightInfo = ();
}

#[cfg(test)]
mod session_rotation_tests {
    use super::{is_epoch_boundary, CalibreSessionManager};
    use crate::{AccountId, Runtime, SessionKeys};
    use pallet_session::SessionManager;
    use sp_keyring::{Ed25519Keyring, Sr25519Keyring};

    fn register_candidates(count: usize, keys: usize) -> Vec<AccountId> {
        let identities = [
            (Sr25519Keyring::Alice, Ed25519Keyring::Alice),
            (Sr25519Keyring::Bob, Ed25519Keyring::Bob),
            (Sr25519Keyring::Charlie, Ed25519Keyring::Charlie),
            (Sr25519Keyring::Dave, Ed25519Keyring::Dave),
            (Sr25519Keyring::Eve, Ed25519Keyring::Eve),
            (Sr25519Keyring::Ferdie, Ed25519Keyring::Ferdie),
            (Sr25519Keyring::One, Ed25519Keyring::One),
        ];
        identities
            .into_iter()
            .take(count)
            .enumerate()
            .map(|(index, (sr, ed))| {
                let account = sr.to_account_id();
                pallet_stake::Stake::<Runtime>::insert(
                    &account,
                    1_000_000_000_000_000_000_000u128 + index as u128,
                );
                pallet_stake::Candidates::<Runtime>::insert(&account, ());
                if index < keys {
                    pallet_session::NextKeys::<Runtime>::insert(
                        &account,
                        SessionKeys {
                            babe: sr.public().into(),
                            grandpa: ed.public().into(),
                        },
                    );
                }
                account
            })
            .collect()
    }

    #[test]
    fn election_keeps_current_set_when_only_six_candidates_exist() {
        sp_io::TestExternalities::default().execute_with(|| {
            register_candidates(6, 6);
            assert_eq!(CalibreSessionManager::new_session(6), None);
        });
    }

    #[test]
    fn election_keeps_current_set_when_a_candidate_lacks_keys() {
        sp_io::TestExternalities::default().execute_with(|| {
            register_candidates(7, 6);
            assert_eq!(CalibreSessionManager::new_session(6), None);
        });
    }

    #[test]
    fn election_accepts_seven_candidates_with_keys_at_boundary() {
        sp_io::TestExternalities::default().execute_with(|| {
            let accounts = register_candidates(7, 7);
            assert_eq!(CalibreSessionManager::new_session(5), None);
            assert_eq!(
                CalibreSessionManager::new_session(6),
                Some(accounts.into_iter().rev().collect())
            );
        });
    }

    #[test]
    fn session_zero_is_not_a_boundary() {
        // Genesis set is used at session 0; no rotation.
        assert!(!is_epoch_boundary(0, 6));
    }

    #[test]
    fn sessions_within_first_epoch_are_not_boundaries() {
        for s in 1..6 {
            assert!(!is_epoch_boundary(s, 6), "session {} should not rotate", s);
        }
    }

    #[test]
    fn session_six_is_boundary() {
        assert!(is_epoch_boundary(6, 6));
    }

    #[test]
    fn session_twelve_is_boundary() {
        assert!(is_epoch_boundary(12, 6));
    }

    #[test]
    fn session_eighteen_is_boundary() {
        assert!(is_epoch_boundary(18, 6));
    }

    #[test]
    fn mid_epoch_sessions_do_not_rotate() {
        for s in [7u32, 8, 9, 10, 11, 13, 14, 15, 16, 17] {
            assert!(!is_epoch_boundary(s, 6), "session {} should not rotate", s);
        }
    }

    #[test]
    fn zero_epoch_len_is_safe() {
        // Defensive: no rotation if epoch length is somehow 0.
        assert!(!is_epoch_boundary(0, 0));
        assert!(!is_epoch_boundary(6, 0));
        assert!(!is_epoch_boundary(12, 0));
    }

    #[test]
    fn epoch_len_one_rotates_every_session_except_zero() {
        // Pathological config: rotate every session.
        assert!(!is_epoch_boundary(0, 1));
        assert!(is_epoch_boundary(1, 1));
        assert!(is_epoch_boundary(2, 1));
        assert!(is_epoch_boundary(100, 1));
    }
}

#[cfg(test)]
mod session_integration_tests;
