// Copyright 2019-2021
//     by  Centrality Investments Ltd.
//     and Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! Mocks for the module.

#![cfg(test)]

use super::*;
use crate as prml_generic_asset;
use crate::{NegativeImbalance, PositiveImbalance};
use frame_support::{parameter_types, weights::Weight};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	Perbill,
};

// test accounts
pub const ALICE: u64 = 1;
pub const BOB: u64 = 2;
pub const CHARLIE: u64 = 3;

// staking asset id
pub const STAKING_ASSET_ID: u32 = 16000;
// spending asset id
pub const SPENDING_ASSET_ID: u32 = 16001;
// pre-existing asset 1
pub const TEST1_ASSET_ID: u32 = 16003;
// pre-existing asset 2
pub const TEST2_ASSET_ID: u32 = 16004;
// default next asset id
pub const ASSET_ID: u32 = 1000;

// initial issuance for creating new asset
pub const INITIAL_ISSUANCE: u64 = 1000;
// iniital balance for seting free balance
pub const INITIAL_BALANCE: u64 = 100;

pub type PositiveImbalanceOf = PositiveImbalance<Test>;
pub type NegativeImbalanceOf = NegativeImbalance<Test>;

type Block = frame_system::mocking::MockBlock<Test>;
type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Module, Call, Config, Storage, Event<T>},
		GenericAsset: prml_generic_asset::{Module, Call, Storage, Config<T>, Event<T>}
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: Weight = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
}

impl frame_system::Config for Test {
	type BaseCallFilter = ();
	type Origin = Origin;
	type Index = u64;
	type Call = Call;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type BlockLength = ();
	type BlockWeights = ();
	type DbWeight = ();
	type Version = ();
	type AccountData = ();
	type PalletInfo = PalletInfo;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
}

impl Config for Test {
	type Balance = u64;
	type AssetId = u32;
	type Event = Event;
	type WeightInfo = ();
}

pub struct ExtBuilder {
	asset_id: u32,
	next_asset_id: u32,
	accounts: Vec<u64>,
	initial_balance: u64,
	permissions: Vec<(u32, u64)>,
}

// Returns default values for genesis config
impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			asset_id: 0,
			next_asset_id: ASSET_ID,
			accounts: vec![0],
			initial_balance: 0,
			permissions: vec![],
		}
	}
}

impl ExtBuilder {
	// Sets free balance to genesis config
	pub fn free_balance(mut self, free_balance: (u32, u64, u64)) -> Self {
		self.asset_id = free_balance.0;
		self.accounts = vec![free_balance.1];
		self.initial_balance = free_balance.2;
		self
	}

	pub fn permissions(mut self, permissions: Vec<(u32, u64)>) -> Self {
		self.permissions = permissions;
		self
	}

	pub fn next_asset_id(mut self, asset_id: u32) -> Self {
		self.next_asset_id = asset_id;
		self
	}

	// builds genesis config
	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

		prml_generic_asset::GenesisConfig::<Test> {
			assets: vec![self.asset_id],
			endowed_accounts: self.accounts,
			initial_balance: self.initial_balance,
			next_asset_id: self.next_asset_id,
			staking_asset_id: STAKING_ASSET_ID,
			spending_asset_id: SPENDING_ASSET_ID,
			permissions: self.permissions,
			asset_meta: vec![
				(TEST1_ASSET_ID, AssetInfo::new(b"TST1".to_vec(), 1)),
				(TEST2_ASSET_ID, AssetInfo::new(b"TST 2".to_vec(), 2)),
			],
		}.assimilate_storage(&mut t).unwrap();

		t.into()
	}
}

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
pub fn new_test_ext() -> sp_io::TestExternalities {
	frame_system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap()
		.into()
}
