#![cfg(test)]

use crate::{self as pallet_qutxo, *};
use frame_support::{assert_noop, assert_ok, parameter_types, traits::ConstU32};
use frame_support::traits::Hooks;
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};
use calibre_primitives::{FeeHandler, QuantumLock, Utxo, TransactionInput, TransactionOutput};
use calibre_aegis_crypto::{AegisWitness, DeviceAttestation};
use parity_scale_codec::Encode;
use sp_runtime::traits::Hash;

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        Qutxo: pallet_qutxo,
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Nonce = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Block = Block;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
    type RuntimeTask = RuntimeTask;
    type SingleBlockMigrations = ();
    type MultiBlockMigrator = ();
    type PreInherents = ();
    type PostInherents = ();
    type PostTransactions = ();
    type ExtensionsWeightInfo = ();
}

impl pallet_qutxo::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Balance = u128;
    type MaxTxInputs = ConstU32<16>;
    type MaxTxOutputs = ConstU32<16>;
	type WeightInfo = ();
	type FeeHandler = TestFeeHandler;
}

thread_local! {
    /// Minimum fee returned by the test FeeHandler. Tests that exercise the
    /// pool-side fee check set this; the default 0 lets existing tests run
    /// untouched (no tx is ever "under-priced" by default).
    static MIN_FEE: std::cell::RefCell<u128> = std::cell::RefCell::new(0);
}

/// Test FeeHandler. Replaces `()` so the pool-admission fee check is
/// exercisable end-to-end. `charge_fee` and `charge_priority_fee` are no-ops
/// (they don't affect validate_unsigned, which only reads `minimum_fee`).
pub struct TestFeeHandler;
impl FeeHandler<u128> for TestFeeHandler {
    fn charge_fee(_fee: u128) -> (u128, u128, u128) { (0, 0, 0) }
    fn minimum_fee(_inputs: u32, _outputs: u32) -> u128 {
        MIN_FEE.with(|m| *m.borrow())
    }
    fn charge_priority_fee(_fee: u128) -> u128 { 0 }
}

fn set_min_fee(f: u128) {
    MIN_FEE.with(|m| *m.borrow_mut() = f);
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    set_min_fee(0);
    let t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}

fn mock_witness() -> Vec<u8> {
    let w = AegisWitness {
        pub_keys: vec![],
        aggregated_signature: vec![],
        attestation: DeviceAttestation {
            hardware_signature: vec![],
            app_binary_hash: [0; 32],
            backend_challenge: [0; 32],
        },
        time_drift: 0,
    };
    w.encode()
}

/// Real ML-DSA-44 keypair, real signature, real AegisThreshold lock.
///
/// Signs `(inputs, outputs).encode()` with an empty context, matching the
/// payload built inside `execute_utxo_tx` and the verification convention in
/// `calibre_aegis_crypto::verify_aegis_transaction` (which uses ctx = b"").
///
/// Returns `(lock, tx)` where `lock` is the `AegisThreshold` the caller must
/// place on the input UTXO — the verifier hashes `pub_keys[0]` and compares
/// to the lock, so they must be paired.
fn real_signed_tx(
    inputs: Vec<TransactionInput>,
    outputs: Vec<TransactionOutput<u128>>,
) -> (QuantumLock, Transaction<u128>) {
    let kp = dilithium::MlDsaKeyPair::generate(dilithium::ML_DSA_44).expect("keygen");
    let pk_bytes = kp.public_key().to_vec();
    let lock_bytes = sp_core::hashing::blake2_256(&pk_bytes);

    let msg = (inputs.clone(), outputs.clone()).encode();
    let sig = kp.sign(&msg, b"").expect("sign");

    let witness = AegisWitness {
        pub_keys: vec![pk_bytes],
        aggregated_signature: sig.as_bytes().to_vec(),
        attestation: DeviceAttestation {
            hardware_signature: vec![],
            app_binary_hash: [0; 32],
            backend_challenge: [0; 32],
        },
        time_drift: 0,
    };

    (
        QuantumLock::AegisThreshold(lock_bytes),
        Transaction {
            inputs,
            outputs,
            pq_signature: vec![],
            witness: witness.encode(),
        },
    )
}

#[test]
fn test_utxo_conservation_of_mass_and_double_spend() {
    new_test_ext().execute_with(|| {
        // 1. Build the real signed transaction first, so we know the
        //    AegisThreshold lock the genesis UTXO must carry (it must match
        //    blake2_256(pubkey) of the signer).
        let genesis_tx_hash = H256::from([1u8; 32]);
        let genesis_utxo_hash = Qutxo::calculate_utxo_hash(genesis_tx_hash, 0);

        let inputs = vec![TransactionInput {
            tx_hash: genesis_tx_hash,
            output_index: 0,
        }];
        let outputs = vec![
            TransactionOutput { value: 60, lock: QuantumLock::SingleSig([2u8; 32]) },
            TransactionOutput { value: 40, lock: QuantumLock::SingleSig([3u8; 32]) },
        ];
        let (input_lock, valid_tx) = real_signed_tx(inputs, outputs);

        // 2. Inject the genesis UTXO with the matching lock.
        UtxoSet::<Test>::insert(
            genesis_utxo_hash,
            Utxo { value: 100, lock: input_lock },
        );
        TotalIssuance::<Test>::put(100);

        assert_eq!(UtxoSet::<Test>::get(genesis_utxo_hash).unwrap().value, 100);

        // Execute the valid transaction.
        assert_ok!(Qutxo::execute_utxo_tx(RuntimeOrigin::none(), valid_tx.clone()));

        // 3. Verify state transition: Old UTXO is destroyed, new ones are created
        assert!(UtxoSet::<Test>::get(genesis_utxo_hash).is_none(), "Genesis UTXO should be destroyed");
        
        let valid_tx_hash = BlakeTwo256::hash(&(valid_tx.inputs.clone(), valid_tx.outputs.clone()).encode());
        let new_utxo_1_hash = Qutxo::calculate_utxo_hash(valid_tx_hash, 0);
        let new_utxo_2_hash = Qutxo::calculate_utxo_hash(valid_tx_hash, 1);
        
        assert_eq!(UtxoSet::<Test>::get(new_utxo_1_hash).unwrap().value, 60);
        assert_eq!(UtxoSet::<Test>::get(new_utxo_2_hash).unwrap().value, 40);

        // 4. The Double Spend Attack (The Conflict Detector)
        // Try to submit the EXACT same transaction again. 
        // The input (genesis_utxo_hash) no longer exists in the UTXO set.
        // The Fast-Path conflict detector MUST reject this instantly.
        assert_noop!(
            Qutxo::execute_utxo_tx(RuntimeOrigin::none(), valid_tx),
            Error::<Test>::UtxoDoesNotExist
        );
        
        println!("✅ CONSERVATION OF MASS VERIFIED.");
        println!("✅ DOUBLE-SPEND (CONFLICT) INSTANTLY REJECTED.");
    });
}

#[test]
fn root_is_zero_when_set_empty() {
    new_test_ext().execute_with(|| {
        assert_eq!(UtxoSetRoot::<Test>::get(), H256::zero());
    });
}

#[test]
fn root_changes_when_utxo_added() {
    new_test_ext().execute_with(|| {
        let before = UtxoSetRoot::<Test>::get();
        UtxoSet::<Test>::insert(
            H256::from([1u8; 32]),
            Utxo { value: 100, lock: QuantumLock::SingleSig([0u8; 32]) },
        );
        Qutxo::mark_utxo_set_dirty();
        Qutxo::on_finalize(2);
        assert_ne!(before, UtxoSetRoot::<Test>::get());
    });
}

#[test]
fn root_changes_when_utxo_removed() {
    new_test_ext().execute_with(|| {
        let h = H256::from([1u8; 32]);
        UtxoSet::<Test>::insert(
            h,
            Utxo { value: 100, lock: QuantumLock::SingleSig([0u8; 32]) },
        );
        Qutxo::mark_utxo_set_dirty();
        Qutxo::on_finalize(2);
        let before = UtxoSetRoot::<Test>::get();

        UtxoSet::<Test>::remove(h);
        Qutxo::mark_utxo_set_dirty();
        Qutxo::on_finalize(3);
        assert_ne!(before, UtxoSetRoot::<Test>::get());
    });
}

fn compute_root_after_inserting(order: &[u8]) -> H256 {
    let mut r = H256::zero();
    new_test_ext().execute_with(|| {
        for i in order {
            UtxoSet::<Test>::insert(
                H256::from([*i; 32]),
                Utxo { value: *i as u128, lock: QuantumLock::SingleSig([*i; 32]) },
            );
        }
        Qutxo::mark_utxo_set_dirty();
        Qutxo::on_finalize(2);
        r = UtxoSetRoot::<Test>::get();
    });
    r
}

#[test]
fn root_is_deterministic_across_insertion_order() {
    assert_eq!(
        compute_root_after_inserting(&[1, 2, 3]),
        compute_root_after_inserting(&[3, 2, 1]),
    );
}

#[test]
fn inclusion_proof_folds_to_root() {
    new_test_ext().execute_with(|| {
        for i in 1u8..=3 {
            UtxoSet::<Test>::insert(
                H256::from([i; 32]),
                Utxo { value: i as u128 * 100, lock: QuantumLock::SingleSig([i; 32]) },
            );
        }
        Qutxo::mark_utxo_set_dirty();
        Qutxo::on_finalize(2);

        let target = H256::from([2u8; 32]);
        let proof = Qutxo::get_inclusion_proof(target).expect("present");
        assert_eq!(proof.utxo_id, target);
        assert_eq!(proof.root, UtxoSetRoot::<Test>::get());

        // Independently fold the path via calibre_merkle.
        let mut cur = calibre_merkle::leaf_hash(
            target.as_fixed_bytes(),
            proof.value_hash.as_fixed_bytes(),
        );
        for step in &proof.path {
            let sib = step.sibling.as_fixed_bytes();
            cur = if step.current_is_left {
                calibre_merkle::inner_hash(&cur, sib)
            } else {
                calibre_merkle::inner_hash(sib, &cur)
            };
        }
        assert_eq!(cur, *proof.root.as_fixed_bytes(), "path does not fold to root");
    });
}

#[test]
fn inclusion_proof_missing_utxo_is_none() {
    new_test_ext().execute_with(|| {
        assert!(Qutxo::get_inclusion_proof(H256::from([42u8; 32])).is_none());
    });
}


// ─────────────────────────────────────────────────────────────
// ANTI-SPAM: validate_unsigned tests
//
// These exercise the pool-admission path added in the anti-spam
// patch. The goal is to prove junk txs are rejected before they
// can occupy the mempool or force expensive ML-DSA verification
// in the block producer. None of these tests need a valid ML-DSA
// signature, so the mock_witness() helper (which is deliberately
// invalid) is fine.
// ─────────────────────────────────────────────────────────────

use sp_runtime::transaction_validity::{
    InvalidTransaction, TransactionSource, TransactionValidity, TransactionValidityError,
};
use sp_runtime::traits::ValidateUnsigned as _;

fn assert_invalid(res: TransactionValidity, expected: InvalidTransaction) {
    match res {
        Err(TransactionValidityError::Invalid(e)) => {
            assert_eq!(e, expected, "wrong invalid reason")
        }
        Ok(v) => panic!("expected Invalid({:?}), got Ok({:?})", expected, v),
        Err(TransactionValidityError::Unknown(u)) => {
            panic!("expected Invalid({:?}), got Unknown({:?})", expected, u)
        }
    }
}

fn run_validate_unsigned(tx: Transaction<u128>) -> TransactionValidity {
    let call: crate::Call<Test> = crate::Call::execute_utxo_tx { tx };
    crate::Pallet::<Test>::validate_unsigned(TransactionSource::External, &call)
}

fn one_input(tx_hash: H256, output_index: u32) -> Vec<TransactionInput> {
    vec![TransactionInput { tx_hash, output_index }]
}

fn dummy_tx(witness: Vec<u8>, inputs: Vec<TransactionInput>) -> Transaction<u128> {
    Transaction {
        inputs,
        outputs: vec![TransactionOutput { value: 1, lock: QuantumLock::SingleSig([9u8; 32]) }],
        pq_signature: vec![],
        witness,
    }
}

#[test]
fn validate_unsigned_rejects_empty_witness() {
    new_test_ext().execute_with(|| {
        let tx = dummy_tx(vec![], one_input(H256::from([1u8; 32]), 0));
        assert_invalid(run_validate_unsigned(tx), InvalidTransaction::BadProof);
    });
}

#[test]
fn validate_unsigned_rejects_oversized_witness() {
    new_test_ext().execute_with(|| {
        let tx = dummy_tx(vec![0u8; 16 * 1024], one_input(H256::from([1u8; 32]), 0));
        assert_invalid(run_validate_unsigned(tx), InvalidTransaction::BadProof);
    });
}

#[test]
fn validate_unsigned_rejects_empty_inputs() {
    new_test_ext().execute_with(|| {
        let tx = dummy_tx(mock_witness(), vec![]);
        assert_invalid(run_validate_unsigned(tx), InvalidTransaction::ExhaustsResources);
    });
}

#[test]
fn validate_unsigned_rejects_too_many_inputs() {
    new_test_ext().execute_with(|| {
        // MaxTxInputs = 16; 17 inputs must be rejected before any storage read.
        let inputs: Vec<_> = (0..17u8)
            .map(|i| TransactionInput { tx_hash: H256::from([i; 32]), output_index: 0 })
            .collect();
        let tx = dummy_tx(mock_witness(), inputs);
        assert_invalid(run_validate_unsigned(tx), InvalidTransaction::ExhaustsResources);
    });
}

#[test]
fn validate_unsigned_rejects_duplicate_inputs() {
    new_test_ext().execute_with(|| {
        let h = H256::from([7u8; 32]);
        let inputs = vec![
            TransactionInput { tx_hash: h, output_index: 0 },
            TransactionInput { tx_hash: h, output_index: 0 },
        ];
        let tx = dummy_tx(mock_witness(), inputs);
        assert_invalid(run_validate_unsigned(tx), InvalidTransaction::BadProof);
    });
}

#[test]
fn validate_unsigned_rejects_stale_utxo() {
    new_test_ext().execute_with(|| {
        // No UTXO inserted; first-input lookup must fail Stale.
        let tx = dummy_tx(mock_witness(), one_input(H256::from([1u8; 32]), 0));
        assert_invalid(run_validate_unsigned(tx), InvalidTransaction::Stale);
    });
}

#[test]
fn validate_unsigned_rejects_non_aegis_lock() {
    new_test_ext().execute_with(|| {
        let tx_hash = H256::from([1u8; 32]);
        let utxo_id = Qutxo::calculate_utxo_hash(tx_hash, 0);
        UtxoSet::<Test>::insert(
            utxo_id,
            Utxo { value: 100, lock: QuantumLock::SingleSig([0u8; 32]) },
        );
        let tx = dummy_tx(mock_witness(), one_input(tx_hash, 0));
        assert_invalid(run_validate_unsigned(tx), InvalidTransaction::BadProof);
    });
}

#[test]
fn validate_unsigned_rejects_corrupt_witness_encoding() {
    new_test_ext().execute_with(|| {
        let tx_hash = H256::from([1u8; 32]);
        let utxo_id = Qutxo::calculate_utxo_hash(tx_hash, 0);
        UtxoSet::<Test>::insert(
            utxo_id,
            Utxo { value: 100, lock: QuantumLock::AegisThreshold([0u8; 32]) },
        );
        // Compact mode 0b11 prefix declares a 4-byte length, but only 3 bytes
        // follow -> SCALE decode must fail -> BadProof.
        let tx = dummy_tx(vec![0xff, 0xff, 0xff, 0xff], one_input(tx_hash, 0));
        assert_invalid(run_validate_unsigned(tx), InvalidTransaction::BadProof);
    });
}

#[test]
fn validate_unsigned_rejects_empty_pub_keys() {
    new_test_ext().execute_with(|| {
        let tx_hash = H256::from([1u8; 32]);
        let utxo_id = Qutxo::calculate_utxo_hash(tx_hash, 0);
        UtxoSet::<Test>::insert(
            utxo_id,
            Utxo { value: 100, lock: QuantumLock::AegisThreshold([0u8; 32]) },
        );
        // mock_witness decodes cleanly but has zero pub_keys -> InsufficientShares -> BadProof.
        let tx = dummy_tx(mock_witness(), one_input(tx_hash, 0));
        assert_invalid(run_validate_unsigned(tx), InvalidTransaction::BadProof);
    });
}

// ── Fee-rejection E2E (debt item from SESSION_STATE.md, paired with the
//    Phase 5 un-ignore): proves that a tx with a valid ML-DSA witness but an
//    under-priced fee is rejected at pool admission with Payment, and that
//    one meeting the minimum is accepted.

#[test]
fn validate_unsigned_rejects_below_minimum_fee() {
    new_test_ext().execute_with(|| {
        // 100 in, 1 out -> fee 99. Set minimum to 100 -> reject.
        set_min_fee(100);

        let tx_hash = H256::from([42u8; 32]);
        let utxo_id = Qutxo::calculate_utxo_hash(tx_hash, 0);

        let inputs = one_input(tx_hash, 0);
        let outputs = vec![TransactionOutput {
            value: 1,
            lock: QuantumLock::SingleSig([9u8; 32]),
        }];
        let (input_lock, tx) = real_signed_tx(inputs, outputs);

        UtxoSet::<Test>::insert(utxo_id, Utxo { value: 100, lock: input_lock });

        assert_invalid(run_validate_unsigned(tx), InvalidTransaction::Payment);
    });
}

#[test]
fn validate_unsigned_accepts_at_minimum_fee() {
    new_test_ext().execute_with(|| {
        // Same shape, minimum lowered to 99 -> accepted (fee == min, not <).
        set_min_fee(99);

        let tx_hash = H256::from([42u8; 32]);
        let utxo_id = Qutxo::calculate_utxo_hash(tx_hash, 0);

        let inputs = one_input(tx_hash, 0);
        let outputs = vec![TransactionOutput {
            value: 1,
            lock: QuantumLock::SingleSig([9u8; 32]),
        }];
        let (input_lock, tx) = real_signed_tx(inputs, outputs);

        UtxoSet::<Test>::insert(utxo_id, Utxo { value: 100, lock: input_lock });

        assert!(
            run_validate_unsigned(tx).is_ok(),
            "fee meeting the minimum must pass pool admission"
        );
    });
}

#[test]
fn validate_unsigned_accepts_zero_fee_when_minimum_is_zero() {
    new_test_ext().execute_with(|| {
        // The default mock (min = 0) must continue to accept.
        // 100 in, 100 out -> fee 0.
        let tx_hash = H256::from([43u8; 32]);
        let utxo_id = Qutxo::calculate_utxo_hash(tx_hash, 0);

        let inputs = one_input(tx_hash, 0);
        let outputs = vec![TransactionOutput {
            value: 100,
            lock: QuantumLock::SingleSig([9u8; 32]),
        }];
        let (input_lock, tx) = real_signed_tx(inputs, outputs);

        UtxoSet::<Test>::insert(utxo_id, Utxo { value: 100, lock: input_lock });

        assert!(run_validate_unsigned(tx).is_ok());
    });
}

#[test]
fn total_issuance_tracks_execute_utxo_tx_success() {
    new_test_ext().execute_with(|| {
        use crate::{TotalIssuance, UtxoSet};
        use calibre_primitives::{TransactionInput, TransactionOutput, Utxo};

        // Genesis UTXO hash — the input references this.
        let genesis_tx_hash = H256::from([42u8; 32]);
        let genesis_hash = Qutxo::calculate_utxo_hash(genesis_tx_hash, 0);

        // Build a valid signed tx FIRST so we capture the lock it signs against.
        let inputs = vec![TransactionInput { tx_hash: genesis_tx_hash, output_index: 0 }];
        let outputs = vec![TransactionOutput {
            value: 900,
            lock: QuantumLock::SingleSig([1u8; 32]),
        }];
        let (lock, tx) = real_signed_tx(inputs, outputs);

        // Seed the UTXO with the matching lock and 1,000 units.
        UtxoSet::<Test>::insert(genesis_hash, Utxo { value: 1_000, lock });
        TotalIssuance::<Test>::put(1_000);

        // Execute — must succeed with a real witness.
        Qutxo::execute_utxo_tx(RuntimeOrigin::none(), tx)
            .expect("valid signed tx must execute");

        // Invariant: TotalIssuance == sum(UTXO values).
        // Before: 1,000. After: 900 (spender output). The 100 fee was
        // captured by the FeeHandler, not minted as a UTXO yet.
        assert_eq!(
            TotalIssuance::<Test>::get(),
            900,
            "TotalIssuance must decrement by consumed and increment by created"
        );

        // Cross-check: no UTXO for the genesis hash remains.
        assert!(UtxoSet::<Test>::get(genesis_hash).is_none());
    });
}

#[test]
fn total_issuance_unchanged_on_failed_tx() {
    new_test_ext().execute_with(|| {
        use crate::{TotalIssuance, UtxoSet};
        use calibre_primitives::{TransactionInput, TransactionOutput, Utxo};

        let genesis_tx_hash = H256::from([43u8; 32]);
        let genesis_hash = Qutxo::calculate_utxo_hash(genesis_tx_hash, 0);
        UtxoSet::<Test>::insert(
            genesis_hash,
            Utxo { value: 1_000, lock: QuantumLock::AegisThreshold([0u8; 32]) },
        );
        TotalIssuance::<Test>::put(1_000);

        // Build a tx with a bad witness (mock_witness has empty pub_keys).
        let tx = Transaction {
            inputs: vec![TransactionInput { tx_hash: genesis_tx_hash, output_index: 0 }],
            outputs: vec![TransactionOutput {
                value: 900,
                lock: QuantumLock::AegisThreshold([1u8; 32]),
            }],
            pq_signature: vec![],
            witness: mock_witness(),
        };

        let result = Qutxo::execute_utxo_tx(RuntimeOrigin::none(), tx);
        assert!(result.is_err(), "bad witness must fail");

        // No state change on failure.
        assert_eq!(TotalIssuance::<Test>::get(), 1_000);
        assert!(UtxoSet::<Test>::get(genesis_hash).is_some());
    });
}
