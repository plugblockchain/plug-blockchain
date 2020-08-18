// Copyright 2020 Plug New Zealand Limited
// This file is part of Plug.

// Plug is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Plug is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Plug. If not, see <http://www.gnu.org/licenses/>.

use super::*;
use frame_support::{
	additional_traits::DummyDispatchVerifier, impl_outer_event, impl_outer_origin, parameter_types, weights::Weight,
};
use sp_core::H256;
use sp_io::TestExternalities;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	Perbill,
};

pub type System = frame_system::Module<Test>;
pub type ConsortiumPermission = Module<Test>;
pub type AccountId = u64;

/// Reserved topic name for access to submit an extrinsic.
pub const ACCESS_TOPIC: &[u8; 6] = b"access";
/// Reserved value for access to submit an extrinsic.
pub const ACCESS_VALUE: u8 = 1;

impl_outer_origin! {
	pub enum Origin for Test  where system = frame_system {}
}

mod consortium_permission {
	pub use crate::Event;
}

impl_outer_event! {
	pub enum TestEvent for Test {
		frame_system,
		consortium_permission<T>,
	}
}

pub struct IssuerPermissionsMock;

impl IssuerPermissions for IssuerPermissionsMock {
	type AccountId = AccountId;
	/// Give a new issuer access = true.
	fn grant_issuer_permissions(issuer: &Self::AccountId) {
		ConsortiumPermission::do_make_claim(
			issuer,
			issuer,
			ACCESS_TOPIC.to_vec().as_ref(),
			vec![ACCESS_VALUE].as_ref(),
		);
	}
	/// Remove all self-claimed permissions from an issuer.
	fn revoke_issuer_permissions(issuer: &Self::AccountId) {
		let (claim_issuer, _) = ConsortiumPermission::claim((issuer, ACCESS_TOPIC.to_vec()));
		if claim_issuer == *issuer {
			ConsortiumPermission::do_revoke_claim(*issuer, ACCESS_TOPIC.to_vec());
		}
	}
}

#[derive(Clone, Eq, PartialEq)]
pub struct Test;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: Weight = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
	pub const MaximumTopicSize: usize = 32;
	pub const MaximumValueSize: usize = 32;
}

impl frame_system::Trait for Test {
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = u64;
	type Call = ();
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = TestEvent;
	type BlockHashCount = BlockHashCount;
	type MaximumBlockWeight = MaximumBlockWeight;
	type AvailableBlockRatio = AvailableBlockRatio;
	type MaximumBlockLength = MaximumBlockLength;
	type Doughnut = ();
	type DelegatedDispatchVerifier = DummyDispatchVerifier<Self::Doughnut, Self::AccountId>;
	type Version = ();
	type ModuleToIndex = ();
}

impl Trait for Test {
	type Event = TestEvent;
	type MaximumTopicSize = MaximumTopicSize;
	type MaximumValueSize = MaximumValueSize;
	type IssuerPermissions = IssuerPermissionsMock;
}

#[derive(Default)]
pub struct ExtBuilder {
	issuers: Vec<AccountId>,
	topics: (Vec<Topic>, Vec<bool>),
	genesis_issuers: Vec<AccountId>,
	genesis_topics: Vec<Topic>,
}

impl ExtBuilder {
	pub fn issuer(mut self, issuer: AccountId) -> Self {
		if !self.issuers.contains(&issuer) {
			self.issuers.push(issuer);
		}
		self
	}

	pub fn topic(mut self, topic: &[u8], enabled: bool) -> Self {
		if !self.topics.0.contains(&topic.to_owned()) {
			self.topics.0.push(topic.to_owned());
			self.topics.1.push(enabled);
		}
		self
	}

	pub fn genesis_issuer(mut self, issuer: AccountId) -> Self {
		self.genesis_issuers.push(issuer);
		self
	}

	pub fn genesis_topic(mut self, topic: &[u8]) -> Self {
		self.genesis_topics.push(topic.to_owned());
		self
	}

	pub fn build(self) -> TestExternalities {
		let mut t: TestExternalities = frame_system::GenesisConfig::default()
			.build_storage::<Test>()
			.unwrap()
			.into();
		t.execute_with(|| {
			<crate::Issuers<Test>>::put(self.issuers);
			<crate::Topics>::put(&self.topics.0);
			for i in 0..self.topics.0.len() {
				<crate::TopicEnabled>::insert(&self.topics.0[i], self.topics.1[i]);
			}
			if !self.genesis_issuers.is_empty() {
				ConsortiumPermission::initialise_issuers(&self.genesis_issuers)
			}
			if !self.genesis_topics.is_empty() {
				ConsortiumPermission::initialise_topics(&self.genesis_topics)
			}
		});
		t
	}
}
