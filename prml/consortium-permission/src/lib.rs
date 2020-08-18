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

//! # Consortium Permission module.
//!
//! This module is intended to store permissions that can be used in a Consortium Chain.
//! Root can add and remove accounts as "issuers".
//! "issuers" accounts can grant and revoke different permissions topics for other chain users.
//! Runtime modules can use this module to look up granted permissions when needed.
//!
//! ## Dispatchable methods
//!
//! There are a number of dispatchable methods. Some of which require Root privilege.
//!
//! ```ignore
//! /// Manage issuers. Requires Root.
//! pub fn add_issuer(origin, who: T::AccountId) { ... }
//! pub fn remove_issuer(origin, who: T::AccountId) { ... }
//!
//! /// Manage permission Topics. Requires Root.
//! pub fn add_topic(origin, topic: Topic) { ... }
//! pub fn enable_topic(origin, topic: Topic) { ... }
//! pub fn disable_topic(origin, topic: Topic) { ... }
//!
//! /// Manage permission Claims. Requires caller to be an "issuer".
//! pub fn make_claim(origin, holder: T::AccountId, topic: Topic, value: Value) { ... }
//! pub fn revoke_claim(origin, holder: T::AccountId, topic: Topic) { ... }
//!
//! /// Revokes a preexisting claim about a holder. Requires Root.
//! pub fn sudo_revoke_claim(origin, holder: T::AccountId, topic: Topic) { ... }
//! ```
//!
//! ## Interfacing with other modules
//!
//! Interaction with the consortium-permission module can be done via traits implementation
//! in the Runtime.
//! 'IssuerPermissions' can be used as a reference example.
//!

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

use frame_support::{decl_error, decl_event, decl_module, decl_storage, ensure, traits::Get};
use frame_system::{ensure_root, ensure_signed};
use sp_runtime::DispatchResult;
use sp_std::prelude::*;

/// Type used for topic names.
pub type Topic = Vec<u8>;
/// Type used for values of corresponding topics.
pub type Value = Vec<u8>;

/// Allows runtime implmentation of issuer configuration.
pub trait IssuerPermissions {
	type AccountId;

	/// Grants permissions for an account to be an issuer.
	fn grant_issuer_permissions(issuer: &Self::AccountId);

	/// Revokes permissions from an account to be a non-issuer.
	fn revoke_issuer_permissions(issuer: &Self::AccountId);
}

/// The module's config trait.
pub trait Trait: frame_system::Trait {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
	/// The maximum number of bytes allowed for a topic name.
	type MaximumTopicSize: Get<usize>;
	/// The maximum number of bytes allowed for a value.
	type MaximumValueSize: Get<usize>;
	/// Provides an interface for setting issuer permissions
	type IssuerPermissions: IssuerPermissions<AccountId = <Self as frame_system::Trait>::AccountId>;
}

decl_storage! {
	trait Store for Module<T: Trait> as ConsortiumPermission {
		/// List of whitelisted accounts with permission to issue an attestation.
		Issuers get(fn issuers): Vec<T::AccountId>;
		/// List of all topics.
		Topics get(fn topics): Vec<Topic>;
		/// Map of topics to enabled / disabled status.
		TopicEnabled get(fn topic_enabled): map hasher(twox_64_concat) Topic => bool;
		/// Map of `holder, topic` to a `claim` containing `issuer, value`.
		Claim get(fn claim): map hasher(twox_64_concat) (T::AccountId, Topic) => (T::AccountId, Value);
		/// Map of issuer to all holder/topic pairs they have made claims on.
		IssuerClaims get(fn issuer_claims): map hasher(twox_64_concat) T::AccountId => Vec<(T::AccountId, Topic)>;
		/// Map of holder to all topics that have been claimed about them.
		HolderClaims get(fn holder_claims): map hasher(twox_64_concat) T::AccountId => Vec<Topic>;
	}
	add_extra_genesis {
		config(issuers): Vec<T::AccountId>;
		config(topics): Vec<Vec<u8>>;
		build(|config| {
			Module::<T>::initialise_topics(&config.topics);
			Module::<T>::initialise_issuers(&config.issuers);
		})
	}
}

decl_event! {
	pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId {
		/// A new issuer is added.
		IssuerAdded(AccountId),
		/// An existing issuer is removed.
		IssuerRemoved(AccountId),
		/// A claim has been made.
		ClaimMade(AccountId, AccountId, Topic, Value),
		/// A claim has been revoked.
		ClaimRevoked(AccountId, AccountId, Topic),
		/// A claim has been revoked by sudo.
		ClaimRevokedBySudo(AccountId, Topic),
		/// A new topic is added.
		TopicAdded(Topic),
		/// An existing topic is enabled.
		TopicEnabled(Topic),
		/// An existing topic is disabled.
		TopicDisabled(Topic),
	}
}

decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Must be an issuer to make a claim.
		NotAnIssuer,
		/// Issuer already exists in storage.
		IssuerExists,
		/// Topic does not exist.
		InvalidTopic,
		/// Topic is disabled.
		DisabledTopic,
		/// Topic already exists in storage.
		TopicExists,
		/// Topic name has too many bytes.
		TopicExceedsAllowableSize,
		/// Value has too many bytes.
		ValueExceedsAllowableSize,
		/// Attempt to remove claim that doesn't exist.
		CannotRemoveNonExistentClaim,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin, system = frame_system {
		// Initialises errors.
		type Error = Error<T>;

		// Initialises events.
		fn deposit_event() = default;

		/// Adds a new issuer as root.
		pub fn add_issuer(origin, who: T::AccountId) {
			ensure_root(origin)?;
			let mut issuers = Self::issuers();
			ensure!(!issuers.contains(&who), Error::<T>::IssuerExists);
			issuers.push(who.clone());
			Issuers::<T>::put(issuers);

			T::IssuerPermissions::grant_issuer_permissions(&who);

			Self::deposit_event(RawEvent::IssuerAdded(who));
		}

		/// Removes an existing issuer as root.
		pub fn remove_issuer(origin, who: T::AccountId) {
			ensure_root(origin)?;
			let mut issuers = Self::issuers();
			ensure!(issuers.contains(&who), Error::<T>::NotAnIssuer);
			issuers.retain(|x| *x != who);
			Issuers::<T>::put(issuers);

			T::IssuerPermissions::revoke_issuer_permissions(&who);

			Self::deposit_event(RawEvent::IssuerRemoved(who));
		}

		/// Adds a new topic as root.
		pub fn add_topic(origin, topic: Topic) {
			ensure_root(origin)?;
			Self::insert_topic(&topic)?;
			Self::deposit_event(RawEvent::TopicAdded(topic));
		}

		/// Enable an existing topic as root.
		pub fn enable_topic(origin, topic: Topic) {
			ensure_root(origin)?;
			Self::update_topic(&topic, true)?;
			Self::deposit_event(RawEvent::TopicEnabled(topic));
		}

		/// Disable an existing topic as root.
		pub fn disable_topic(origin, topic: Topic) {
			ensure_root(origin)?;
			Self::update_topic(&topic, false)?;
			Self::deposit_event(RawEvent::TopicDisabled(topic));
		}

		/// Makes a claim on a topic about a holder.
		pub fn make_claim(origin, holder: T::AccountId, topic: Topic, value: Value) {
			let issuer = ensure_signed(origin)?;
			ensure!(Self::issuers().contains(&issuer), Error::<T>::NotAnIssuer);
			ensure!(Self::topics().contains(&topic), Error::<T>::InvalidTopic);
			ensure!(Self::topic_enabled(&topic), Error::<T>::DisabledTopic);
			ensure!(value.len() <= T::MaximumValueSize::get(), Error::<T>::ValueExceedsAllowableSize);

			Self::do_make_claim(&issuer, &holder, &topic, &value);

			Self::deposit_event(RawEvent::ClaimMade(issuer, holder, topic, value));
		}

		/// Revokes a preexisting claim about a holder.
		pub fn revoke_claim(origin, holder: T::AccountId, topic: Topic) {
			let issuer = ensure_signed(origin)?;
			ensure!(Self::issuers().contains(&issuer), Error::<T>::NotAnIssuer);
			ensure!(Self::holder_claims(&holder).contains(&topic), Error::<T>::CannotRemoveNonExistentClaim);

			Self::do_revoke_claim(holder.clone(), topic.clone());

			Self::deposit_event(RawEvent::ClaimRevoked(issuer, holder, topic));
		}

		/// Revokes a preexisting claim about a holder - root only.
		pub fn sudo_revoke_claim(origin, holder: T::AccountId, topic: Topic) {
			ensure_root(origin)?;
			ensure!(Self::holder_claims(&holder).contains(&topic), Error::<T>::CannotRemoveNonExistentClaim);

			Self::do_revoke_claim(holder.clone(), topic.clone());

			Self::deposit_event(RawEvent::ClaimRevokedBySudo(holder, topic));
		}
	}
}

impl<T: Trait> Module<T> {
	/// Initialises whitelisted issuers configured in genesis.
	fn initialise_issuers(issuers: &[T::AccountId]) {
		Issuers::<T>::put(issuers.clone());
		for issuer in issuers {
			T::IssuerPermissions::grant_issuer_permissions(&issuer);
		}
	}

	/// Performs all storage changes to make a claim by an issuer on a topic about a holder.
	pub fn do_make_claim(issuer: &T::AccountId, holder: &T::AccountId, topic: &Topic, value: &Value) {
		let mut holder_claims = Self::holder_claims(&holder);
		if !holder_claims.contains(&topic) {
			holder_claims.push(topic.clone());
			HolderClaims::<T>::insert(&holder, holder_claims);
		} else {
			// Remove from previous issuer's claim list
			let (old_issuer, _) = Self::claim((&holder, &topic));
			Self::remove_issuer_claim(old_issuer, holder.clone(), topic.clone());
		}

		let mut issuer_claims = Self::issuer_claims(&issuer);
		if !issuer_claims.contains(&(holder.clone(), topic.clone())) {
			issuer_claims.push((holder.clone(), topic.clone()));
			IssuerClaims::<T>::insert(&issuer, issuer_claims);
		}

		Claim::<T>::insert((holder, topic), (issuer, value));
	}

	/// Performs all storage changes to revoke a claim on a topic about a holder.
	pub fn do_revoke_claim(holder: T::AccountId, topic: Topic) {
		// Remove claim from issuer list
		let (old_issuer, _) = Self::claim((&holder, &topic));
		Self::remove_issuer_claim(old_issuer, holder.clone(), topic.clone());

		// Remove claim from holder list
		let mut holder_claims = Self::holder_claims(&holder);
		holder_claims.retain(|x| *x != topic.clone());
		HolderClaims::<T>::insert(&holder, holder_claims);

		Claim::<T>::remove((holder, topic));
	}

	/// Removes a claim from a specific issuer's claim list.
	fn remove_issuer_claim(issuer: T::AccountId, holder: T::AccountId, topic: Topic) {
		let mut issuer_claims = Self::issuer_claims(&issuer);
		issuer_claims.retain(|x| *x != (holder.clone(), topic.clone()));
		IssuerClaims::<T>::insert(&issuer, issuer_claims);
	}

	/// Initialises reserved topics at genesis.
	fn initialise_topics(topics: &Vec<Topic>) {
		Topics::put(topics.clone());
		for topic in topics {
			TopicEnabled::insert(topic, true);
		}
	}

	/// Performs all storage changes to add a topic.
	fn insert_topic(topic: &[u8]) -> DispatchResult {
		ensure!(
			topic.len() <= T::MaximumTopicSize::get(),
			Error::<T>::TopicExceedsAllowableSize
		);
		let mut topics = Self::topics();
		ensure!(!topics.contains(&topic.to_vec()), Error::<T>::TopicExists);
		topics.push(topic.to_vec());
		Topics::put(topics);
		TopicEnabled::insert(topic, false);
		Ok(())
	}

	/// Performs all storage changes to revoke a claim on a topic about a holder.
	fn update_topic(topic: &Topic, enabled: bool) -> DispatchResult {
		ensure!(TopicEnabled::contains_key(topic.clone()), Error::<T>::InvalidTopic);
		TopicEnabled::mutate(topic, |status| *status = enabled);
		Ok(())
	}
}
