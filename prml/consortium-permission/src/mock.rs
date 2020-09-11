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
    additional_traits::DummyDispatchVerifier, impl_outer_event, impl_outer_origin, parameter_types,
    weights::Weight,
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

pub struct IssuerPermissionsMock;

impl IssuerPermissions for IssuerPermissionsMock {
    type AccountId = AccountId;
    type Topic = Topic;
    /// When an issuer is authorized to make claims on the "access" topic, also grant themself the
    /// "access" permission.
    fn grant_issuer_permissions(issuer: &Self::AccountId, topic: &Topic) {
        if *topic == ACCESS_TOPIC {
            ConsortiumPermission::do_make_claim(
                issuer,
                issuer,
                ACCESS_TOPIC.to_vec().as_ref(),
                vec![ACCESS_VALUE].as_ref(),
            );
        }
    }
    /// When an issuer's authority on the "access" topic is revoked, also revoke their self-claimed
    /// "access" permission.
    fn revoke_issuer_permissions(issuer: &Self::AccountId, topic: &Topic) {
        if *topic == ACCESS_TOPIC {
            let (claim_issuer, _) = ConsortiumPermission::claim((issuer, ACCESS_TOPIC.to_vec()));
            if claim_issuer == *issuer {
                ConsortiumPermission::do_revoke_claim(*issuer, ACCESS_TOPIC.to_vec());
            }
        }
    }
}

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
    issuers: Vec<(AccountId, Vec<Topic>)>,
    topics: (Vec<Topic>, Vec<bool>),
    genesis_issuers: Vec<(AccountId, Vec<Topic>)>,
    genesis_topics: Vec<Topic>,
}

impl ExtBuilder {
    pub fn issuer(mut self, issuer: Vec<(AccountId, Vec<Topic>)>) -> Self {
        for i in issuer.into_iter() {
            if !self.issuers.contains(&i) {
                self.issuers.push(i);
            }
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

    pub fn genesis_issuer(mut self, issuer: Vec<(AccountId, Vec<Topic>)>) -> Self {
        for i in issuer.into_iter() {
            self.genesis_issuers.push(i);
        }
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
            if !self.genesis_topics.is_empty() {
                ConsortiumPermission::initialise_topics(&self.genesis_topics)
            }
            for i in 0..self.topics.0.len() {
                let _=ConsortiumPermission::insert_topic(&self.topics.0[i]);
                let _=ConsortiumPermission::update_topic(&self.topics.0[i], self.topics.1[i]);
            }
            if !self.genesis_issuers.is_empty() {
                ConsortiumPermission::initialise_issuers(&self.genesis_issuers)
            }
            for i in self.issuers.into_iter() {
                <crate::Issuers<Test>>::insert(i.0, i.1);
            }
        });
        t
    }
}
