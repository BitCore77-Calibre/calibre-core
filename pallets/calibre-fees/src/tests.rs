#![cfg(test)]

use crate::{self as pallet_calibre_fees, *};
use frame_support::{assert_ok, derive_impl, parameter_types};
use frame_support::traits::ConstU32;
use sp_runtime::BuildStorage;

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        Fees: pallet_calibre_fees,
    }
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type Block = Block;
}

parameter_types! {
    pub const BaseTxFee: u128 = 1_000;
    pub const PerInOutFee: u128 = 100;
    pub const ProducerFeeShare: u8 = 50;
    pub const TreasuryFeeShare: u8 = 30;
}

impl pallet_calibre_fees::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Balance = u128;
    type BaseTxFee = BaseTxFee;
    type PerInOutFee = PerInOutFee;
    type ProducerFeeShare = ProducerFeeShare;
    type TreasuryFeeShare = TreasuryFeeShare;
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
        let (p, t, b) = Fees::charge_fee(1_000, None);
        assert_eq!((p, t, b), (500, 300, 200));
        assert_eq!(TreasuryAccumulated::<Test>::get(), 300);
        assert_eq!(BurnCounter::<Test>::get(), 200);

        Fees::charge_fee(1_000, None);
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
