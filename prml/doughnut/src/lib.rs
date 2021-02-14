// Copyright 2019-2020 Plug New Zealand Limited
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


//! A collection of doughnut traits and structs which provide doughnut integration for a plug runtime.
//! This includes validation and signature verification and type conversions.
#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode};
use sp_std::{self, prelude::Vec, any::Any};
use sp_runtime::{
	Doughnut,
	traits::{PlugDoughnutApi, Member},
};
use frame_support::{
	additional_traits::DelegatedDispatchVerifier,
	traits::Time,
	Parameter,
};

mod constants;
pub use constants::error_code;
mod impls;

// TODO: This should eventually become a super trait for `system::Config` so that all doughnut functionality may be moved here
/// A runtime which supports doughnut verification and validation
pub trait DoughnutRuntime {
	type AccountId: Member + Parameter;
	type Call;
	type Doughnut: Member + Parameter + PlugDoughnutApi;
	type TimestampProvider: Time;
}

/// A doughnut wrapped for compatibility with the extrinsic transport layer and the plug runtime types.
/// It can be passed to the runtime as a `SignedExtension` in an extrinsic.
#[derive(Encode, Decode, Clone, Eq, PartialEq)]
pub struct PlugDoughnut<Runtime: DoughnutRuntime>(Doughnut, sp_std::marker::PhantomData<Runtime>);

impl<Runtime> sp_std::fmt::Debug for PlugDoughnut<Runtime>
where
	Runtime: DoughnutRuntime + Send + Sync,
{
	fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		self.0.encode().fmt(f)
	}
}

impl<Runtime> PlugDoughnut<Runtime>
where
	Runtime: DoughnutRuntime,
{
	/// Create a new PlugDoughnut
	pub fn new(doughnut: Doughnut) -> Self {
		Self(doughnut, sp_std::marker::PhantomData)
	}
}

/// It verifies that a doughnut allows execution of a module+method combination
pub struct PlugDoughnutDispatcher<Runtime: DoughnutRuntime>(sp_std::marker::PhantomData<Runtime>);

impl<Runtime: DoughnutRuntime> DelegatedDispatchVerifier for PlugDoughnutDispatcher<Runtime> {
	type Doughnut = Runtime::Doughnut;
	type AccountId = Runtime::AccountId;
	const DOMAIN: &'static str = "plug";
	/// Verify a Doughnut proof authorizes method dispatch given some input parameters
	fn verify_dispatch(
		_doughnut: &Runtime::Doughnut,
		_module: &str,
		_method: &str,
		_args: Vec<(&str, &dyn Any)>,
	) -> Result<(), &'static str> {
		Err("Doughnut dispatch verification is not implemented for this domain")
	}
}
