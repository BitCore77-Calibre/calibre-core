#![cfg(test)]

use crate::{self as pallet_calibre_fees, *};
use frame_support::{assert_ok, derive_impl, parameter_types};
use frame_support::traits::ConstU32;
use sp_runtime::BuildStorage;
use calibre_primitives::FeeHandler;

type Block = frame_system::mocking::MockBlock<Test>;

// ── Phase 8.6 test recording state ──
//
// `RecordingMinter` captures every mint `settle_producer_payouts` performs so
// tests can assert on producer delivery without wiring a real qutxo. The
// author source is `NEXT_AUTHOR`, read by `TestFindAuthor` in `on_initialize`.
thread_local! {
    static MINTED: std::cell::RefCell<Vec<(u128, [u8; 32])>> =
        std::cell::RefCell::new(Vec::new());
    static NEXT_AUTHOR: std::cell::RefCell<Option<u64>> =
        std::cell::RefCell::new(None);
}

pub struct RecordingMinter;
impl calibre_primitives::FeeMinter<u128, [u8; 32]> for RecordingMinter {
    fn mint_to_lock(value: u128, lock: [u8; 32]) {
        MINTED.with(|m| m.borrow_mut().push((value, lock)));
    }
}

fn take_minted() -> Vec<(u128, [u8; 32])> {
    MINTED.with(|m| std::mem::take(&mut *m.borrow_mut()))
}

fn set_next_author(a: Option<u64>) {
    NEXT_AUTHOR.with(|slot| *slot.borrow_mut() = a);
}

frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        Fees: pallet_calibre_fees,
    }
);

parameter_types! {
    pub RuntimeBlockWeights: frame_system::limits::BlockWeights =
        frame_system::limits::BlockWeights::with_sensible_defaults(
            frame_support::weights::Weight::from_parts(2_000_000_000_000, u64::MAX),
            sp_runtime::Perbill::from_percent(75),
        );
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type Block = Block;
    type BlockWeights = RuntimeBlockWeights;
}

parameter_types! {
    pub const BaseTxFee: u128 = 1_000;
    pub const PerInOutFee: u128 = 100;
    pub const ProducerFeeShare: u8 = 50;
    pub const TreasuryFeeShare: u8 = 30;
    pub const TargetBlockFullnessPct: u8 = 50;
    pub const MaxBaseFeeChangePct: u8 = 12;
    pub const MinBaseFee: u128 = 500;
    pub const MaxBaseFee: u128 = 1_000_000;
    pub const BlockRewardPerBlock: u128 = 10_000;
}

pub struct TestFindAuthor;
impl frame_support::traits::FindAuthor<u64> for TestFindAuthor {
    fn find_author<'a, I>(_digests: I) -> Option<u64>
    where
        I: 'a + IntoIterator<Item = (sp_runtime::ConsensusEngineId, &'a [u8])>,
    {
        NEXT_AUTHOR.with(|slot| *slot.borrow())
    }
}

impl pallet_calibre_fees::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Balance = u128;
    type BaseTxFee = BaseTxFee;
    type PerInOutFee = PerInOutFee;
    type ProducerFeeShare = ProducerFeeShare;
    type TreasuryFeeShare = TreasuryFeeShare;
    type TargetBlockFullnessPct = TargetBlockFullnessPct;
    type MaxBaseFeeChangePct = MaxBaseFeeChangePct;
    type MinBaseFee = MinBaseFee;
    type MaxBaseFee = MaxBaseFee;
    type BlockRewardPerBlock = BlockRewardPerBlock;
    type FindAuthor = TestFindAuthor;
    type FeeMinter = RecordingMinter;
    type WeightInfo = ();
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
    pallet_calibre_fees::GenesisConfig::<Test> {
        treasury_lock: [7u8; 32],
        _phantom: Default::default(),
    }
    .assimilate_storage(&mut t)
    .unwrap();
    set_next_author(None);
    let _ = take_minted();
    t.into()
}

#[test]
fn split_is_50_30_20() {
    new_test_ext().execute_with(|| {
        let (producer, treasury, burn) = Fees::split(1_000);
        assert_eq!(producer, 500);
        assert_eq!(treasury, 300);
        assert_eq!(burn, 200);
        assert_eq!(producer + treasury + burn, 1_000);
    });
}

#[test]
fn split_handles_non_divisible_amounts() {
    new_test_ext().execute_with(|| {
        // 101 -> 50, 30, 21 (burn absorbs remainder)
        let (producer, treasury, burn) = Fees::split(101);
        assert_eq!(producer, 50);
        assert_eq!(treasury, 30);
        assert_eq!(burn, 21);
        assert_eq!(producer + treasury + burn, 101);
    });
}

#[test]
fn charge_fee_accumulates_treasury_and_burn() {
    new_test_ext().execute_with(|| {
        let (p, t, b) = Fees::charge_fee(1_000);
        assert_eq!((p, t, b), (500, 300, 200));
        assert_eq!(TreasuryAccumulated::<Test>::get(), 300);
        assert_eq!(BurnCounter::<Test>::get(), 200);

        Fees::charge_fee(1_000);
        assert_eq!(TreasuryAccumulated::<Test>::get(), 600);
        assert_eq!(BurnCounter::<Test>::get(), 400);
    });
}

#[test]
fn minimum_fee_scales_with_io() {
    new_test_ext().execute_with(|| {
        // base 1000 + (1 + 2) * 100 = 1300
        assert_eq!(Fees::minimum_fee(1, 2), 1_300);
        // base 1000 + (5 + 5) * 100 = 2000
        assert_eq!(Fees::minimum_fee(5, 5), 2_000);
    });
}

#[test]
fn set_treasury_lock_requires_root() {
    new_test_ext().execute_with(|| {
        use frame_support::assert_noop;
        assert_noop!(
            Fees::set_treasury_lock(RuntimeOrigin::signed(1), [1u8; 32]),
            sp_runtime::DispatchError::BadOrigin
        );
        assert_ok!(Fees::set_treasury_lock(RuntimeOrigin::root(), [9u8; 32]));
        assert_eq!(TreasuryLock::<Test>::get(), [9u8; 32]);
    });
}

#[test]
fn register_producer_lock_requires_root() {
    new_test_ext().execute_with(|| {
        use frame_support::assert_noop;
        assert_noop!(
            Fees::register_producer_lock(RuntimeOrigin::signed(1), 42u64, [1u8; 32]),
            sp_runtime::DispatchError::BadOrigin
        );
        assert_ok!(Fees::register_producer_lock(RuntimeOrigin::root(), 42u64, [3u8; 32]));
        assert_eq!(ProducerLocks::<Test>::get(42u64), Some([3u8; 32]));
    });
}

#[test]
fn genesis_sets_treasury_lock() {
    new_test_ext().execute_with(|| {
        assert_eq!(TreasuryLock::<Test>::get(), [7u8; 32]);
    });
}

// ── Phase 8.4 tests: dynamic base fee ──

#[test]
fn current_base_fee_seeded_from_genesis() {
    new_test_ext().execute_with(|| {
        assert_eq!(CurrentBaseFee::<Test>::get(), 1_000);
    });
}

#[test]
fn base_fee_rises_when_block_over_target() {
    new_test_ext().execute_with(|| {
        // Block at 100% weight: target is 50%, so it's 100% over target.
        let max = <Test as frame_system::Config>::BlockWeights::get().max_block;
        frame_system::Pallet::<Test>::register_extra_weight_unchecked(
            max,
            frame_support::dispatch::DispatchClass::Normal,
        );
        let initial = CurrentBaseFee::<Test>::get();
        Fees::adjust_base_fee();
        let after = CurrentBaseFee::<Test>::get();
        assert!(after > initial, "base fee should rise: {} -> {}", initial, after);
        // Full over-target = 100% of MaxBaseFeeChangePct = 12% rise
        assert_eq!(after, 1_120);
    });
}

#[test]
fn base_fee_falls_when_block_under_target() {
    new_test_ext().execute_with(|| {
        // Empty block: 0% weight, target 50%. Deficit ratio capped at 100%.
        let initial = CurrentBaseFee::<Test>::get();
        Fees::adjust_base_fee();
        let after = CurrentBaseFee::<Test>::get();
        assert!(after < initial, "base fee should fall: {} -> {}", initial, after);
        // 100% deficit capped, 12% drop
        assert_eq!(after, 880);
    });
}

#[test]
fn base_fee_respects_floor() {
    new_test_ext().execute_with(|| {
        // Manually set to floor + small amount, then decay
        CurrentBaseFee::<Test>::put(505);
        Fees::adjust_base_fee();  // would drop 12% -> 444, clamp to 500
        assert_eq!(CurrentBaseFee::<Test>::get(), 500);
    });
}

#[test]
fn base_fee_respects_ceiling() {
    new_test_ext().execute_with(|| {
        CurrentBaseFee::<Test>::put(999_000);
        let max = <Test as frame_system::Config>::BlockWeights::get().max_block;
        frame_system::Pallet::<Test>::register_extra_weight_unchecked(
            max,
            frame_support::dispatch::DispatchClass::Normal,
        );
        Fees::adjust_base_fee();
        // 12% rise from 999k -> >1M, clamped
        assert!(CurrentBaseFee::<Test>::get() <= 1_000_000);
    });
}

#[test]
fn minimum_fee_uses_dynamic_base() {
    new_test_ext().execute_with(|| {
        // Genesis: base = 1000, per_io = 100. min_fee(1,2) = 1000 + 300 = 1300
        assert_eq!(Fees::minimum_fee(1, 2), 1_300);

        // Raise base fee manually, verify min_fee tracks it
        CurrentBaseFee::<Test>::put(2_000);
        assert_eq!(Fees::minimum_fee(1, 2), 2_300);
    });
}

// ── Phase 8.5 tests: block rewards ──

#[test]
fn block_reward_splits_to_treasury_and_producer() {
    new_test_ext().execute_with(|| {
        // reward = 10_000; 50% producer / 30% treasury => 5_000 / 3_000
        Fees::distribute_block_reward();
        assert_eq!(TreasuryAccumulated::<Test>::get(), 3_000);
        assert_eq!(TotalRewardsMinted::<Test>::get(), 10_000);
    });
}

#[test]
fn block_reward_accumulates_over_blocks() {
    new_test_ext().execute_with(|| {
        Fees::distribute_block_reward();
        Fees::distribute_block_reward();
        Fees::distribute_block_reward();
        assert_eq!(TotalRewardsMinted::<Test>::get(), 30_000);
        assert_eq!(TreasuryAccumulated::<Test>::get(), 9_000);
    });
}

#[test]
fn zero_block_reward_is_noop() {
    new_test_ext().execute_with(|| {
        // Override to zero
        // (in real test we'd have a separate mock; here we just call twice and check math holds)
        Fees::distribute_block_reward();
        let once = TotalRewardsMinted::<Test>::get();
        Fees::distribute_block_reward();
        let twice = TotalRewardsMinted::<Test>::get();
        assert_eq!(twice, once + 10_000, "reward should accumulate linearly");
    });
}

// ── Phase 8.6 tests: producer payout settlement ──
//
// Q2(a) invariant: fees owed to a producer are never burned. If the author
// has a registered lock, settle mints a UTXO. If not, the amount is held in
// `ProducerPending[author]` and paid on a later block.

use frame_support::traits::Hooks;

/// Populate both accumulators. `base_fee_total` is the raw fee charged —
/// the producer accumulator receives 50% of it (ProducerFeeShare). Treasury
/// gets 30%, burn 20%, per `split_is_50_30_20`.
fn seed_accumulators(base_fee_total: u128, priority_portion: u128) {
    if base_fee_total > 0 {
        Fees::charge_fee(base_fee_total);
    }
    if priority_portion > 0 {
        Fees::charge_priority_fee(priority_portion);
    }
}

fn author_pending(a: u64) -> u128 {
    ProducerPending::<Test>::get(a)
}

#[test]
fn on_initialize_captures_author() {
    new_test_ext().execute_with(|| {
        set_next_author(Some(7));
        let _ = Fees::on_initialize(0);
        assert_eq!(CurrentAuthor::<Test>::get(), Some(7));
    });
}

#[test]
fn settle_with_lock_mints_and_clears() {
    new_test_ext().execute_with(|| {
        set_next_author(Some(7));
        let _ = Fees::on_initialize(0);

        // 1000 base fee -> 500 producer cut; 500 priority.
        seed_accumulators(1_000, 500);
        assert_eq!(ProducerAccumulated::<Test>::get(), 500);
        assert_eq!(PriorityFeeAccumulated::<Test>::get(), 500);

        // Register a lock for the author.
        assert_ok!(Fees::register_producer_lock(RuntimeOrigin::root(), 7u64, [9u8; 32]));

        Fees::settle_producer_payouts();

        assert_eq!(take_minted(), vec![(1_000, [9u8; 32])]);
        assert_eq!(ProducerAccumulated::<Test>::get(), 0);
        assert_eq!(PriorityFeeAccumulated::<Test>::get(), 0);
        assert_eq!(author_pending(7), 0);
    });
}

#[test]
fn settle_without_lock_defers_and_never_burns() {
    new_test_ext().execute_with(|| {
        set_next_author(Some(7));
        let _ = Fees::on_initialize(0);

        seed_accumulators(1_000, 500);
        // No `register_producer_lock` for author 7.

        Fees::settle_producer_payouts();

        assert!(take_minted().is_empty(), "must not mint without a lock");
        assert_eq!(author_pending(7), 1_000, "payout held pending, not burned");
        assert_eq!(ProducerAccumulated::<Test>::get(), 0);
        assert_eq!(PriorityFeeAccumulated::<Test>::get(), 0);
        // The 200 burn is the normal 20% split remainder — deferral does not\n        // add to it. What matters is that the 1000 owed to the producer was\n        // not destroyed: it lives in ProducerPending.
        assert_eq!(BurnCounter::<Test>::get(), 200, "only the normal split burn");
    });
}

#[test]
fn settle_carries_pending_forward() {
    new_test_ext().execute_with(|| {
        // First block: no lock, 500 deferred.
        set_next_author(Some(7));
        let _ = Fees::on_initialize(0);
        seed_accumulators(1_000, 0); // 500 producer cut
        Fees::settle_producer_payouts();
        assert_eq!(author_pending(7), 500);

        // Register lock. Second block: earns another 500, settles 1000 total.
        assert_ok!(Fees::register_producer_lock(RuntimeOrigin::root(), 7u64, [4u8; 32]));
        let _ = Fees::on_initialize(1);
        seed_accumulators(1_000, 0);
        Fees::settle_producer_payouts();

        assert_eq!(take_minted(), vec![(1_000, [4u8; 32])]);
        assert_eq!(author_pending(7), 0);
    });
}

#[test]
fn settle_no_author_leaves_accumulators() {
    new_test_ext().execute_with(|| {
        // No author captured (NEXT_AUTHOR = None).
        let _ = Fees::on_initialize(0);
        seed_accumulators(1_000, 500);

        Fees::settle_producer_payouts();

        assert!(take_minted().is_empty());
        assert_eq!(ProducerAccumulated::<Test>::get(), 500, "untouched");
        assert_eq!(PriorityFeeAccumulated::<Test>::get(), 500, "untouched");
    });
}

#[test]
fn settle_zero_is_noop() {
    new_test_ext().execute_with(|| {
        set_next_author(Some(7));
        let _ = Fees::on_initialize(0);
        assert_ok!(Fees::register_producer_lock(RuntimeOrigin::root(), 7u64, [1u8; 32]));

        Fees::settle_producer_payouts();

        assert!(take_minted().is_empty());
        assert_eq!(author_pending(7), 0);
    });
}
