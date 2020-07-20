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

//! # Validator Manager module.
//!
//! This module provides configurable proof-of-authority through adding and
//! removing of validators, controlled by root. It is intended to be used in
//! conjunction with AURA and GRANDPA.
//!
//! ## Dispatchable methods
//!
//! There are two dispatchable methods, both of which require root previlage.
//!
//! ```no_run
//! pub fn add(origin, validator: T::ValidatorId) { ... }
//! pub fn remove(origin, validator: T::ValidatorId) { ... }
//! ```
//!
//! *Note* session keys of new validators must be set prior to calling `add()`.
//!
//! ## Dependency
//!
//! The module implements `pallet_session::SessionManager` trait to put a set
//! of validators in a queue to start participating in block production from
//! the next session.

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

use frame_support::{decl_error, decl_event, decl_module, decl_storage, ensure, traits::Get};
use frame_system::ensure_root;
use pallet_session::Module as Session;
use sp_runtime::traits::Zero;
use sp_staking::SessionIndex;
use sp_std::prelude::*;

/// The module's config trait.
pub trait Trait: frame_system::Trait + pallet_session::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    /// The minimum number of validators persisted in storage to ensure block production continues.
    type MinimumValidatorCount: Get<u32>;
}

decl_storage! {
    trait Store for Module<T: Trait> as ValidatorManager {
        /// Current validators set.
        Validators get(fn validators) config(): Vec<T::ValidatorId>;
    }
}

decl_event!(
    pub enum Event<T>
    where
        ValidatorId = <T as pallet_session::Trait>::ValidatorId,
    {
        /// New validator added.
        Added(ValidatorId),
        /// Validator removed.
        Removed(ValidatorId),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Number of validators in Validators should be at least MinimumValidatorCount.
        MinimumValidatorCount,
        /// Validator is already added.
        ValidatorAlreadyAdded,
        /// Validator to be removed is not found.
        ValidatorNotFound,
        /// Session keys are not set for a new validator.
        SessionKeysNotSet,
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin, system = frame_system {
        // Initialises errors.
        type Error = Error<T>;

        // Initialises events.
        fn deposit_event() = default;

        /// Adds a new validator using sudo privileges. New validator's
        /// session keys should be set in session module before calling this.
        pub fn add(origin, validator: T::ValidatorId) {
            ensure_root(origin)?;

            ensure!(Session::<T>::has_keys(&validator), Error::<T>::SessionKeysNotSet);

            let mut validators = Validators::<T>::get();
            ensure!(!validators.contains(&validator), Error::<T>::ValidatorAlreadyAdded);

            validators.push(validator.clone());
            Validators::<T>::put(validators);
            Self::deposit_event(RawEvent::Added(validator));
        }

        /// Removes a validator using sudo privileges.
        pub fn remove(origin, validator: T::ValidatorId) {
            ensure_root(origin)?;

            let mut validators = Validators::<T>::get();
            ensure!(validators.contains(&validator), Error::<T>::ValidatorNotFound);

            validators.retain(|x| *x != validator);
            ensure!(validators.len() >= T::MinimumValidatorCount::get() as usize, Error::<T>::MinimumValidatorCount);

            Validators::<T>::put(validators);
            Self::deposit_event(RawEvent::Removed(validator));
        }
    }
}

impl<T: Trait> Module<T> {
    /// Returns currently queued validators.
    fn queued_validators() -> Vec<T::ValidatorId> {
        Session::<T>::queued_keys()
            .iter()
            .map(|x| x.0.clone())
            .collect::<Vec<_>>()
    }

    /// Returns whether validator set is updated or not.
    fn is_updated(validators: &Vec<T::ValidatorId>) -> bool {
        let queued_validators = Self::queued_validators();
        !queued_validators.iter().eq(validators.iter())
    }
}

/// A trait for managing creation of new validator set.
impl<T: Trait> pallet_session::SessionManager<T::ValidatorId> for Module<T> {
    /// Provides a new set of validators to be queued for the next session.
    /// Guaranteed to be called before a new session starts.
    fn new_session(new_index: SessionIndex) -> Option<Vec<T::ValidatorId>> {
        if new_index.is_zero() {
            return None;
        }

        let validators = Validators::<T>::get();
        if Self::is_updated(&validators) {
            Some(validators)
        } else {
            None
        }
    }
    fn end_session(_end_index: SessionIndex) {}
    fn start_session(_start_index: SessionIndex) {}
}
