// This file is part of Substrate.

// Copyright (C) 2019-2020 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Substrate core types and inherents for timestamps.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::Encode;
#[cfg(feature = "std")]
use codec::Decode;
#[cfg(feature = "std")]
use sp_inherents::ProvideInherentData;
use sp_inherents::{InherentIdentifier, IsFatalError, InherentData};

#[cfg(feature = "std")]
use log::info;

use sp_runtime::RuntimeString;

/// The identifier for the `timestamp` inherent.
pub const INHERENT_IDENTIFIER: InherentIdentifier = *b"timstap0";
/// The type of the inherent.
pub type InherentType = u64;

/// Errors that can occur while checking the timestamp inherent.
#[derive(Encode, sp_runtime::RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Decode))]
pub enum InherentError {
	/// The timestamp is valid in the future.
	/// This is a non-fatal-error and will not stop checking the inherents.
	ValidAtTimestamp(InherentType),
	/// Some other error.
	Other(RuntimeString),
}

impl IsFatalError for InherentError {
	fn is_fatal_error(&self) -> bool {
		match self {
			InherentError::ValidAtTimestamp(_) => false,
			InherentError::Other(_) => true,
		}
	}
}

impl InherentError {
	/// Try to create an instance ouf of the given identifier and data.
	#[cfg(feature = "std")]
	pub fn try_from(id: &InherentIdentifier, data: &[u8]) -> Option<Self> {
		if id == &INHERENT_IDENTIFIER {
			<InherentError as codec::Decode>::decode(&mut &data[..]).ok()
		} else {
			None
		}
	}
}

/// Auxiliary trait to extract timestamp inherent data.
pub trait TimestampInherentData {
	/// Get timestamp inherent data.
	fn timestamp_inherent_data(&self) -> Result<InherentType, sp_inherents::Error>;
}

impl TimestampInherentData for InherentData {
	fn timestamp_inherent_data(&self) -> Result<InherentType, sp_inherents::Error> {
		self.get_data(&INHERENT_IDENTIFIER)
			.and_then(|r| r.ok_or_else(|| "Timestamp inherent data not found".into()))
	}
}

/// Provide duration since unix epoch in millisecond for timestamp inherent.
#[cfg(feature = "std")]
pub struct InherentDataProvider;

#[cfg(feature = "std")]
impl ProvideInherentData for InherentDataProvider {
	fn inherent_identifier(&self) -> &'static InherentIdentifier {
		&INHERENT_IDENTIFIER
	}

	fn provide_inherent_data(
		&self,
		inherent_data: &mut InherentData,
	) -> Result<(), sp_inherents::Error> {
		use wasm_timer::SystemTime;

		let now = SystemTime::now();
		let timestamp = now.duration_since(SystemTime::UNIX_EPOCH)
			.expect("Current time is always after unix epoch; qed")
			.as_millis() as u64;

		// NIKAU HOTFIX: mutate timestamp to make it revert back in time and have slots
		// happen at 5x their speed from then until we have caught up with the present time.
		// ref: https://github.com/paritytech/substrate/pull/4543/files

		// validators will start authoring at warp speed after this timestamp
		// (it's set to some future time when this patch will be live on validators)
		// Wed Oct 27 2021 17:13:38 GMT+1300 (New Zealand Daylight Time)
		const REVIVE_TIMESTAMP: u64 = 1635417801 * 1000;
		// the block timestamp we'll start again from
		// Block #1,805,572
		const FORK_TIMESTAMP: u64 = 1634594250000;
		const WARP_FACTOR: u64 = 5;

		// time goes forward this diff gets bigger
		let time_since_revival = timestamp.saturating_sub(REVIVE_TIMESTAMP);
		// bigger diff = bigger warp timestamp
		// once warp has caught up we can go back to ordinary timestamp
		let warped_timestamp = FORK_TIMESTAMP + (WARP_FACTOR * time_since_revival);

		info!(target: "babe", "timestamp warped: {:?} to {:?} ({:?} since revival)", timestamp, warped_timestamp, time_since_revival);

		// we want to ensure our timestamp is such that slots run monotonically with blocks
		// at 1/5th of the slot_duration from this slot onwards until we catch up to the
		// wall-clock time.
		let maybe_warped_timestamp = timestamp.min(warped_timestamp);
		inherent_data.put_data(INHERENT_IDENTIFIER, &maybe_warped_timestamp)
	}

	fn error_to_string(&self, error: &[u8]) -> Option<String> {
		InherentError::try_from(&INHERENT_IDENTIFIER, error).map(|e| format!("{:?}", e))
	}
}


/// A trait which is called when the timestamp is set.
#[impl_trait_for_tuples::impl_for_tuples(30)]
pub trait OnTimestampSet<Moment> {
	fn on_timestamp_set(moment: Moment);
}
