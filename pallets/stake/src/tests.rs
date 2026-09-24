#![cfg(test)]

use crate::{self as pallet_stake, *};
use calibre_primitives::{FeeMinter, TransactionInput, UtxoConsumer};
use frame_support::{assert_noop, assert_ok, derive_impl, parameter_types};
use frame_support::traits::{ConstU32, ConstU128};
use sp_runtime::BuildStorage;

type Block = frame_system::mocking::MockBlock<Test>;

thread_local! {
    static CONSUMED: std::cell::RefCell<Vec<(u64, usize, Vec<u8>)>> =
        std::cell::RefCell::new(Vec::new());
    static CONSUME_RESULT: std::cell::RefCell<Option<u128>> =
        std::cell::RefCell::new(None);
    static MINTED: std::cell::RefCell<Vec<(u128, [u8; 32])>> =
        std::cell::RefCell::new(Vec::new());
}

pub struct TestUtxoConsumer;
impl UtxoConsumer<u128, u64> for TestUtxoConsumer {
    fn consume_with_witness(beneficiary: &u64, inputs: &[TransactionInput], witness: &[u8]) -> Option<u128> {
        CONSUMED.with(|c| c.borrow_mut().push((*beneficiary, inputs.len(), witness.to_vec())));
        CONSUME_RESULT.with(|r| *r.borrow())
    }
}

pub struct TestUtxoMinter;
impl FeeMinter<u128, [u8; 32]> for TestUtxoMinter {
    fn mint_to_lock(value: u128, lock: [u8; 32]) {
        MINTED.with(|m| m.borrow_mut().push((value, lock)));
    }
}

fn set_consume_result(r: Option<u128>) {
    CONSUME_RESULT.with(|slot| *slot.borrow_mut() = r);
}
fn take_minted() -> Vec<(u128, [u8; 32])> {
    MINTED.with(|m| std::mem::take(&mut *m.borrow_mut()))
}

frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        Stake: pallet_stake,
    }
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type Block = Block;
}

impl pallet_stake::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Balance = u128;
    type MaxBondInputs = ConstU32<16>;
    type MinValidatorStake = ConstU128<1000>;
    type UtxoConsumer = TestUtxoConsumer;
    type UtxoMinter = TestUtxoMinter;
    type WeightInfo = ();
}

fn new_test_ext() -> sp_io::TestExternalities {
    set_consume_result(None);
    CONSUMED.with(|c| c.borrow_mut().clear());
    let _ = take_minted();
    frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap()
        .into()
}

fn dummy_input() -> TransactionInput {
    TransactionInput { tx_hash: sp_core::H256::zero(), output_index: 0 }
}

#[test]
fn bond_credits_stake_and_total() {
    new_test_ext().execute_with(|| {
        set_consume_result(Some(1_000));
        assert_ok!(Stake::bond(RuntimeOrigin::signed(1), vec![dummy_input()], vec![7, 7]));
        assert_eq!(Stake::stake_of(1u64), 1_000);
        assert_eq!(Stake::total_staked(), 1_000);
        CONSUMED.with(|c| assert_eq!(*c.borrow(), vec![(1, 1, vec![7, 7])]));
    });
}

#[test]
fn bond_propagates_consumer_failure() {
    new_test_ext().execute_with(|| {
        set_consume_result(None);
        let r = Stake::bond(RuntimeOrigin::signed(1), vec![dummy_input()], vec![]);
        assert!(r.is_err(), "consumer returned None; bond must fail");
        assert_eq!(Stake::stake_of(1u64), 0);
    });
}

#[test]
fn bond_rejects_too_many_inputs() {
    new_test_ext().execute_with(|| {
        set_consume_result(Some(1));
        let inputs: Vec<_> = (0..17).map(|i| TransactionInput {
            tx_hash: sp_core::H256::from_low_u64_be(i), output_index: 0,
        }).collect();
        let r = Stake::bond(RuntimeOrigin::signed(1), inputs, vec![]);
        assert!(r.is_err());
    });
}

#[test]
fn bond_accumulates_across_calls() {
    new_test_ext().execute_with(|| {
        set_consume_result(Some(500));
        assert_ok!(Stake::bond(RuntimeOrigin::signed(1), vec![dummy_input()], vec![]));
        assert_ok!(Stake::bond(RuntimeOrigin::signed(1), vec![dummy_input()], vec![]));
        assert_eq!(Stake::stake_of(1u64), 1_000);
        assert_eq!(Stake::total_staked(), 1_000);
    });
}

#[test]
fn unbond_debits_and_mints() {
    new_test_ext().execute_with(|| {
        set_consume_result(Some(1_000));
        assert_ok!(Stake::bond(RuntimeOrigin::signed(1), vec![dummy_input()], vec![]));
        assert_ok!(Stake::unbond(RuntimeOrigin::signed(1), 400, [3u8; 32]));
        assert_eq!(Stake::stake_of(1u64), 600);
        assert_eq!(Stake::total_staked(), 600);
        assert_eq!(take_minted(), vec![(400, [3u8; 32])]);
    });
}

#[test]
fn unbond_over_balance_fails() {
    new_test_ext().execute_with(|| {
        set_consume_result(Some(100));
        assert_ok!(Stake::bond(RuntimeOrigin::signed(1), vec![dummy_input()], vec![]));
        let r = Stake::unbond(RuntimeOrigin::signed(1), 101, [1u8; 32]);
        assert!(r.is_err());
        assert_eq!(Stake::stake_of(1u64), 100);
    });
}

#[test]
fn unbond_full_balance_is_noop_state() {
    new_test_ext().execute_with(|| {
        set_consume_result(Some(100));
        assert_ok!(Stake::bond(RuntimeOrigin::signed(1), vec![dummy_input()], vec![]));
        assert_ok!(Stake::unbond(RuntimeOrigin::signed(1), 100, [1u8; 32]));
        assert_eq!(Stake::stake_of(1u64), 0);
        assert_eq!(Stake::total_staked(), 0);
        assert_eq!(take_minted(), vec![(100, [1u8; 32])]);
    });
}

#[test]
fn accounts_are_independent() {
    new_test_ext().execute_with(|| {
        set_consume_result(Some(300));
        assert_ok!(Stake::bond(RuntimeOrigin::signed(1), vec![dummy_input()], vec![]));
        assert_ok!(Stake::bond(RuntimeOrigin::signed(2), vec![dummy_input()], vec![]));
        assert_eq!(Stake::stake_of(1u64), 300);
        assert_eq!(Stake::stake_of(2u64), 300);
        assert_eq!(Stake::total_staked(), 600);
        assert_ok!(Stake::unbond(RuntimeOrigin::signed(1), 300, [5u8; 32]));
        assert_eq!(Stake::stake_of(1u64), 0);
        assert_eq!(Stake::stake_of(2u64), 300, "unaffected");
    });
}

// ─────────────────────────────────────────────────────────────
// Candidate registry tests (Phase 9.3)
// ─────────────────────────────────────────────────────────────

#[test]
fn join_candidates_below_minimum_fails() {
    new_test_ext().execute_with(|| {
        // MinValidatorStake = 1000. Stake 999 -> below threshold.
        crate::Stake::<Test>::insert(1u64, 999u128);
        assert_noop!(
            Pallet::<Test>::join_candidates(RuntimeOrigin::signed(1)),
            Error::<Test>::InsufficientStakeForCandidate
        );
        assert!(!crate::Candidates::<Test>::contains_key(1u64));
    });
}

#[test]
fn join_candidates_at_minimum_succeeds() {
    new_test_ext().execute_with(|| {
        crate::Stake::<Test>::insert(1u64, 1000u128);
        assert_ok!(Pallet::<Test>::join_candidates(RuntimeOrigin::signed(1)));
        assert!(crate::Candidates::<Test>::contains_key(1u64));
    });
}

#[test]
fn join_candidates_above_minimum_succeeds() {
    new_test_ext().execute_with(|| {
        crate::Stake::<Test>::insert(1u64, 1_000_000u128);
        assert_ok!(Pallet::<Test>::join_candidates(RuntimeOrigin::signed(1)));
        assert!(crate::Candidates::<Test>::contains_key(1u64));
    });
}

#[test]
fn join_candidates_twice_fails() {
    new_test_ext().execute_with(|| {
        crate::Stake::<Test>::insert(1u64, 1000u128);
        assert_ok!(Pallet::<Test>::join_candidates(RuntimeOrigin::signed(1)));
        assert_noop!(
            Pallet::<Test>::join_candidates(RuntimeOrigin::signed(1)),
            Error::<Test>::AlreadyCandidate
        );
    });
}

#[test]
fn leave_candidates_removes_from_registry() {
    new_test_ext().execute_with(|| {
        crate::Stake::<Test>::insert(1u64, 1000u128);
        assert_ok!(Pallet::<Test>::join_candidates(RuntimeOrigin::signed(1)));
        assert!(crate::Candidates::<Test>::contains_key(1u64));
        assert_ok!(Pallet::<Test>::leave_candidates(RuntimeOrigin::signed(1)));
        assert!(!crate::Candidates::<Test>::contains_key(1u64));
    });
}

#[test]
fn leave_candidates_non_candidate_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Pallet::<Test>::leave_candidates(RuntimeOrigin::signed(1)),
            Error::<Test>::NotCandidate
        );
    });
}

#[test]
fn elect_top_n_picks_highest_stake() {
    new_test_ext().execute_with(|| {
        // Three candidates with different stakes.
        for (acct, amt) in [(1u64, 1_000u128), (2u64, 5_000u128), (3u64, 3_000u128)] {
            crate::Stake::<Test>::insert(acct, amt);
            assert_ok!(Pallet::<Test>::join_candidates(RuntimeOrigin::signed(acct)));
        }
        let top = Pallet::<Test>::elect_top_n(2);
        assert_eq!(top, vec![2u64, 3u64], "highest two by stake");
    });
}

#[test]
fn elect_top_n_skips_zero_stake() {
    new_test_ext().execute_with(|| {
        // Candidate with stake above min, then drained to zero.
        crate::Stake::<Test>::insert(1u64, 1_000u128);
        assert_ok!(Pallet::<Test>::join_candidates(RuntimeOrigin::signed(1)));
        crate::Stake::<Test>::insert(1u64, 0u128);
        // Second candidate with real stake.
        crate::Stake::<Test>::insert(2u64, 2_000u128);
        assert_ok!(Pallet::<Test>::join_candidates(RuntimeOrigin::signed(2)));

        let top = Pallet::<Test>::elect_top_n(10);
        assert_eq!(top, vec![2u64], "zero-stake candidate excluded");
    });
}

#[test]
fn elect_top_n_respects_limit() {
    new_test_ext().execute_with(|| {
        for acct in 1u64..=5 {
            crate::Stake::<Test>::insert(acct, acct as u128 * 1_000);
            assert_ok!(Pallet::<Test>::join_candidates(RuntimeOrigin::signed(acct)));
        }
        let top = Pallet::<Test>::elect_top_n(3);
        assert_eq!(top.len(), 3);
        assert_eq!(top, vec![5u64, 4u64, 3u64], "top 3 by stake, descending");
    });
}

#[test]
fn elect_top_n_empty_when_no_candidates() {
    new_test_ext().execute_with(|| {
        let top = Pallet::<Test>::elect_top_n(10);
        assert!(top.is_empty());
    });
}

#[test]
fn candidate_count_reflects_registry() {
    new_test_ext().execute_with(|| {
        for acct in 1u64..=3 {
            crate::Stake::<Test>::insert(acct, 1_000u128);
            assert_ok!(Pallet::<Test>::join_candidates(RuntimeOrigin::signed(acct)));
        }
        assert_eq!(Pallet::<Test>::candidate_count(), 3);
        assert_ok!(Pallet::<Test>::leave_candidates(RuntimeOrigin::signed(2)));
        assert_eq!(Pallet::<Test>::candidate_count(), 2);
    });
}
