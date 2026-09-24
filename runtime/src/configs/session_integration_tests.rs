use super::BabeFindAuthor;
use crate::{
    AccountId, Babe, BalancesConfig, Grandpa, Runtime, RuntimeCall, RuntimeGenesisConfig,
    RuntimeOrigin, Session, SessionConfig, SessionKeys, Stake, System,
};
use codec::Encode;
use frame_support::{
    assert_noop, assert_ok,
    traits::{FindAuthor, Hooks},
};
use sp_consensus_babe::{
    digests::{PreDigest, SecondaryPlainPreDigest},
    BABE_ENGINE_ID,
};
use sp_keyring::{Ed25519Keyring, Sr25519Keyring};
use sp_runtime::{
    generic::{Digest, DigestItem},
    traits::Dispatchable,
    BuildStorage,
};

const INITIAL: [Sr25519Keyring; 7] = [
    Sr25519Keyring::Alice,
    Sr25519Keyring::Bob,
    Sr25519Keyring::Charlie,
    Sr25519Keyring::Dave,
    Sr25519Keyring::Eve,
    Sr25519Keyring::Ferdie,
    Sr25519Keyring::One,
];

fn account(key: Sr25519Keyring) -> AccountId {
    key.to_account_id()
}

fn grandpa_identity(key: Sr25519Keyring) -> Ed25519Keyring {
    match key {
        Sr25519Keyring::Alice => Ed25519Keyring::Alice,
        Sr25519Keyring::Bob => Ed25519Keyring::Bob,
        Sr25519Keyring::Charlie => Ed25519Keyring::Charlie,
        Sr25519Keyring::Dave => Ed25519Keyring::Dave,
        Sr25519Keyring::Eve => Ed25519Keyring::Eve,
        Sr25519Keyring::Ferdie => Ed25519Keyring::Ferdie,
        Sr25519Keyring::One => Ed25519Keyring::One,
        Sr25519Keyring::Two => Ed25519Keyring::Two,
        _ => unreachable!("test uses only known development identities"),
    }
}

fn session_keys(sr: Sr25519Keyring, ed: Ed25519Keyring) -> SessionKeys {
    SessionKeys {
        babe: sr.public().into(),
        grandpa: ed.public().into(),
    }
}

fn test_ext() -> sp_io::TestExternalities {
    let mut genesis = RuntimeGenesisConfig::default();
    genesis.balances = BalancesConfig {
        balances: INITIAL
            .iter()
            .copied()
            .chain([Sr25519Keyring::Two])
            .map(|key| (account(key), 1u128 << 60))
            .collect(),
        ..Default::default()
    };
    genesis.session = SessionConfig {
        keys: INITIAL
            .iter()
            .copied()
            .map(|key| {
                let who = account(key);
                (who.clone(), who, session_keys(key, grandpa_identity(key)))
            })
            .collect(),
        non_authority_keys: vec![],
    };
    sp_io::TestExternalities::new(genesis.build_storage().expect("valid test genesis"))
}

fn authority_accounts() -> Vec<AccountId> {
    Session::validators()
}

fn run_block(block: u32) {
    // Jump one full 600-slot epoch between synthetic blocks. The normal BABE
    // and session hooks then perform the same boundary transitions as a live
    // chain without waiting six hours in a runtime test.
    let slot = 1 + (block as u64 - 1) * 600;
    let pre_digest = PreDigest::SecondaryPlain(SecondaryPlainPreDigest {
        authority_index: 0,
        slot: slot.into(),
    });
    let digest = Digest {
        logs: vec![DigestItem::PreRuntime(BABE_ENGINE_ID, pre_digest.encode())],
    };
    System::initialize(&block, &Default::default(), &digest);
    Babe::on_initialize(block);
    Session::on_initialize(block);
    // GRANDPA enacts its scheduled change at the end of the rotation block.
    Grandpa::on_finalize(block);
    Babe::on_finalize(block);
    System::finalize();
}

fn advance_session() {
    let block = System::block_number() + 1;
    run_block(block);
    assert_eq!(Session::current_index(), block - 1);
}

fn assert_authorities(expected: &[Sr25519Keyring]) {
    let accounts: Vec<_> = expected.iter().copied().map(account).collect();
    assert_eq!(authority_accounts(), accounts);
    assert_eq!(Babe::authorities().len(), expected.len());
    assert_eq!(Grandpa::grandpa_authorities().len(), expected.len());
    for (i, key) in expected.iter().enumerate() {
        assert_eq!(Babe::authorities()[i].0, key.public().into());
        assert_eq!(
            Grandpa::grandpa_authorities()[i].0,
            grandpa_identity(*key).public().into()
        );
        assert_eq!(Grandpa::grandpa_authorities()[i].1, 1);
    }
}

#[test]
fn signed_key_registration_replaces_one_validator_after_election_delay() {
    test_ext().execute_with(|| {
        run_block(1);
        assert_authorities(&INITIAL);
        let replacement = Sr25519Keyring::Two;
        let replacement_account = account(replacement);
        assert_ok!(RuntimeCall::Session(pallet_session::Call::set_keys {
            keys: session_keys(replacement, Ed25519Keyring::Two),
            proof: vec![],
        })
        .dispatch(RuntimeOrigin::signed(replacement_account.clone())));
        assert!(pallet_session::NextKeys::<Runtime>::contains_key(
            &replacement_account
        ));

        for (index, key) in INITIAL[..6]
            .iter()
            .copied()
            .chain([replacement])
            .enumerate()
        {
            let who = account(key);
            pallet_stake::Stake::<Runtime>::insert(&who, (10 - index) as u128 * 10u128.pow(21));
            assert_ok!(Stake::join_candidates(RuntimeOrigin::signed(who)));
        }

        // Session 6 is first planned at the session-5 transition.
        for index in 1..=4 {
            advance_session();
            assert_eq!(Session::current_index(), index);
            assert_authorities(&INITIAL);
            assert_eq!(Session::queued_keys().len(), 7);
        }
        advance_session();
        assert_eq!(Session::current_index(), 5);
        assert_authorities(&INITIAL);
        let queued: Vec<_> = Session::queued_keys()
            .into_iter()
            .map(|(id, _)| id)
            .collect();
        assert_eq!(queued.len(), 7);
        assert!(queued.contains(&replacement_account));
        assert!(!queued.contains(&account(Sr25519Keyring::One)));

        advance_session();
        assert_eq!(Session::current_index(), 6);
        let expected = [
            Sr25519Keyring::Alice,
            Sr25519Keyring::Bob,
            Sr25519Keyring::Charlie,
            Sr25519Keyring::Dave,
            Sr25519Keyring::Eve,
            Sr25519Keyring::Ferdie,
            Sr25519Keyring::Two,
        ];
        assert_authorities(&expected);

        // The fee pallet's author resolver must follow the same account order.
        let digest = PreDigest::SecondaryPlain(SecondaryPlainPreDigest {
            authority_index: 6,
            slot: 1u64.into(),
        })
        .encode();
        assert_eq!(
            BabeFindAuthor::find_author([(BABE_ENGINE_ID, digest.as_slice())]),
            Some(replacement_account),
        );
    });
}

#[test]
fn missing_replacement_keys_keep_all_seven_current_authorities() {
    test_ext().execute_with(|| {
        run_block(1);
        for (index, key) in INITIAL[..6]
            .iter()
            .copied()
            .chain([Sr25519Keyring::Two])
            .enumerate()
        {
            let who = account(key);
            pallet_stake::Stake::<Runtime>::insert(&who, (10 - index) as u128 * 10u128.pow(21));
            assert_ok!(Stake::join_candidates(RuntimeOrigin::signed(who)));
        }
        for _ in 1..=6 {
            advance_session();
        }
        assert_eq!(Session::current_index(), 6);
        assert_eq!(Session::queued_keys().len(), 7);
        assert_authorities(&INITIAL);
    });
}

#[test]
fn signed_purge_cannot_remove_an_active_authority() {
    test_ext().execute_with(|| {
        run_block(1);
        let alice = account(Sr25519Keyring::Alice);
        assert_noop!(
            RuntimeCall::Session(pallet_session::Call::purge_keys {})
                .dispatch(RuntimeOrigin::signed(alice.clone())),
            frame_system::Error::<Runtime>::CallFiltered,
        );
        assert!(pallet_session::NextKeys::<Runtime>::contains_key(&alice));
        advance_session();
        assert_authorities(&INITIAL);
    });
}
