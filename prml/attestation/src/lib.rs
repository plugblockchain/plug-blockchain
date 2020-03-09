// Copyright 2019 Plug New Zealand Limited
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # Attestation Module
//!
//! The Attestation module provides functionality for entities to create attestation claims about one another.
//!
//! This module borrows heavily from ERC 780 https://github.com/ethereum/EIPs/issues/780
//!
//! ## Terminology
//!
//! Issuer: the entity creating the claim
//! Holder: the entity that the claim is about
//! Topic: the topic which the claim is about ie isOver18
//! Value: any value pertaining to the claim
//!
//! ## Usage
//!
//! Topic and Value are U256 integers. This means that Topic and Value can technically store any value that can be represented in 256 bits.
//!
//! The user of the module must convert whatever value that they would like to store into a value that can be stored as a U256.
//!
//! It is recommended that Topic be a string value converted to hex and stored on the blockchain as a U256.

#![cfg_attr(not(feature = "std"), no_std)]

mod mock;

use sp_core::uint::U256;
use frame_support::sp_std::prelude::*;
use frame_support::{decl_event, decl_module, decl_storage, dispatch::DispatchResult};
use frame_system::ensure_signed;

pub trait Trait: frame_system::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

type AttestationTopic = U256;
type AttestationValue = U256;

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin, system = frame_system {
		fn deposit_event() = default;

		/// Create a new claim
		pub fn set_claim(origin, holder: T::AccountId, topic: AttestationTopic, value: AttestationValue) -> DispatchResult {
			let issuer = ensure_signed(origin)?;

			Self::create_claim(holder, issuer, topic, value)?;
			Ok(())
		}

		/// Remove a claim, only the original issuer can remove a claim
		pub fn remove_claim(origin, holder: T::AccountId, topic: AttestationTopic) -> DispatchResult {
			let issuer = ensure_signed(origin)?;
			<Values<T>>::remove((holder.clone(), issuer.clone(), topic));

			<Topics<T>>::mutate((holder.clone(), issuer.clone()),|topics| topics.retain(|vec_topic| *vec_topic != topic));

			<Issuers<T>>::mutate(&holder, |issuers| {
					issuers.retain(|vec_issuer| {
						*vec_issuer != issuer.clone() ||
						Self::get_topics((holder.clone(), issuer.clone())).len() != 0
					})
			});


			Self::deposit_event(RawEvent::ClaimRemoved(holder, issuer, topic));

			Ok(())
		}
	}
}

decl_event!(
	pub enum Event<T> where <T as frame_system::Trait>::AccountId {
		ClaimSet(AccountId, AccountId, AttestationTopic, AttestationValue),
		ClaimRemoved(AccountId, AccountId, AttestationTopic),
	}
);

decl_storage! {
	trait Store for Module<T: Trait> as Attestation {
		/// The maps are layed out to support the nested structure shown below in JSON, will look to optimise later.
		///
		/// {
		///  holder: {
		///    issuer: {
		///      topic: <value>
		///    }
		///  }
		/// }
		///

		/// A map of HolderId => Vec<IssuerId>
		Issuers get(fn get_issuers):
			map hasher(blake2_256) T::AccountId => Vec<T::AccountId>;
		/// A map of (HolderId, IssuerId) => Vec<AttestationTopic>
		Topics get(fn get_topics):
			map hasher(blake2_256) (T::AccountId, T::AccountId) => Vec<AttestationTopic>;
		/// A map of (HolderId, IssuerId, AttestationTopic) => AttestationValue
		Values get(fn get_value):
			map hasher(blake2_256) (T::AccountId, T::AccountId, AttestationTopic) => AttestationValue;
	}
}

impl<T: Trait> Module<T> {
	fn create_claim(
		holder: T::AccountId,
		issuer: T::AccountId,
		topic: AttestationTopic,
		value: AttestationValue,
	) -> DispatchResult {
		<Issuers<T>>::mutate(&holder, |issuers| {
			if !issuers.contains(&issuer) {
				issuers.push(issuer.clone())
			}
		});

		<Topics<T>>::mutate((holder.clone(), issuer.clone()), |topics| {
			if !topics.contains(&topic) {
				topics.push(topic)
			}
		});

		<Values<T>>::insert((holder.clone(), issuer.clone(), topic), value);
		Self::deposit_event(RawEvent::ClaimSet(holder, issuer, topic, value));
		Ok(())
	}
}



#[cfg(test)]
mod tests {
	use super::*;
	use crate::mock::{ExtBuilder, Attestation, Origin, TestEvent, System};

	#[test]
	fn initialize_holder_has_no_claims() {
		let holder = 0xbaa;
		ExtBuilder::build().execute_with(|| {
			// Note: without any valid issuers, there is no valid input for
			// get_topics or get_values
			assert_eq!(Attestation::get_issuers(holder), []);
		})
	}

	#[test]
	fn adding_claim_to_storage() {
		let issuer = 0xf00;
		let holder = 0xbaa;
		let topic = AttestationTopic::from(0xf00d);
		let value = AttestationValue::from(0xb33f);
		ExtBuilder::build().execute_with(|| {
			let result = Attestation::set_claim(Origin::signed(issuer), holder, topic, value);

			assert_eq!(result, Ok(()));

			assert_eq!(Attestation::get_issuers(holder), [issuer]);
			assert_eq!(Attestation::get_topics((holder, issuer)), [topic]);
			assert_eq!(Attestation::get_value((holder, issuer, topic)), value);
		})
	}

	#[test]
	fn account_can_claim_on_itself() {
		let holder = 0x1d107;
		let topic = AttestationTopic::from(0xf001);
		let value = AttestationValue::from(0xb01);
		ExtBuilder::build().execute_with(|| {
			let result = Attestation::set_claim(Origin::signed(holder), holder, topic, value);

			assert_eq!(result, Ok(()));

			assert_eq!(Attestation::get_issuers(holder), [holder]);
			assert_eq!(Attestation::get_topics((holder, holder)), [topic]);
			assert_eq!(Attestation::get_value((holder, holder, topic)), value);
		})
	}

	#[test]
	fn adding_existing_claim_overwrites_claim() {
		let issuer = 0xf00;
		let holder = 0xbaa;
		let topic = AttestationTopic::from(0xf00d);
		let value_old = AttestationValue::from(0xb33f);
		let value_new = AttestationValue::from(0xcabba93);
		ExtBuilder::build().execute_with(|| {
			let result_old = Attestation::set_claim(Origin::signed(issuer), holder, topic, value_old);
			let result_new = Attestation::set_claim(Origin::signed(issuer), holder, topic, value_new);

			assert_eq!(result_old, Ok(()));
			assert_eq!(result_new, Ok(()));

			assert_eq!(Attestation::get_value((holder, issuer, topic)), value_new);
		})
	}

	#[test]
	fn adding_multiple_claims_from_same_issuer() {
		let issuer = 0xf00;
		let holder = 0xbaa;
		let topic_food = AttestationTopic::from(0xf00d);
		let value_food = AttestationValue::from(0xb33f);
		let topic_loot = AttestationTopic::from(0x1007);
		let value_loot = AttestationValue::from(0x901d);
		ExtBuilder::build().execute_with(|| {
			let result_food = Attestation::set_claim(Origin::signed(issuer), holder, topic_food, value_food);
			let result_loot = Attestation::set_claim(Origin::signed(issuer), holder, topic_loot, value_loot);

			assert_eq!(result_food, Ok(()));
			assert_eq!(result_loot, Ok(()));

			assert_eq!(Attestation::get_issuers(holder), [issuer]);
			assert_eq!(Attestation::get_topics((holder, issuer)), [topic_food, topic_loot]);
			assert_eq!(Attestation::get_value((holder, issuer, topic_food)), value_food);
			assert_eq!(Attestation::get_value((holder, issuer, topic_loot)), value_loot);
		})
	}

	#[test]
	fn adding_claims_from_different_issuers() {
		let issuer_foo = 0xf00;
		let issuer_boa = 0xb0a;
		let holder = 0xbaa;
		let topic_food = AttestationTopic::from(0xf00d);
		let value_food_foo = AttestationValue::from(0xb33f);
		let value_food_boa = AttestationValue::from(0x90a7);
		ExtBuilder::build().execute_with(|| {
			let result_foo = Attestation::set_claim(Origin::signed(issuer_foo), holder, topic_food, value_food_foo);
			let result_boa = Attestation::set_claim(Origin::signed(issuer_boa), holder, topic_food, value_food_boa);

			assert_eq!(result_foo, Ok(()));
			assert_eq!(result_boa, Ok(()));

			assert_eq!(Attestation::get_issuers(holder), [issuer_foo, issuer_boa]);
			assert_eq!(Attestation::get_topics((holder, issuer_foo)), [topic_food]);
			assert_eq!(Attestation::get_topics((holder, issuer_boa)), [topic_food]);
			assert_eq!(Attestation::get_value((holder, issuer_foo, topic_food)), value_food_foo);
			assert_eq!(Attestation::get_value((holder, issuer_boa, topic_food)), value_food_boa);
		})
	}

	#[test]
	fn remove_claim_from_storage() {
		let issuer = 0xf00;
		let holder = 0xbaa;
		let topic = AttestationTopic::from(0xf00d);
		let value = AttestationValue::from(0xb33f);
		let invalid_value = AttestationValue::zero();
		ExtBuilder::build().execute_with(|| {
			let result_add = Attestation::set_claim(Origin::signed(issuer), holder, topic, value);

			let result_remove = Attestation::remove_claim(Origin::signed(issuer), holder, topic);

			assert_eq!(result_add, Ok(()));
			assert_eq!(result_remove, Ok(()));

			assert_eq!(Attestation::get_issuers(holder), []);
			assert_eq!(Attestation::get_topics((holder, issuer)), []);
			assert_eq!(Attestation::get_value((holder, issuer, topic)), invalid_value);
		})
	}

	#[test]
	fn remove_claim_from_account_with_multiple_issuers() {
		let issuer_foo = 0xf00;
		let issuer_boa = 0xb0a;
		let holder = 0xbaa;
		let topic_food = AttestationTopic::from(0xf00d);
		let value_food_foo = AttestationValue::from(0xb33f);
		let value_food_boa = AttestationValue::from(0x90a7);
		let invalid_value = AttestationValue::zero();
		ExtBuilder::build().execute_with(|| {
			let result_foo = Attestation::set_claim(Origin::signed(issuer_foo), holder, topic_food, value_food_foo);
			let result_boa = Attestation::set_claim(Origin::signed(issuer_boa), holder, topic_food, value_food_boa);

			let result_remove = Attestation::remove_claim(Origin::signed(issuer_foo), holder, topic_food);

			assert_eq!(result_foo, Ok(()));
			assert_eq!(result_boa, Ok(()));
			assert_eq!(result_remove, Ok(()));

			assert_eq!(Attestation::get_issuers(holder), [issuer_boa]);
			assert_eq!(Attestation::get_topics((holder, issuer_foo)), []);
			assert_eq!(Attestation::get_topics((holder, issuer_boa)), [topic_food]);
			assert_eq!(Attestation::get_value((holder, issuer_foo, topic_food)), invalid_value);
			assert_eq!(Attestation::get_value((holder, issuer_boa, topic_food)), value_food_boa);
		})
	}

	#[test]
	fn remove_claim_from_account_with_multiple_claims_from_same_issuer() {
		let issuer = 0xf00;
		let holder = 0xbaa;
		let topic_food = AttestationTopic::from(0xf00d);
		let value_food = AttestationValue::from(0xb33f);
		let topic_loot = AttestationTopic::from(0x1007);
		let value_loot = AttestationValue::from(0x901d);
		let invalid_value = AttestationValue::zero();
		ExtBuilder::build().execute_with(|| {
			let result_food = Attestation::set_claim(Origin::signed(issuer), holder, topic_food, value_food);
			let result_loot = Attestation::set_claim(Origin::signed(issuer), holder, topic_loot, value_loot);

			let result_remove = Attestation::remove_claim(Origin::signed(issuer), holder, topic_food);

			assert_eq!(result_food, Ok(()));
			assert_eq!(result_loot, Ok(()));
			assert_eq!(result_remove, Ok(()));

			assert_eq!(Attestation::get_issuers(holder), [issuer]);
			assert_eq!(Attestation::get_topics((holder, issuer)), [topic_loot]);
			assert_eq!(Attestation::get_value((holder, issuer, topic_food)), invalid_value);
			assert_eq!(Attestation::get_value((holder, issuer, topic_loot)), value_loot);
		})
	}

	#[test]
	fn issuer_is_removed_if_there_are_no_claims_left() {
		let issuer = 0xf00;
		let holder = 0xbaa;
		let topic_food = AttestationTopic::from(0xf00d);
		let value_food = AttestationValue::from(0xb33f);
		let topic_loot = AttestationTopic::from(0x1007);
		let value_loot = AttestationValue::from(0x901d);
		let invalid_value = AttestationValue::zero();
		ExtBuilder::build().execute_with(|| {
			let result_food = Attestation::set_claim(Origin::signed(issuer), holder, topic_food, value_food);
			let result_loot = Attestation::set_claim(Origin::signed(issuer), holder, topic_loot, value_loot);

			let result_remove_food = Attestation::remove_claim(Origin::signed(issuer), holder, topic_food);
			let result_remove_loot = Attestation::remove_claim(Origin::signed(issuer), holder, topic_loot);

			assert_eq!(result_food, Ok(()));
			assert_eq!(result_loot, Ok(()));
			assert_eq!(result_remove_food, Ok(()));
			assert_eq!(result_remove_loot, Ok(()));

			assert_eq!(Attestation::get_issuers(holder), []);
			assert_eq!(Attestation::get_topics((holder, issuer)), []);
			assert_eq!(Attestation::get_value((holder, issuer, topic_food)), invalid_value);
			assert_eq!(Attestation::get_value((holder, issuer, topic_loot)), invalid_value);
		})
	}

	#[test]
	fn adding_claim_emits_event() {
		let issuer = 0xf00;
		let holder = 0xbaa;
		let topic = AttestationTopic::from(0xf00d);
		let value = AttestationValue::from(0xb33f);
		ExtBuilder::build().execute_with(|| {
			assert_eq!(Attestation::set_claim(Origin::signed(issuer), holder, topic, value), Ok(()));

			let expected_event = TestEvent::attestation(
				RawEvent::ClaimSet(holder, issuer, topic, value),
			);
			// Assert
			assert!(System::events().iter().any(|record| record.event == expected_event));
		})
	}

	#[test]
	fn removing_claim_emits_event() {
		let issuer = 0xf00;
		let holder = 0xbaa;
		let topic = AttestationTopic::from(0xf00d);
		let value = AttestationValue::from(0xb33f);
		ExtBuilder::build().execute_with(|| {
			assert_eq!(Attestation::set_claim(Origin::signed(issuer), holder, topic, value), Ok(()));
			assert_eq!(Attestation::remove_claim(Origin::signed(issuer), holder, topic), Ok(()));

			let expected_event = TestEvent::attestation(
				RawEvent::ClaimRemoved(holder, issuer, topic),
			);
			// Assert
			assert!(System::events().iter().any(|record| record.event == expected_event));
		})
	}

}
