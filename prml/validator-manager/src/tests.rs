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
use crate::mock::*;
use frame_support::{assert_noop, assert_ok};
use pallet_session::SessionManager;
use sp_runtime::DispatchError::BadOrigin;

const ALICE: DummyValidatorId = 0;
const BOB: DummyValidatorId = 1;
const CHARLIE: DummyValidatorId = 2;

#[test]
fn add_requires_session_keys_to_be_set() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            ValidatorManager::add(Origin::ROOT, ALICE),
            Error::<Test>::SessionKeysNotSet
        );
    });
}

#[test]
fn add_requires_root_origin() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(set_keys(ALICE));
        assert_ok!(ValidatorManager::add(Origin::ROOT, ALICE));
        assert_eq!(ValidatorManager::validators(), vec![ALICE]);
        assert_noop!(ValidatorManager::add(Origin::NONE, ALICE), BadOrigin);
        assert_noop!(ValidatorManager::add(Origin::signed(1), ALICE), BadOrigin);
    });
}

#[test]
fn add_rejects_validators_in_list() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(set_keys(ALICE));
        assert_ok!(ValidatorManager::add(Origin::ROOT, ALICE));
        assert_noop!(
            ValidatorManager::add(Origin::ROOT, ALICE),
            Error::<Test>::ValidatorAlreadyAdded,
        );
        assert_eq!(ValidatorManager::validators(), vec![ALICE]);
        assert_ok!(set_keys(BOB));
        assert_ok!(ValidatorManager::add(Origin::ROOT, BOB));
        assert_noop!(
            ValidatorManager::add(Origin::ROOT, BOB),
            Error::<Test>::ValidatorAlreadyAdded,
        );
        assert_eq!(ValidatorManager::validators(), vec![ALICE, BOB]);
    });
}

#[test]
fn add_event_works() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(set_keys(ALICE));
        assert_ok!(set_keys(BOB));
        assert_ok!(ValidatorManager::add(Origin::ROOT, ALICE));
        assert_ok!(ValidatorManager::add(Origin::ROOT, BOB));

        let events = MockSystem::events();
        assert_eq!(events[0].event, TestEvent::poa(RawEvent::Added(ALICE)));
        assert_eq!(events[1].event, TestEvent::poa(RawEvent::Added(BOB)));
    });
}

#[test]
fn remove_requires_root_origin() {
    ExtBuilder::default()
        .validator(ALICE)
        .build()
        .execute_with(|| {
            assert_ok!(set_keys(BOB));
            assert_ok!(ValidatorManager::add(Origin::ROOT, BOB));
            assert_eq!(ValidatorManager::validators(), vec![ALICE, BOB]);
            assert_ok!(ValidatorManager::remove(Origin::ROOT, BOB));
            assert_eq!(ValidatorManager::validators(), vec![ALICE]);
            assert_noop!(ValidatorManager::remove(Origin::NONE, BOB), BadOrigin);
            assert_noop!(ValidatorManager::remove(Origin::signed(1), BOB), BadOrigin);
        });
}

#[test]
fn remove_returns_error_on_non_existing_validator() {
    ExtBuilder::default()
        .validator(ALICE)
        .build()
        .execute_with(|| {
            let validator = 7357;
            assert_noop!(
                ValidatorManager::remove(Origin::ROOT, validator),
                Error::<Test>::ValidatorNotFound,
            );
        });
}

#[test]
fn remove_rejects_below_minium_validator_count() {
    ExtBuilder::default()
        .validator(ALICE)
        .build()
        .execute_with(|| {
            assert_noop!(
                ValidatorManager::remove(Origin::ROOT, ALICE),
                Error::<Test>::MinimumValidatorCount,
            );
        });
}

#[test]
fn remove_event_works() {
    ExtBuilder::default()
        .validator(ALICE)
        .validator(BOB)
        .validator(CHARLIE)
        .build()
        .execute_with(|| {
            assert_ok!(ValidatorManager::remove(Origin::ROOT, ALICE));
            assert_ok!(ValidatorManager::remove(Origin::ROOT, BOB));

            let events = MockSystem::events();
            assert_eq!(events[0].event, TestEvent::poa(RawEvent::Removed(ALICE)));
            assert_eq!(events[1].event, TestEvent::poa(RawEvent::Removed(BOB)));
        });
}

#[test]
fn first_session_returns_none() {
    ExtBuilder::default().build().execute_with(|| {
        let session = MockSession::current_index();
        assert_eq!(session, 0);
        assert_eq!(ValidatorManager::new_session(session), None);
        assert_eq!(ValidatorManager::validators(), Vec::new());
    });
}

#[test]
fn new_session_returns_none_if_list_is_not_updated() {
    ExtBuilder::default()
        .validator(ALICE)
        .build()
        .execute_with(|| {
            assert_eq!(MockSession::current_index(), 0);
            let _ = MockSession::rotate_session();
            assert_eq!(MockSession::current_index(), 1);
            assert_eq!(ValidatorManager::new_session(1), None);
            let _ = MockSession::rotate_session();
            assert_eq!(MockSession::current_index(), 2);
            assert_eq!(ValidatorManager::new_session(2), None);
        });
}

#[test]
fn new_session_returns_none_if_list_remains_unchanged() {
    ExtBuilder::default()
        .validator(ALICE)
        .build()
        .execute_with(|| {
            assert_eq!(MockSession::current_index(), 0);
            let _ = MockSession::rotate_session();

            // Session 1.
            let session_index = MockSession::current_index();
            assert_eq!(session_index, 1);
            assert_eq!(ValidatorManager::new_session(session_index), None);

            // Session 2: add BOB
            let _ = MockSession::rotate_session();
            let session_index = MockSession::current_index();
            assert_eq!(session_index, 2);
            assert_ok!(set_keys(BOB));
            assert_ok!(ValidatorManager::add(Origin::ROOT, BOB));
            assert_ok!(ValidatorManager::remove(Origin::ROOT, BOB));
            assert_eq!(ValidatorManager::new_session(session_index), None);
        });
}

#[test]
fn new_session_returns_some_validators_if_updated() {
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(MockSession::current_index(), 0);
        let _ = MockSession::rotate_session();

        // Session 1. add ALICE
        let session_index = MockSession::current_index();
        assert_eq!(session_index, 1);
        assert_eq!(ValidatorManager::queued_validators(), vec![]);
        assert_ok!(set_keys(ALICE));
        assert_ok!(ValidatorManager::add(Origin::ROOT, ALICE));
        assert_eq!(
            ValidatorManager::new_session(session_index),
            Some(vec![ALICE])
        );

        // Session 2: add BOB and CHARLIE
        let _ = MockSession::rotate_session();
        let session_index = MockSession::current_index();
        assert_eq!(session_index, 2);
        assert_eq!(ValidatorManager::queued_validators(), vec![ALICE]);
        assert_ok!(set_keys(BOB));
        assert_ok!(set_keys(CHARLIE));
        assert_ok!(ValidatorManager::add(Origin::ROOT, BOB));
        assert_ok!(ValidatorManager::add(Origin::ROOT, CHARLIE));
        assert_eq!(
            ValidatorManager::new_session(session_index),
            Some(vec![ALICE, BOB, CHARLIE]),
        );

        // Session 3: remove BOB and CHARLIE
        let _ = MockSession::rotate_session();
        let session_index = MockSession::current_index();
        assert_eq!(session_index, 3);
        assert_eq!(
            ValidatorManager::queued_validators(),
            vec![ALICE, BOB, CHARLIE]
        );
        assert_ok!(ValidatorManager::remove(Origin::ROOT, BOB));
        assert_ok!(ValidatorManager::remove(Origin::ROOT, CHARLIE));
        assert_eq!(ValidatorManager::validators(), vec![ALICE]);
        assert_eq!(
            ValidatorManager::new_session(session_index),
            Some(vec![ALICE])
        );

        // Session 4: no changes
        let _ = MockSession::rotate_session();
        assert_eq!(MockSession::validators(), vec![ALICE, BOB, CHARLIE]);
        assert_eq!(ValidatorManager::validators(), vec![ALICE]);

        // Session 5
        let _ = MockSession::rotate_session();
        assert_eq!(MockSession::validators(), vec![ALICE]);
    });
}
