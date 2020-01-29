// Copyright 2019-2020 Plug New Zealand Limited
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

//! A collection of doughnut traits and srtucts which provide doughnut integartion for a plug runtime.
//! This includes validation and signature verification and type conversions.
#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode};
use sp_std::{self};
use sp_runtime::{Doughnut};
use sp_runtime::traits::{PlugDoughnutApi, Member};
use frame_support::Parameter;
use frame_support::additional_traits::DelegatedDispatchVerifier;
use frame_support::traits::Time;

mod impls;

// TODO: This should eventually become a super trait for `system::Trait` so that all doughnut functionality may be moved here
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
	) -> Result<(), &'static str> {
		Err("Doughnut dispatch verification is not implemented for this domain")
	}
}
