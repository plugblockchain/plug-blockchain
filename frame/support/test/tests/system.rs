// Copyright 2019-2020 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

use frame_support::additional_traits::{DelegatedDispatchVerifier as DelegatedDispatchVerifierT, MaybeDoughnutRef};
use frame_support::codec::{Encode, Decode, EncodeLike};

pub trait Trait: 'static + Eq + Clone {
	type Origin: Into<Result<RawOrigin<Self::AccountId, Self::Doughnut>, Self::Origin>>
			+ From<RawOrigin<Self::AccountId, Self::Doughnut>> + MaybeDoughnutRef<Doughnut=()>;
	type BlockNumber: Decode + Encode + EncodeLike + Clone + Default;
	type Hash;
	type AccountId: Encode + EncodeLike + Decode;
	type Event: From<Event>;
	type ModuleToIndex: frame_support::traits::ModuleToIndex;
	type DelegatedDispatchVerifier: DelegatedDispatchVerifierT<Doughnut = ()>;
	type Doughnut;
}

frame_support::decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {}
}

impl<T: Trait> Module<T> {
	pub fn deposit_event(_event: impl Into<T::Event>) {}
}

frame_support::decl_event!(
	pub enum Event {
		ExtrinsicSuccess,
		ExtrinsicFailed,
	}
);

frame_support::decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Test error documentation
		TestError,
		/// Error documentation
		/// with multiple lines
		AnotherError
	}
}

/// Origin for the system module.
#[derive(PartialEq, Eq, Clone, sp_runtime::RuntimeDebug)]
pub enum RawOrigin<AccountId, Doughnut> {
	Root,
	Signed(AccountId),
	Delegated(AccountId, Doughnut),
	None,
}

impl<AccountId, Doughnut> From<(Option<AccountId>,Option<Doughnut>)> for RawOrigin<AccountId, Doughnut> {
	fn from(s: (Option<AccountId>, Option<Doughnut>)) -> RawOrigin<AccountId, Doughnut> {
		match s {
			(Some(who), None) => RawOrigin::Signed(who),
			(Some(who), Some(doughnut)) => RawOrigin::Delegated(who, doughnut),
			_ => RawOrigin::None,
		}
	}
}

pub type Origin<T> = RawOrigin<<T as Trait>::AccountId, <T as Trait>::Doughnut>;

#[allow(dead_code)]
pub fn ensure_root<OuterOrigin, AccountId, Doughnut>(o: OuterOrigin) -> Result<(), &'static str>
	where OuterOrigin: Into<Result<RawOrigin<AccountId, Doughnut>, OuterOrigin>>
{
	o.into().map(|_| ()).map_err(|_| "bad origin: expected to be a root origin")
}
