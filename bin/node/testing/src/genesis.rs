// This file is part of Substrate.

// Copyright (C) 2019-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Genesis Configuration.

use crate::keyring::*;
use sp_keyring::{Ed25519Keyring, Sr25519Keyring};
use node_runtime::{
	GenesisConfig, GenericAssetConfig, SessionConfig, StakingConfig, SystemConfig,
	GrandpaConfig, IndicesConfig, ContractsConfig, SocietyConfig, wasm_binary_unwrap,
	AccountId, StakerStatus,
};
use node_runtime::constants::{asset::*, currency::*};
use prml_generic_asset::AssetInfo;
use sp_core::ChangesTrieConfiguration;
use sp_runtime::Perbill;

/// Create genesis runtime configuration for tests.
pub fn config(support_changes_trie: bool, code: Option<&[u8]>) -> GenesisConfig {
	config_endowed(support_changes_trie, code, Default::default())
}

/// Create genesis runtime configuration for tests with some extra
/// endowed accounts.
pub fn config_endowed(
	support_changes_trie: bool,
	code: Option<&[u8]>,
	extra_endowed: Vec<AccountId>,
) -> GenesisConfig {

	let mut endowed = vec![alice(), bob(), charlie(), dave(), eve(), ferdie()];

	endowed.extend(extra_endowed.into_iter());

	GenesisConfig {
		frame_system: Some(SystemConfig {
			changes_trie_config: if support_changes_trie { Some(ChangesTrieConfiguration {
				digest_interval: 2,
				digest_levels: 2,
			}) } else { None },
			code: code.map(|x| x.to_vec()).unwrap_or_else(|| wasm_binary_unwrap().to_vec()),
		}),
		pallet_indices: Some(IndicesConfig {
			indices: vec![],
		}),
		prml_generic_asset: Some(GenericAssetConfig {
			assets: vec![CENNZ_ASSET_ID, CENTRAPAY_ASSET_ID],
			// Grant root key full permissions (mint,burn,update) on the following assets
			permissions: vec![(CENNZ_ASSET_ID, alice()), (CENTRAPAY_ASSET_ID, alice())],
			initial_balance: 100 * DOLLARS,
			endowed_accounts: endowed,
			next_asset_id: NEXT_ASSET_ID,
			staking_asset_id: STAKING_ASSET_ID,
			spending_asset_id: SPENDING_ASSET_ID,
			asset_meta: vec![
				(CENNZ_ASSET_ID, AssetInfo::new(b"CENNZ".to_vec(), 4, 1)),
				(CENTRAPAY_ASSET_ID, AssetInfo::new(b"CPAY".to_vec(), 4, 1)),
			],
		}),
		pallet_session: Some(SessionConfig {
			keys: vec![
				(dave(), alice(), to_session_keys(
					&Ed25519Keyring::Alice,
					&Sr25519Keyring::Alice,
				)),
				(eve(), bob(), to_session_keys(
					&Ed25519Keyring::Bob,
					&Sr25519Keyring::Bob,
				)),
				(ferdie(), charlie(), to_session_keys(
					&Ed25519Keyring::Charlie,
					&Sr25519Keyring::Charlie,
				)),
			]
		}),
		pallet_staking: Some(StakingConfig {
			stakers: vec![
				(dave(), alice(), 111 * DOLLARS, StakerStatus::Validator),
				(eve(), bob(), 100 * DOLLARS, StakerStatus::Validator),
				(ferdie(), charlie(), 100 * DOLLARS, StakerStatus::Validator)
			],
			validator_count: 3,
			minimum_validator_count: 0,
			slash_reward_fraction: Perbill::from_percent(10),
			invulnerables: vec![alice(), bob(), charlie()],
			.. Default::default()
		}),
		pallet_contracts: Some(ContractsConfig {
			current_schedule: Default::default(),
		}),
		pallet_babe: Some(Default::default()),
		pallet_grandpa: Some(GrandpaConfig {
			authorities: vec![],
		}),
		pallet_im_online: Some(Default::default()),
		pallet_authority_discovery: Some(Default::default()),
		pallet_democracy: Some(Default::default()),
		pallet_collective_Instance1: Some(Default::default()),
		pallet_collective_Instance2: Some(Default::default()),
		pallet_membership_Instance1: Some(Default::default()),
		pallet_elections_phragmen: Some(Default::default()),
		pallet_sudo: Some(Default::default()),
		pallet_treasury: Some(Default::default()),
		pallet_society: Some(SocietyConfig {
			members: vec![alice(), bob()],
			pot: 0,
			max_members: 999,
		}),
		pallet_vesting: Some(Default::default()),
		pallet_gilt: Some(Default::default()),
	}
}
