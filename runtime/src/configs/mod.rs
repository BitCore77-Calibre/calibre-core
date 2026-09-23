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
	traits::{ConstBool, ConstU128, ConstU32, ConstU64, ConstU8, VariantCountOf},
	weights::{
		constants::{RocksDbWeight, WEIGHT_REF_TIME_PER_SECOND},
		IdentityFee, Weight,
	},
};
use frame_system::limits::{BlockLength, BlockWeights};
use pallet_transaction_payment::{ConstFeeMultiplier, FungibleAdapter, Multiplier};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_runtime::{traits::One, Perbill};
use sp_version::RuntimeVersion;

// Local module imports
use super::{
	AccountId, Aura, Balance, Balances, Block, BlockNumber, Hash, Nonce, PalletInfo, Runtime,
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

/// The default types are being injected by [`derive_impl`](`frame_support::derive_impl`) from
/// [`SoloChainDefaultConfig`](`struct@frame_system::config_preludes::SolochainDefaultConfig`),
/// but overridden as needed.
#[derive_impl(frame_system::config_preludes::SolochainDefaultConfig)]
impl frame_system::Config for Runtime {
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

impl pallet_aura::Config for Runtime {
	type AuthorityId = AuraId;
	type DisabledValidators = ();
	type MaxAuthorities = ConstU32<32>;
	type AllowMultipleBlocksPerSlot = ConstBool<false>;
	type SlotDuration = pallet_aura::MinimumPeriodTimesTwo<Runtime>;
}

impl pallet_grandpa::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;

	type WeightInfo = ();
	type MaxAuthorities = ConstU32<32>;
	type MaxNominators = ConstU32<0>;
	type MaxSetIdSessionEntries = ConstU64<0>;

	type KeyOwnerProof = sp_core::Void;
	type EquivocationReportSystem = ();
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = Aura;
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
}

/// Resolve the Aura block author from the pre-runtime digest.
///
/// Aura's digest carries the slot index; the author is
/// `Authorities[slot % n]`. Wired into calibre-fees as `FindAuthor` so
/// producer payouts route to the block author without needing a separate
/// pallet-authorship instance.
pub struct AuraFindAuthor;
impl frame_support::traits::FindAuthor<AccountId> for AuraFindAuthor {
    fn find_author<'a, I>(digests: I) -> Option<AccountId>
    where
        I: 'a + IntoIterator<Item = (sp_runtime::ConsensusEngineId, &'a [u8])>,
    {
        use frame_support::pallet_prelude::Decode;
        use sp_consensus_aura::AURA_ENGINE_ID;
        use sp_core::crypto::ByteArray;
        for (id, data) in digests {
            if id == AURA_ENGINE_ID {
                let authorities = pallet_aura::Authorities::<Runtime>::get();
                if authorities.is_empty() {
                    return None;
                }
                if let Ok(slot) = u64::decode(&mut &data[..]) {
                    let idx = author_index_for_slot(slot, authorities.len())?;
                    let authority = authorities.get(idx).cloned()?;
                    let bytes: [u8; 32] = authority.to_raw_vec().try_into().ok()?;
                    return Some(sp_runtime::AccountId32::from(bytes).into());
                }
            }
        }
        None
    }
}

/// Pure slot→author-index math used by [`AuraFindAuthor`]. Extracted so it
/// can be unit-tested without spinning up a full runtime.
///
/// Aura rotates authorities round-robin by slot: `authorities[slot % n]`.
/// Returns `None` for an empty authority set (should never happen at
/// runtime, but defensive).
#[inline]
pub(crate) fn author_index_for_slot(slot: u64, n_authorities: usize) -> Option<usize> {
    if n_authorities == 0 {
        return None;
    }
    Some((slot % n_authorities as u64) as usize)
}

#[cfg(test)]
mod aura_find_author_tests {
    use super::author_index_for_slot;

    #[test]
    fn empty_authority_set_returns_none() {
        assert_eq!(author_index_for_slot(0, 0), None);
        assert_eq!(author_index_for_slot(42, 0), None);
    }

    #[test]
    fn single_authority_always_index_zero() {
        assert_eq!(author_index_for_slot(0, 1), Some(0));
        assert_eq!(author_index_for_slot(1, 1), Some(0));
        assert_eq!(author_index_for_slot(999_999, 1), Some(0));
    }

    #[test]
    fn seven_authorities_wrap_round_robin() {
        // Matches the staging validator set size.
        let n = 7;
        assert_eq!(author_index_for_slot(0, n), Some(0));
        assert_eq!(author_index_for_slot(1, n), Some(1));
        assert_eq!(author_index_for_slot(6, n), Some(6));
        assert_eq!(author_index_for_slot(7, n), Some(0), "slot 7 wraps to index 0");
        assert_eq!(author_index_for_slot(8, n), Some(1));
        assert_eq!(author_index_for_slot(14, n), Some(0), "two full rotations");
    }

    #[test]
    fn twenty_one_authorities_wrap_round_robin() {
        // Matches the Phase 9 target validator set size.
        let n = 21;
        assert_eq!(author_index_for_slot(0, n), Some(0));
        assert_eq!(author_index_for_slot(20, n), Some(20));
        assert_eq!(author_index_for_slot(21, n), Some(0));
        assert_eq!(author_index_for_slot(42, n), Some(0));
    }

    #[test]
    fn large_slot_values_do_not_overflow() {
        // Aura slot is u64; ensure modulo is safe at extremes.
        let n = 7;
        assert_eq!(author_index_for_slot(u64::MAX, n), Some((u64::MAX % 7) as usize));
        assert_eq!(author_index_for_slot(u64::MAX - 1, n), Some(((u64::MAX - 1) % 7) as usize));
    }
}

impl pallet_stake::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type MaxBondInputs = ConstU32<16>;
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
	type FindAuthor = AuraFindAuthor;
	type FeeMinter = crate::Qutxo;
	type WeightInfo = pallet_calibre_fees::weights::SubstrateWeight<Runtime>;
}

// ─────────────────────────────────────────────────────────────
// Session pallet — key rotation and validator set updates
// ─────────────────────────────────────────────────────────────
//
// Phase 9.2: pallet-session is wired but the authority set does not
// rotate yet. `SessionManager = ()` returns `None` from new_session,
// which means "keep the same set". Session *keys* are registered and
// rotate at session boundaries; the set itself stays genesis-frozen
// until Phase 9.3 (epochs) and 9.5 (stake-weighted election).
//
// Session length: 600 blocks = 1 hour at 6s block time.

parameter_types! {
	pub const SessionPeriod: BlockNumber = 600;
	pub const SessionOffset: BlockNumber = 0;
}

impl pallet_session::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ValidatorId = AccountId;
	type ValidatorIdOf = sp_runtime::traits::ConvertInto;
	type ShouldEndSession = pallet_session::PeriodicSessions<SessionPeriod, SessionOffset>;
	type NextSessionRotation = pallet_session::PeriodicSessions<SessionPeriod, SessionOffset>;
	// No election yet. Same set persists; only session keys rotate.
	type SessionManager = ();
	// Aura + GRANDPA both implement SessionHandler for their key types.
	type SessionHandler =
		<SessionKeys as sp_runtime::traits::OpaqueKeys>::KeyTypeIdProviders;
	type Keys = SessionKeys;
	// No validators are disabled in this phase.
	type DisablingStrategy = ();
	type WeightInfo = ();
}
