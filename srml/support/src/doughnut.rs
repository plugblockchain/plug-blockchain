pub use crate::rstd::prelude::Vec;
use sr_primitives::codec::{Decode, Encode, Input};
use sr_primitives::traits::Verify;

#[derive(Clone, Eq, PartialEq, Default, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Certificate<AccountId> {
	pub expires: u64,
	pub version: u32,
	pub holder: AccountId,
	pub not_before: u64,
	//	use vec of tuple to work as a key value map
	pub permissions: Vec<(Vec<u8>, Vec<u8>)>,
	pub issuer: AccountId,
}

#[derive(Clone, Eq, PartialEq, Default, Encode)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Doughnut<AccountId, Signature> {
	pub certificate: Certificate<AccountId>,
	pub signature: Signature,
}

impl<AccountId, Signature> Decode for Doughnut<AccountId, Signature>
	where
		AccountId: Decode,
		Signature: Decode,
{
	fn decode<I: Input>(input: &mut I) -> Option<Self> {
		Some(Doughnut {
			certificate: Decode::decode(input)?,
			signature: Decode::decode(input)?,
		})
	}
}

impl<AccountId, Signature> Doughnut<AccountId, Signature>
	where
		Signature: Verify<Signer = AccountId> + Encode,
		AccountId: Encode,
{
	pub fn validate(&self) -> bool {
		if self
			.signature
			.verify(self.certificate.encode().as_slice(), &self.certificate.issuer)
		{
			// TODO: ensure doughnut hasn't been revoked
			return true;
		} else {
			return false;
		}
	}
}
