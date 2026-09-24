// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::{
    AccountId, BalancesConfig, RuntimeGenesisConfig, SessionConfig, SessionKeys, SudoConfig,
};
use alloc::{vec, vec::Vec};
use frame_support::build_struct_json_patch;
use serde_json::Value;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_genesis_builder::{self, PresetId};
use sp_keyring::{Ed25519Keyring, Sr25519Keyring};

// Returns the genesis config presets populated with given parameters.
fn testnet_genesis(
    initial_authorities: Vec<(AccountId, BabeId, GrandpaId)>,
    endowed_accounts: Vec<AccountId>,
    root: AccountId,
) -> Value {
    build_struct_json_patch!(RuntimeGenesisConfig {
        balances: BalancesConfig {
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, 1u128 << 60))
                .collect::<Vec<_>>(),
        },
        babe: pallet_babe::GenesisConfig {
            // Session owns authority initialization when pallet-session is enabled.
            authorities: vec![],
        },
        grandpa: pallet_grandpa::GenesisConfig {
            // Session owns authority initialization when pallet-session is enabled.
            authorities: vec![],
        },
        session: SessionConfig {
            keys: initial_authorities
                .iter()
                .map(|(account, babe, grandpa)| {
                    (
                        account.clone(),
                        account.clone(),
                        SessionKeys {
                            babe: babe.clone(),
                            grandpa: grandpa.clone(),
                        },
                    )
                })
                .collect::<Vec<_>>(),
        },
        sudo: SudoConfig { key: Some(root) },
    })
}

/// Return the development genesis config.
pub fn development_config_genesis() -> Value {
    testnet_genesis(
        vec![(
            sp_keyring::Sr25519Keyring::Alice.to_account_id(),
            sp_keyring::Sr25519Keyring::Alice.public().into(),
            sp_keyring::Ed25519Keyring::Alice.public().into(),
        )],
        vec![
            Sr25519Keyring::Alice.to_account_id(),
            Sr25519Keyring::Bob.to_account_id(),
            Sr25519Keyring::AliceStash.to_account_id(),
            Sr25519Keyring::BobStash.to_account_id(),
        ],
        sp_keyring::Sr25519Keyring::Alice.to_account_id(),
    )
}

/// Return the local genesis config preset.
pub fn local_config_genesis() -> Value {
    testnet_genesis(
        vec![
            (
                sp_keyring::Sr25519Keyring::Alice.to_account_id(),
                sp_keyring::Sr25519Keyring::Alice.public().into(),
                sp_keyring::Ed25519Keyring::Alice.public().into(),
            ),
            (
                sp_keyring::Sr25519Keyring::Bob.to_account_id(),
                sp_keyring::Sr25519Keyring::Bob.public().into(),
                sp_keyring::Ed25519Keyring::Bob.public().into(),
            ),
        ],
        Sr25519Keyring::iter()
            .filter(|v| v != &Sr25519Keyring::One && v != &Sr25519Keyring::Two)
            .map(|v| v.to_account_id())
            .collect::<Vec<_>>(),
        Sr25519Keyring::Alice.to_account_id(),
    )
}

/// Chain spec identifier for the Calibre staging network (7 authorities).
pub const CALIBRE_STAGING_RUNTIME_PRESET: &str = "calibre_staging";

/// Return the Calibre staging genesis config: 7 BABE + 7 GRANDPA authorities
/// using well-known `sp_keyring` identities (Alice..Ferdie + One).
///
/// NOT for production: these keys are public and live in `sp_keyring`.
/// Purpose: exercise 7-validator consensus on LAN before real keygen tooling
/// lands. Each validator inserts its own seed (`//Alice`, `//Bob`, ...) into
/// its local keystore via `key insert`.
pub fn staging_testnet_genesis() -> Value {
    let authorities: Vec<(Sr25519Keyring, Ed25519Keyring)> = vec![
        (Sr25519Keyring::Alice, Ed25519Keyring::Alice),
        (Sr25519Keyring::Bob, Ed25519Keyring::Bob),
        (Sr25519Keyring::Charlie, Ed25519Keyring::Charlie),
        (Sr25519Keyring::Dave, Ed25519Keyring::Dave),
        (Sr25519Keyring::Eve, Ed25519Keyring::Eve),
        (Sr25519Keyring::Ferdie, Ed25519Keyring::Ferdie),
        (Sr25519Keyring::One, Ed25519Keyring::One),
    ];
    let initial_authorities: Vec<(AccountId, BabeId, GrandpaId)> = authorities
        .iter()
        .map(|(s, e)| (s.to_account_id(), s.public().into(), e.public().into()))
        .collect();
    let endowed_accounts: Vec<AccountId> =
        authorities.iter().map(|(s, _)| s.to_account_id()).collect();
    let root = Sr25519Keyring::Alice.to_account_id();
    testnet_genesis(initial_authorities, endowed_accounts, root)
}

/// Provides the JSON representation of predefined genesis config for given `id`.
pub fn get_preset(id: &PresetId) -> Option<Vec<u8>> {
    let patch = match id.as_ref() {
        sp_genesis_builder::DEV_RUNTIME_PRESET => development_config_genesis(),
        sp_genesis_builder::LOCAL_TESTNET_RUNTIME_PRESET => local_config_genesis(),
        CALIBRE_STAGING_RUNTIME_PRESET => staging_testnet_genesis(),
        _ => return None,
    };
    Some(
        serde_json::to_string(&patch)
            .expect("serialization to json is expected to work. qed.")
            .into_bytes(),
    )
}

/// List of supported presets.
pub fn preset_names() -> Vec<PresetId> {
    vec![
        PresetId::from(sp_genesis_builder::DEV_RUNTIME_PRESET),
        PresetId::from(sp_genesis_builder::LOCAL_TESTNET_RUNTIME_PRESET),
        PresetId::from(CALIBRE_STAGING_RUNTIME_PRESET),
    ]
}
