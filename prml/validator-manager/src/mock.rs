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
    additional_traits::DummyDispatchVerifier, dispatch, impl_outer_event, impl_outer_origin,
    parameter_types, weights::Weight,
};
use sp_core::H256;
use sp_io::TestExternalities;
use sp_runtime::{
    impl_opaque_keys,
    testing::{Header, UintAuthorityId},
    traits::{BlakeTwo256, ConvertInto, IdentityLookup, OpaqueKeys},
    KeyTypeId, Perbill,
};

pub type MockSystem = frame_system::Module<Test>;
pub type MockSession = pallet_session::Module<Test>;
pub type ValidatorManager = Module<Test>;
pub type DummyValidatorId = u64;

impl_opaque_keys! {
    pub struct MockSessionKeys {
        pub dummy: UintAuthorityId,
    }
}

impl_outer_origin! {
    pub enum Origin for Test  where system = frame_system {}
}

mod poa {
    pub use crate::Event;
}

impl_outer_event! {
    pub enum TestEvent for Test {
        frame_system,
        pallet_session,
        poa<T>,
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct Test;

parameter_types! {
    pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(33);
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: Weight = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::one();
    pub const MinimumValidatorCount: u32 = 1;
}

impl pallet_session::Trait for Test {
    type SessionManager = ValidatorManager;
    type Keys = UintAuthorityId;
    type ShouldEndSession = TestShouldEndSession;
    type SessionHandler = TestSessionHandler;
    type Event = TestEvent;
    type ValidatorId = <Self as frame_system::Trait>::AccountId;
    type ValidatorIdOf = ConvertInto;
    type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
}

impl frame_system::Trait for Test {
    type Origin = Origin;
    type Index = u64;
    type BlockNumber = u64;
    type Call = ();
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = DummyValidatorId;
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
    type MinimumValidatorCount = MinimumValidatorCount;
}

pub struct TestShouldEndSession;
impl pallet_session::ShouldEndSession<u64> for TestShouldEndSession {
    fn should_end_session(_: u64) -> bool {
        true
    }
}

pub struct TestSessionHandler;
impl pallet_session::SessionHandler<DummyValidatorId> for TestSessionHandler {
    const KEY_TYPE_IDS: &'static [KeyTypeId] = &[sp_runtime::key_types::DUMMY];
    fn on_genesis_session<Ks: OpaqueKeys>(_: &[(DummyValidatorId, Ks)]) {}
    fn on_new_session<Ks: OpaqueKeys>(
        _: bool,
        _: &[(DummyValidatorId, Ks)],
        _: &[(DummyValidatorId, Ks)],
    ) {
    }
    fn on_before_session_ending() {}
    fn on_disabled(_: usize) {}
}

#[derive(Default)]
pub struct ExtBuilder {
    validators: Vec<DummyValidatorId>,
}

impl ExtBuilder {
    pub fn validator(mut self, validator: DummyValidatorId) -> Self {
        self.validators.push(validator);
        self
    }

    pub fn build(self) -> TestExternalities {
        let mut t = frame_system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();
        GenesisConfig::<Test> {
            validators: self.validators,
        }
        .assimilate_storage(&mut t)
        .unwrap();
        t.into()
    }
}

// Just a wrapper for set_keys extrinsics
pub fn set_keys(validator: DummyValidatorId) -> dispatch::DispatchResult {
    MockSession::set_keys(
        Origin::signed(validator),
        UintAuthorityId(validator),
        Vec::new(),
    )
}
