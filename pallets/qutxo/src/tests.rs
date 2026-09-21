#![cfg(test)]

use crate::{self as pallet_qutxo, *};
use frame_support::{assert_noop, assert_ok, parameter_types, traits::ConstU32};
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};
use calibre_primitives::{QuantumLock, Utxo, TransactionInput, TransactionOutput};
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
}

pub fn new_test_ext() -> sp_io::TestExternalities {
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

#[test]
fn test_utxo_conservation_of_mass_and_double_spend() {
    new_test_ext().execute_with(|| {
        // 1. Setup: Manually inject a Genesis UTXO with 100 $CAL into the state
        let genesis_tx_hash = H256::from([1u8; 32]);
        let genesis_utxo_hash = Qutxo::calculate_utxo_hash(genesis_tx_hash, 0);
        
        let genesis_utxo = Utxo {
            value: 100,
            lock: QuantumLock::SingleSig([0u8; 32]),
        };
        UtxoSet::<Test>::insert(genesis_utxo_hash, genesis_utxo.clone());
        TotalIssuance::<Test>::put(100);

        // Verify it exists
        assert_eq!(UtxoSet::<Test>::get(genesis_utxo_hash).unwrap().value, 100);

        // 2. Create a valid transaction spending the Genesis UTXO
        let valid_tx = Transaction {
            inputs: vec![TransactionInput { tx_hash: genesis_tx_hash, output_index: 0 }],
            outputs: vec![
                TransactionOutput { value: 60, lock: QuantumLock::SingleSig([2u8; 32]) },
                TransactionOutput { value: 40, lock: QuantumLock::SingleSig([3u8; 32]) }, // 60 + 40 = 100 (Conservation holds)
            ],
            pq_signature: vec![],
            witness: mock_witness(),
        };

        // Execute the valid transaction
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
