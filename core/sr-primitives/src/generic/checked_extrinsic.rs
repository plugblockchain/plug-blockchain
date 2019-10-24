// Copyright 2017-2019 Parity Technologies (UK) Ltd.
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

//! Generic implementation of an extrinsic that has passed the verification
//! stage.

use crate::traits::{
	self, Dispatchable, DoughnutApi, MaybeDisplay, MaybeDoughnut, Member, SignedExtension, ValidateUnsigned,
};
use crate::transaction_validity::TransactionValidity;
use crate::weights::{GetDispatchInfo, DispatchInfo};

/// Definition of something that the external world might want to say; its
/// existence implies that it has been checked and is good, particularly with
/// regards to the signature.
#[derive(PartialEq, Eq, Clone, primitives::RuntimeDebug)]
pub struct CheckedExtrinsic<AccountId, Call, Extra> {
	/// Who this purports to be from and the number of extrinsics have come before
	/// from the same signer, if anyone (note this is not a signature).
	pub signed: Option<(AccountId, Extra)>,

	/// The function that should be called.
	pub function: Call,
}

impl<AccountId, Call, Extra, Origin, Doughnut> traits::Applyable
for
	CheckedExtrinsic<AccountId, Call, Extra>
where
	AccountId: Member + MaybeDisplay + AsRef<[u8]>,
	Call: Member + Dispatchable<Origin=Origin>,
	Extra: SignedExtension<AccountId=AccountId, Call=Call> + MaybeDoughnut<Doughnut=Doughnut>,
	Origin: From<(Option<AccountId>, Option<Doughnut>)>,
	Doughnut: Member + DoughnutApi<PublicKey=AccountId>,
{
	type AccountId = AccountId;
	type Call = Call;

	fn sender(&self) -> Option<&Self::AccountId> {
		self.signed.as_ref().map(|x| &x.0)
	}

	fn validate<U: ValidateUnsigned<Call = Self::Call>>(
		&self,
		info: DispatchInfo,
		len: usize,
	) -> TransactionValidity {
		if let Some((ref id, ref extra)) = self.signed {
			Extra::validate(extra, id, &self.function, info, len)
		} else {
			let valid = Extra::validate_unsigned(&self.function, info, len)?;
			Ok(valid.combine_with(U::validate_unsigned(&self.function)?))
		}
	}

	fn apply(
		self,
		info: DispatchInfo,
		len: usize,
	) -> crate::ApplyResult {
		let (pre, res) = if let Some((id, extra)) = self.signed {
			let pre = Extra::pre_dispatch(&extra, &id, &self.function, info, len)?;
			if let Some(doughnut) = extra.doughnut() {
				// A delegated transaction
				(pre, self.function.dispatch(Origin::from((Some(doughnut.issuer()), Some(doughnut)))))
			} else {
				// An ordinary signed transaction
				(pre, self.function.dispatch(Origin::from((Some(id), None))))
			}
		} else {
			// An inherent unsiged transaction
			let pre = Extra::pre_dispatch_unsigned(&self.function, info, len)?;
			(pre, self.function.dispatch(Origin::from((None, None))))
		};
		Extra::post_dispatch(pre, info, len);
		Ok(res.map_err(Into::into))
	}
}

impl<AccountId, Call, Extra> GetDispatchInfo for CheckedExtrinsic<AccountId, Call, Extra>
where
	Call: GetDispatchInfo,
{
	fn get_dispatch_info(&self) -> DispatchInfo {
		self.function.get_dispatch_info()
	}
}
