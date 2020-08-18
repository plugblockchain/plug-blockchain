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
use sp_runtime::DispatchError::BadOrigin;

const ALICE: AccountId = 0;
const BOB: AccountId = 1;
const CHARLIE: AccountId = 2;

// Issuers

#[test]
fn initialise_issuers_works() {
	ExtBuilder::default()
		.genesis_topic(ACCESS_TOPIC)
		.genesis_issuer(ALICE)
		.build()
		.execute_with(|| {
			assert_eq!(ConsortiumPermission::issuers().contains(&ALICE), true);
			assert_eq!(
				ConsortiumPermission::claim((ALICE, ACCESS_TOPIC.to_vec())),
				(ALICE, vec![ACCESS_VALUE])
			);
		})
}

#[test]
fn add_issuer_requires_root() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(ConsortiumPermission::add_issuer(Origin::signed(ALICE), BOB), BadOrigin);
		assert_ok!(ConsortiumPermission::add_issuer(Origin::ROOT, BOB));
	});
}

#[test]
fn add_issuer_rejects_adding_existing_account() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(ConsortiumPermission::add_issuer(Origin::ROOT, ALICE));
		assert_noop!(
			ConsortiumPermission::add_issuer(Origin::ROOT, ALICE),
			Error::<Test>::IssuerExists
		);
	});
}

#[test]
fn add_issuer_populates_storage() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(ConsortiumPermission::add_issuer(Origin::ROOT, ALICE));
		assert_eq!(ConsortiumPermission::issuers(), vec![ALICE]);
		assert_ok!(ConsortiumPermission::add_issuer(Origin::ROOT, BOB));
		assert_eq!(ConsortiumPermission::issuers(), vec![ALICE, BOB]);
	});
}

#[test]
fn add_issuer_emits_events() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(ConsortiumPermission::add_issuer(Origin::ROOT, ALICE));
		assert_ok!(ConsortiumPermission::add_issuer(Origin::ROOT, BOB));
		let events = System::events();
		assert_eq!(events[0].event, TestEvent::consortium_permission(RawEvent::IssuerAdded(ALICE)));
		assert_eq!(events[1].event, TestEvent::consortium_permission(RawEvent::IssuerAdded(BOB)));
	});
}

#[test]
fn added_issuer_has_access_true() {
	ExtBuilder::default()
		.topic(ACCESS_TOPIC, true)
		.build()
		.execute_with(|| {
			assert_ok!(ConsortiumPermission::add_issuer(Origin::ROOT, BOB));
			assert_eq!(
				ConsortiumPermission::claim((BOB, ACCESS_TOPIC.to_vec())),
				(BOB, vec![ACCESS_VALUE])
			);
		});
}

#[test]
fn remove_issuer_requires_root() {
	ExtBuilder::default().issuer(BOB).build().execute_with(|| {
		assert_noop!(ConsortiumPermission::remove_issuer(Origin::signed(ALICE), BOB), BadOrigin);
		assert_ok!(ConsortiumPermission::remove_issuer(Origin::ROOT, BOB));
	});
}

#[test]
fn remove_issuer_rejects_removing_unknown_account() {
	ExtBuilder::default().issuer(BOB).build().execute_with(|| {
		assert_ok!(ConsortiumPermission::remove_issuer(Origin::ROOT, BOB));
		assert_noop!(
			ConsortiumPermission::remove_issuer(Origin::ROOT, ALICE),
			Error::<Test>::NotAnIssuer
		);
	});
}

#[test]
fn remove_issuer_populates_storage() {
	ExtBuilder::default()
		.issuer(ALICE)
		.issuer(BOB)
		.build()
		.execute_with(|| {
			assert_eq!(ConsortiumPermission::issuers(), vec![ALICE, BOB]);
			assert_ok!(ConsortiumPermission::remove_issuer(Origin::ROOT, ALICE));
			assert_eq!(ConsortiumPermission::issuers(), vec![BOB]);
			assert_ok!(ConsortiumPermission::remove_issuer(Origin::ROOT, BOB));
			assert_eq!(ConsortiumPermission::issuers(), vec![]);
		});
}

#[test]
fn remove_issuer_emits_events() {
	ExtBuilder::default()
		.issuer(ALICE)
		.issuer(BOB)
		.build()
		.execute_with(|| {
			assert_ok!(ConsortiumPermission::remove_issuer(Origin::ROOT, ALICE));
			assert_ok!(ConsortiumPermission::remove_issuer(Origin::ROOT, BOB));
			let events = System::events();
			assert_eq!(
				events[0].event,
				TestEvent::consortium_permission(RawEvent::IssuerRemoved(ALICE))
			);
			assert_eq!(events[1].event, TestEvent::consortium_permission(RawEvent::IssuerRemoved(BOB)));
		});
}

#[test]
fn removed_issuer_loses_self_assigned_access() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(ConsortiumPermission::add_issuer(Origin::ROOT, BOB));
		assert_ok!(ConsortiumPermission::remove_issuer(Origin::ROOT, BOB));
		assert_eq!(ConsortiumPermission::holder_claims(BOB), Vec::<Topic>::default());
	});
}

#[test]
fn removed_issuer_does_not_lose_access_assigned_by_another_issuer() {
	ExtBuilder::default()
		.issuer(ALICE)
		.topic(b"access", true)
		.build()
		.execute_with(|| {
			assert_ok!(ConsortiumPermission::add_issuer(Origin::ROOT, BOB));
			assert_ok!(ConsortiumPermission::make_claim(
				Origin::signed(ALICE),
				BOB,
				b"access".to_vec(),
				vec![0x1]
			));

			assert_ok!(ConsortiumPermission::remove_issuer(Origin::ROOT, BOB));
			assert_eq!(ConsortiumPermission::holder_claims(BOB), [b"access".to_vec()]);
		});
}

// Claims

#[test]
fn claim_extrinsics_must_be_signed() {
	ExtBuilder::default().issuer(ALICE).build().execute_with(|| {
		let topic = String::from("access").into_bytes();
		assert_noop!(
			ConsortiumPermission::make_claim(Origin::NONE, CHARLIE, topic.clone(), vec![0x1]),
			BadOrigin
		);
		assert_noop!(
			ConsortiumPermission::revoke_claim(Origin::NONE, CHARLIE, topic.clone()),
			BadOrigin
		);
	});
}

#[test]
fn claim_cannot_be_made_by_non_issuer() {
	ExtBuilder::default().issuer(ALICE).build().execute_with(|| {
		assert_noop!(
			ConsortiumPermission::make_claim(
				Origin::signed(BOB),
				CHARLIE,
				String::from("access").into_bytes(),
				vec![0x1]
			),
			Error::<Test>::NotAnIssuer
		);
	});
}

#[test]
fn claim_cannot_be_made_by_non_existent_topics() {
	ExtBuilder::default().issuer(ALICE).build().execute_with(|| {
		assert_noop!(
			ConsortiumPermission::make_claim(
				Origin::signed(ALICE),
				CHARLIE,
				String::from("fake-topic").into_bytes(),
				vec![0x1]
			),
			Error::<Test>::InvalidTopic
		);
	});
}

#[test]
fn claim_cannot_be_made_by_disabled_topics() {
	ExtBuilder::default()
		.issuer(ALICE)
		.topic(b"disabled-topic", false)
		.build()
		.execute_with(|| {
			assert_noop!(
				ConsortiumPermission::make_claim(
					Origin::signed(ALICE),
					CHARLIE,
					String::from("disabled-topic").into_bytes(),
					vec![0x1]
				),
				Error::<Test>::DisabledTopic
			);
		});
}

#[test]
fn make_simple_claim() {
	ExtBuilder::default()
		.issuer(ALICE)
		.topic(b"access", true)
		.build()
		.execute_with(|| {
			let topic = String::from("access").into_bytes();
			assert_ok!(ConsortiumPermission::make_claim(
				Origin::signed(ALICE),
				CHARLIE,
				topic.clone(),
				vec![0x1]
			));
			assert_eq!(ConsortiumPermission::claim((CHARLIE, &topic)), (ALICE, vec![0x1]));
			assert_eq!(ConsortiumPermission::issuer_claims(ALICE), [(CHARLIE, topic.clone())]);
			assert_eq!(ConsortiumPermission::holder_claims(CHARLIE), [topic.clone()]);
			let events = System::events();
			assert_eq!(
				events[0].event,
				TestEvent::consortium_permission(RawEvent::ClaimMade(ALICE, CHARLIE, topic, vec![0x1]))
			);
		});
}

#[test]
fn reissue_simple_claim() {
	ExtBuilder::default()
		.issuer(ALICE)
		.issuer(BOB)
		.topic(b"access", true)
		.build()
		.execute_with(|| {
			let topic = String::from("access").into_bytes();
			assert_ok!(ConsortiumPermission::make_claim(
				Origin::signed(ALICE),
				CHARLIE,
				topic.clone(),
				vec![0x1]
			));
			// Reissue
			assert_ok!(ConsortiumPermission::make_claim(
				Origin::signed(BOB),
				CHARLIE,
				topic.clone(),
				vec![0x0]
			));
			assert_eq!(ConsortiumPermission::claim((CHARLIE, &topic)), (BOB, vec![0x0]));
			assert_eq!(ConsortiumPermission::issuer_claims(ALICE), []); // Claim moved off Alice
			assert_eq!(ConsortiumPermission::issuer_claims(BOB), [(CHARLIE, topic.clone())]); // and onto Bob
			assert_eq!(ConsortiumPermission::holder_claims(CHARLIE), [topic.clone()]);
			let events = System::events();
			assert_eq!(
				events[1].event,
				TestEvent::consortium_permission(RawEvent::ClaimMade(BOB, CHARLIE, topic.clone(), vec![0x0]))
			);
		});
}

#[test]
fn claim_value_is_too_damn_long() {
	ExtBuilder::default()
		.issuer(ALICE)
		.topic(b"access", true)
		.build()
		.execute_with(|| {
			assert_noop!(
				ConsortiumPermission::make_claim(
					Origin::signed(ALICE),
					CHARLIE,
					String::from("access").into_bytes(),
					vec![0x1; <mock::Test as Trait>::MaximumValueSize::get() + 1]
				),
				Error::<Test>::ValueExceedsAllowableSize
			);
		});
}

#[test]
fn claim_value_is_just_right() {
	ExtBuilder::default()
		.issuer(ALICE)
		.topic(b"access", true)
		.build()
		.execute_with(|| {
			assert_ok!(ConsortiumPermission::make_claim(
				Origin::signed(ALICE),
				CHARLIE,
				String::from("access").into_bytes(),
				vec![0x1; <mock::Test as Trait>::MaximumValueSize::get()]
			));
		});
}

#[test]
fn claim_revocation_fails_if_it_doesnt_exist() {
	ExtBuilder::default()
		.issuer(ALICE)
		.topic(b"access", true)
		.build()
		.execute_with(|| {
			assert_noop!(
				ConsortiumPermission::revoke_claim(Origin::signed(ALICE), CHARLIE, String::from("access").into_bytes()),
				Error::<Test>::CannotRemoveNonExistentClaim
			);
		});
}

#[test]
fn claim_revocation_fails_with_non_issuer() {
	ExtBuilder::default()
		.issuer(ALICE)
		.topic(b"access", true)
		.build()
		.execute_with(|| {
			assert_ok!(ConsortiumPermission::make_claim(
				Origin::signed(ALICE),
				CHARLIE,
				String::from("access").into_bytes(),
				vec![0x1]
			));
			assert_noop!(
				ConsortiumPermission::revoke_claim(Origin::signed(BOB), CHARLIE, String::from("access").into_bytes()),
				Error::<Test>::NotAnIssuer
			);
		});
}

#[test]
fn revoke_simple_claim() {
	ExtBuilder::default()
		.issuer(ALICE)
		.topic(b"access", true)
		.build()
		.execute_with(|| {
			let topic = String::from("access").into_bytes();
			assert_ok!(ConsortiumPermission::make_claim(
				Origin::signed(ALICE),
				CHARLIE,
				topic.clone(),
				vec![0x1]
			));
			assert_ok!(ConsortiumPermission::revoke_claim(
				Origin::signed(ALICE),
				CHARLIE,
				topic.clone()
			));
			assert_eq!(ConsortiumPermission::claim((CHARLIE, &topic)), (0, vec![]));
			assert_eq!(ConsortiumPermission::issuer_claims(ALICE), vec![]);
			assert_eq!(ConsortiumPermission::holder_claims(CHARLIE), Vec::<Topic>::default());
			let events = System::events();
			assert_eq!(
				events[1].event,
				TestEvent::consortium_permission(RawEvent::ClaimRevoked(ALICE, CHARLIE, topic.clone()))
			);
		});
}

#[test]
fn revoke_someone_elses_claim() {
	ExtBuilder::default()
		.issuer(ALICE)
		.issuer(BOB)
		.topic(b"access", true)
		.build()
		.execute_with(|| {
			let topic = String::from("access").into_bytes();
			assert_ok!(ConsortiumPermission::make_claim(
				Origin::signed(ALICE),
				CHARLIE,
				topic.clone(),
				vec![0x1]
			));
			assert_ok!(ConsortiumPermission::revoke_claim(Origin::signed(BOB), CHARLIE, topic.clone()));
			assert_eq!(ConsortiumPermission::claim((CHARLIE, &topic)), (0, vec![]));
			assert_eq!(ConsortiumPermission::issuer_claims(ALICE), vec![]);
			assert_eq!(ConsortiumPermission::issuer_claims(BOB), vec![]);
			assert_eq!(ConsortiumPermission::holder_claims(CHARLIE), Vec::<Topic>::default());
			let events = System::events();
			assert_eq!(
				events[1].event,
				TestEvent::consortium_permission(RawEvent::ClaimRevoked(BOB, CHARLIE, topic.clone()))
			);
		});
}

#[test]
fn sudo_claim_revocation_fails_without_sudo() {
	ExtBuilder::default()
		.issuer(ALICE)
		.topic(b"access", true)
		.build()
		.execute_with(|| {
			assert_ok!(ConsortiumPermission::make_claim(
				Origin::signed(ALICE),
				CHARLIE,
				String::from("access").into_bytes(),
				vec![0x1]
			));
			assert_noop!(
				ConsortiumPermission::sudo_revoke_claim(Origin::signed(ALICE), CHARLIE, String::from("access").into_bytes()),
				BadOrigin
			);
		});
}

#[test]
fn sudo_claim_revocation_fails_if_it_doesnt_exist() {
	ExtBuilder::default()
		.issuer(ALICE)
		.topic(b"access", true)
		.build()
		.execute_with(|| {
			assert_noop!(
				ConsortiumPermission::sudo_revoke_claim(Origin::ROOT, CHARLIE, String::from("access").into_bytes()),
				Error::<Test>::CannotRemoveNonExistentClaim
			);
		});
}

#[test]
fn sudo_revoke_a_claim() {
	ExtBuilder::default()
		.issuer(ALICE)
		.topic(b"access", true)
		.build()
		.execute_with(|| {
			let topic = String::from("access").into_bytes();
			assert_ok!(ConsortiumPermission::make_claim(
				Origin::signed(ALICE),
				CHARLIE,
				topic.clone(),
				vec![0x1]
			));
			assert_ok!(ConsortiumPermission::sudo_revoke_claim(Origin::ROOT, CHARLIE, topic.clone()));
			assert_eq!(ConsortiumPermission::claim((CHARLIE, &topic)), (0, vec![]));
			assert_eq!(ConsortiumPermission::issuer_claims(ALICE), vec![]);
			assert_eq!(ConsortiumPermission::holder_claims(CHARLIE), Vec::<Topic>::default());
			let events = System::events();
			assert_eq!(
				events[1].event,
				TestEvent::consortium_permission(RawEvent::ClaimRevokedBySudo(CHARLIE, topic.clone()))
			);
		});
}

#[test]
fn initialise_topics_works() {
	ExtBuilder::default()
		.genesis_topic(ACCESS_TOPIC)
		.build()
		.execute_with(|| {
			assert_eq!(Topics::get(), vec![ACCESS_TOPIC.to_vec()]);
			assert_eq!(TopicEnabled::get(ACCESS_TOPIC.to_vec()), true);
		})
}

#[test]
fn add_topic_requires_root() {
	ExtBuilder::default().build().execute_with(|| {
		let topic = b"test".to_vec();
		assert_noop!(ConsortiumPermission::add_topic(Origin::signed(ALICE), topic.clone()), BadOrigin);
		assert_ok!(ConsortiumPermission::add_topic(Origin::ROOT, topic));
	});
}

#[test]
fn add_topic_rejects_adding_existing_topic() {
	ExtBuilder::default().build().execute_with(|| {
		let topic = b"test".to_vec();
		assert_ok!(ConsortiumPermission::add_topic(Origin::ROOT, topic.clone()));
		assert_noop!(ConsortiumPermission::add_topic(Origin::ROOT, topic), Error::<Test>::TopicExists);
	});
}

#[test]
fn add_topic_rejects_long_topic_name() {
	ExtBuilder::default().build().execute_with(|| {
		let topic = vec![0_u8; MaximumTopicSize::get() + 1];
		assert_noop!(
			ConsortiumPermission::add_topic(Origin::ROOT, topic),
			Error::<Test>::TopicExceedsAllowableSize,
		);
	});
}

#[test]
fn add_topic_populates_storage() {
	ExtBuilder::default().build().execute_with(|| {
		let test_topic = b"test".to_vec();
		assert_ok!(ConsortiumPermission::add_topic(Origin::ROOT, test_topic.clone()));
		assert_eq!(ConsortiumPermission::topics(), vec![test_topic.clone()]);
		assert_eq!(ConsortiumPermission::topic_enabled(test_topic), false);
	});
}

#[test]
fn add_topic_emits_events() {
	ExtBuilder::default().build().execute_with(|| {
		let topic = b"test".to_vec();
		assert_ok!(ConsortiumPermission::add_topic(Origin::ROOT, topic.clone()));
		let events = System::events();
		assert_eq!(events[0].event, TestEvent::consortium_permission(RawEvent::TopicAdded(topic)));
	});
}

#[test]
fn enable_topic_requires_root() {
	let topic = b"test";
	ExtBuilder::default().topic(topic, false).build().execute_with(|| {
		let topic = topic.to_vec();
		assert_noop!(
			ConsortiumPermission::enable_topic(Origin::signed(ALICE), topic.clone()),
			BadOrigin
		);
		assert_ok!(ConsortiumPermission::enable_topic(Origin::ROOT, topic));
	});
}

#[test]
fn enable_topic_rejects_unknown_topic() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			ConsortiumPermission::enable_topic(Origin::ROOT, b"test".to_vec()),
			Error::<Test>::InvalidTopic
		);
	});
}

#[test]
fn enable_topic_updates_storage() {
	let topic = b"test";
	ExtBuilder::default().topic(topic, false).build().execute_with(|| {
		let topic = topic.to_vec();
		assert_ok!(ConsortiumPermission::enable_topic(Origin::ROOT, topic.clone()));
		assert_eq!(ConsortiumPermission::topic_enabled(topic), true);
	});
}

#[test]
fn enable_topic_emits_events() {
	let topic = b"test";
	ExtBuilder::default().topic(topic, false).build().execute_with(|| {
		let topic = topic.to_vec();
		assert_ok!(ConsortiumPermission::enable_topic(Origin::ROOT, topic.clone()));
		let events = System::events();
		assert_eq!(events[0].event, TestEvent::consortium_permission(RawEvent::TopicEnabled(topic)));
	});
}

#[test]
fn disable_topic_requires_root() {
	let topic = b"test";
	ExtBuilder::default().topic(topic, true).build().execute_with(|| {
		let topic = topic.to_vec();
		assert_noop!(
			ConsortiumPermission::disable_topic(Origin::signed(ALICE), topic.clone()),
			BadOrigin
		);
		assert_ok!(ConsortiumPermission::disable_topic(Origin::ROOT, topic));
	});
}

#[test]
fn disable_topic_rejects_unknown_topic() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			ConsortiumPermission::disable_topic(Origin::ROOT, b"test".to_vec()),
			Error::<Test>::InvalidTopic
		);
	});
}

#[test]
fn disable_topic_updates_storage() {
	let topic = b"test";
	ExtBuilder::default().topic(topic, true).build().execute_with(|| {
		let topic = topic.to_vec();
		assert_ok!(ConsortiumPermission::disable_topic(Origin::ROOT, topic.clone()));
		assert_eq!(ConsortiumPermission::topic_enabled(topic), false);
	});
}

#[test]
fn disable_topic_emits_events() {
	let topic = b"test";
	ExtBuilder::default().topic(topic, true).build().execute_with(|| {
		let topic = topic.to_vec();
		assert_ok!(ConsortiumPermission::disable_topic(Origin::ROOT, topic.clone()));
		let events = System::events();
		assert_eq!(
			events[0].event,
			TestEvent::consortium_permission(RawEvent::TopicDisabled(topic))
		);
	});
}
