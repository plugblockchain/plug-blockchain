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

use crate::{DoughnutRuntime, PlugDoughnut, constants::error_code};
use sp_core::{
	ed25519::{self},
	sr25519::{self},
};
use sp_std::{self, convert::{TryFrom}, prelude::*};
use sp_runtime::{Doughnut};
use sp_runtime::traits::{PlugDoughnutApi, DoughnutApi, DoughnutVerify, SignedExtension, ValidationError, Verify, VerifyError};
use sp_runtime::transaction_validity::{InvalidTransaction, TransactionValidityError, ValidTransaction};
use frame_support::{
	dispatch::DispatchInfo,
	traits::Time,
};

// Proxy calls to the inner Doughnut type and provide Runtime type conversions where required.
impl<Runtime> PlugDoughnutApi for PlugDoughnut<Runtime>
where
	Runtime: DoughnutRuntime,
	Runtime::AccountId: AsRef<[u8]> + From<[u8; 32]>,
{
	type PublicKey = Runtime::AccountId;
	type Signature = [u8; 64];
	type Timestamp = u32;

	fn holder(&self) -> Self::PublicKey {
		match &self.0 {
			Doughnut::V0(v0) => v0.holder().into()
		}
	}
	fn issuer(&self) -> Self::PublicKey {
		match &self.0 {
			Doughnut::V0(v0) => v0.issuer().into()
		}
	}
	fn not_before(&self) -> Self::Timestamp {
		match &self.0 {
			Doughnut::V0(v0) => v0.not_before().into()
		}
	}
	fn expiry(&self) -> Self::Timestamp {
		match &self.0 {
			Doughnut::V0(v0) => v0.expiry().into()
		}
	}
	fn signature(&self) -> Self::Signature {
		match &self.0 {
			Doughnut::V0(v0) => v0.signature().into()
		}
	}
	fn signature_version(&self) -> u8 {
		match &self.0 {
			Doughnut::V0(v0) => v0.signature_version()
		}
	}
	fn payload(&self) -> Vec<u8> {
		match &self.0 {
			Doughnut::V0(v0) => v0.payload()
		}
	}
	fn get_domain(&self, domain: &str) -> Option<&[u8]> {
		match &self.0 {
			Doughnut::V0(v0) => v0.get_domain(domain)
		}
	}
}

// Re-implemented here due to sr25519 verification requiring an external
// wasm VM call when using `no std`
impl<Runtime> DoughnutVerify for PlugDoughnut<Runtime>
where
	Runtime: DoughnutRuntime,
	Runtime::AccountId: AsRef<[u8]> + From<[u8; 32]>,
{
	/// Verify the doughnut signature. Returns `true` on success, false otherwise
	fn verify(&self) -> Result<(), VerifyError> {
		// TODO: This is starting to look like `MultiSignature`, maybe worth refactoring
		match self.signature_version() {
			// sr25519
			0 => {
				let signature = sr25519::Signature(self.signature());
				let issuer = sr25519::Public::try_from(self.issuer().as_ref())
					.map_err(|_| VerifyError::Invalid)?;
				match signature.verify(&self.payload()[..], &issuer) {
					true => Ok(()),
					false => Err(VerifyError::Invalid),
				}
			},
			// ed25519
			1 => {
				let signature = ed25519::Signature(self.signature());
				let issuer = ed25519::Public::try_from(self.issuer().as_ref())
					.map_err(|_| VerifyError::Invalid)?;
				match signature.verify(&self.payload()[..], &issuer) {
					true => Ok(()),
					false => Err(VerifyError::Invalid),
				}
			},
			// signature version unsupported.
			_ => Err(VerifyError::UnsupportedVersion),
		}
	}
}

impl<Runtime> SignedExtension for PlugDoughnut<Runtime>
where
	Runtime: DoughnutRuntime + Eq + Clone + Send + Sync,
	Runtime::AccountId: AsRef<[u8]> + From<[u8; 32]>,
{
	type AccountId = Runtime::AccountId;
	type AdditionalSigned = ();
	type Call = Runtime::Call;
	type DispatchInfo = DispatchInfo;
	type Pre = ();
	const IDENTIFIER: &'static str = "PlugDoughnutSignedExtension";
	fn additional_signed(&self) -> sp_std::result::Result<(), TransactionValidityError> { Ok(()) }
	fn validate(&self, who: &Self::AccountId, _call: &Self::Call, _info: Self::DispatchInfo, _len: usize) -> Result<ValidTransaction, TransactionValidityError>
	{
		// Check doughnut signature verifies
		if let Err(err) = self.verify() {
			let code = match err {
				VerifyError::Invalid => error_code::VERIFY_INVALID,
				VerifyError::UnsupportedVersion => error_code::VERIFY_UNSUPPORTED_VERSION,
				VerifyError::BadSignatureFormat => error_code::VERIFY_BAD_SIGNATURE_FORMAT,
				VerifyError::BadPublicKeyFormat => error_code::VERIFY_BAD_PUBLIC_KEY_FORMAT,
			};
			return Err(InvalidTransaction::Custom(code).into())
		}
		// Convert chain reported timestamp from milliseconds into seconds as per doughnut timestamp spec.
		let now = Runtime::TimestampProvider::now() / 1000_u32.into();
		// Check doughnut is valid for use by `who` at the current timestamp
		if let Err(err) = PlugDoughnutApi::validate(self, who, now) {
			let code = match err {
				ValidationError::HolderIdentityMismatched => error_code::VALIDATION_HOLDER_SIGNER_IDENTITY_MISMATCH,
				ValidationError::Expired => error_code::VALIDATION_EXPIRED,
				ValidationError::Premature => error_code::VALIDATION_PREMATURE,
				ValidationError::Conversion => error_code::VALIDATION_CONVERSION,
			};
			return Err(InvalidTransaction::Custom(code).into())
		}
		Ok(ValidTransaction::default())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use sp_core::crypto::Pair;
	use sp_io::TestExternalities;
	use sp_keyring::{AccountKeyring, Ed25519Keyring};
	use sp_runtime::{DoughnutV0, Doughnut, MultiSignature, traits::IdentifyAccount};

	type Signature = MultiSignature;
	type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

	#[derive(Clone, Eq, PartialEq)]
	pub struct Runtime;

	pub struct FixedTimestampProvider;
	impl Time for FixedTimestampProvider {
		type Moment = u64;
		// Return a constant timestamp (ms)
		fn now() -> Self::Moment {
			50_000
		}
	}

	impl DoughnutRuntime for Runtime {
		type AccountId = AccountId;
		type Call = ();
		type Doughnut = PlugDoughnut<Self>;
		type TimestampProvider = FixedTimestampProvider;
	}

	#[test]
	fn plug_doughnut_validates() {
		let (issuer, holder) = (AccountKeyring::Alice, AccountKeyring::Bob);
		let mut doughnut = DoughnutV0 {
			issuer: issuer.to_raw_public(),
			holder: holder.to_raw_public(),
			expiry: 3000,
			not_before: 0,
			payload_version: 0,
			signature: [1u8; 64].into(),
			signature_version: 0,
			domains: vec![("test".to_string(), vec![0u8])],
		};
		doughnut.signature = issuer.pair().sign(&doughnut.payload()).into();

		let plug_doughnut = PlugDoughnut::<Runtime>::new(Doughnut::V0(doughnut));
		assert!(
			<PlugDoughnut<_> as SignedExtension>::validate(
				&plug_doughnut,
				&holder.to_account_id(), // who
				&(), // Call
				Default::default(), // DispatchInfo
				0usize // len
			).is_ok()
		);
	}

	#[test]
	fn plug_doughnut_does_not_validate_premature() {
		let (issuer, holder) = (AccountKeyring::Alice, AccountKeyring::Bob);
		let mut doughnut = DoughnutV0 {
			issuer: issuer.to_raw_public(),
			holder: holder.to_raw_public(),
			expiry: 3000,
			not_before: 51,
			payload_version: 0,
			signature: [1u8; 64].into(),
			signature_version: 0,
			domains: vec![("test".to_string(), vec![0u8])],
		};
		doughnut.signature = issuer.pair().sign(&doughnut.payload()).into();

		let plug_doughnut = PlugDoughnut::<Runtime>::new(Doughnut::V0(doughnut));
		// premature
		assert_eq!(
			<PlugDoughnut<_> as SignedExtension>::validate(
				&plug_doughnut,
				&holder.to_account_id(), // who
				&(), // Call
				Default::default(), // DispatchInfo
				0usize // len
			),
			Err(InvalidTransaction::Custom(error_code::VALIDATION_PREMATURE).into())
		);
	}

	#[test]
	fn plug_doughnut_does_not_validate_expired() {
		let (issuer, holder) = (AccountKeyring::Alice, AccountKeyring::Bob);
		let mut doughnut = DoughnutV0 {
			issuer: issuer.to_raw_public(),
			holder: holder.to_raw_public(),
			expiry: 49,
			not_before: 0,
			payload_version: 0,
			signature: [1u8; 64].into(),
			signature_version: 0,
			domains: vec![("test".to_string(), vec![0u8])],
		};
		doughnut.signature = issuer.pair().sign(&doughnut.payload()).into();

		let plug_doughnut = PlugDoughnut::<Runtime>::new(Doughnut::V0(doughnut));
		// expired
		assert_eq!(
			<PlugDoughnut<_> as SignedExtension>::validate(
				&plug_doughnut,
				&holder.to_account_id(),
				&(), // Call
				Default::default(), // DispatchInfo
				0usize // len
			),
			Err(InvalidTransaction::Custom(error_code::VALIDATION_EXPIRED).into())
		);
	}

	#[test]
	fn plug_doughnut_does_not_validate_bad_holder() {
		let (issuer, holder) = (AccountKeyring::Alice, AccountKeyring::Bob);
		let mut doughnut = DoughnutV0 {
			issuer: issuer.to_raw_public(),
			holder: holder.to_raw_public(),
			expiry: 3000,
			not_before: 0,
			payload_version: 0,
			signature: [1u8; 64].into(),
			signature_version: 0,
			domains: vec![("test".to_string(), vec![0u8])],
		};
		doughnut.signature = issuer.pair().sign(&doughnut.payload()).into();

		let plug_doughnut = PlugDoughnut::<Runtime>::new(Doughnut::V0(doughnut));
		// Charlie is not the holder
		assert_eq!(
			<PlugDoughnut<_> as SignedExtension>::validate(
				&plug_doughnut,
				&AccountKeyring::Charlie.to_account_id(), // who
				&(), // Call
				Default::default(), // DispatchInfo
				0usize // len
			),
			Err(InvalidTransaction::Custom(error_code::VALIDATION_HOLDER_SIGNER_IDENTITY_MISMATCH).into())
		);
	}

	#[test]
	fn plug_doughnut_verifies_sr25519_signature() {
		let (issuer, holder) = (AccountKeyring::Alice, AccountKeyring::Bob);
		let mut doughnut_v0 = DoughnutV0 {
			issuer: issuer.to_raw_public(),
			holder: holder.to_raw_public(),
			expiry: 0,
			not_before: 0,
			payload_version: 0,
			signature: [1u8; 64].into(),
			signature_version: 0,
			domains: vec![("test".to_string(), vec![0u8])],
		};
		doughnut_v0.signature = issuer.pair().sign(&doughnut_v0.payload()).into();

		let doughnut = Doughnut::V0(doughnut_v0);
		let plug_doughnut = PlugDoughnut::<Runtime>::new(doughnut);
		assert!(plug_doughnut.verify().is_ok());
	}

	#[test]
	fn plug_doughnut_does_not_verify_sr25519_signature() {
		let (issuer, holder) = (AccountKeyring::Alice, AccountKeyring::Bob);
		let mut doughnut_v0 = DoughnutV0 {
			issuer: issuer.to_raw_public(),
			holder: holder.to_raw_public(),
			expiry: 0,
			not_before: 0,
			payload_version: 0,
			signature: [1u8; 64].into(),
			signature_version: 0,
			domains: vec![("test".to_string(), vec![0u8])],
		};
		doughnut_v0.signature = holder.sign(&doughnut_v0.payload()).into();

		let doughnut = Doughnut::V0(doughnut_v0);
		let plug_doughnut = PlugDoughnut::<Runtime>::new(doughnut);
		assert_eq!(plug_doughnut.verify(), Err(VerifyError::Invalid));
	}

	#[test]
	fn plug_doughnut_verifies_ed25519_signature() {
		let (issuer, holder) = (Ed25519Keyring::Alice, Ed25519Keyring::Bob);
		let mut doughnut_v0 = DoughnutV0 {
			issuer: issuer.to_raw_public(),
			holder: holder.to_raw_public(),
			expiry: 0,
			not_before: 0,
			payload_version: 0,
			signature: [1u8; 64].into(),
			signature_version: 1,
			domains: vec![("test".to_string(), vec![0u8])],
		};
		doughnut_v0.signature = issuer.pair().sign(&doughnut_v0.payload()).into();

		let doughnut = Doughnut::V0(doughnut_v0);
		let plug_doughnut = PlugDoughnut::<Runtime>::new(doughnut);

		// Externalities is required for ed25519 signature verification
		TestExternalities::default().execute_with(|| {
			assert!(plug_doughnut.verify().is_ok());
		});
	}

	#[test]
	fn plug_doughnut_does_not_verify_ed25519_signature() {
		let (issuer, holder) = (Ed25519Keyring::Alice, Ed25519Keyring::Bob);
		let mut doughnut_v0 = DoughnutV0 {
			issuer: issuer.to_raw_public(),
			holder: holder.to_raw_public(),
			expiry: 0,
			not_before: 0,
			payload_version: 0,
			signature: [1u8; 64].into(),
			signature_version: 1,
			domains: vec![("test".to_string(), vec![0u8])],
		};
		// !holder signs the doughnuts
		doughnut_v0.signature = holder.sign(&doughnut_v0.payload()).into();

		let doughnut = Doughnut::V0(doughnut_v0);
		let plug_doughnut = PlugDoughnut::<Runtime>::new(doughnut);

		// Externalities is required for ed25519 signature verification
		TestExternalities::default().execute_with(|| {
			assert_eq!(plug_doughnut.verify(), Err(VerifyError::Invalid));
		});
	}

	#[test]
	fn plug_doughnut_does_not_verify_unknown_signature_version() {
		let (issuer, holder) = (Ed25519Keyring::Alice, Ed25519Keyring::Bob);
		let mut doughnut_v0 = DoughnutV0 {
			issuer: issuer.to_raw_public(),
			holder: holder.to_raw_public(),
			expiry: 0,
			not_before: 0,
			payload_version: 0,
			signature: [1u8; 64].into(),
			signature_version: 200,
			domains: vec![("test".to_string(), vec![0u8])],
		};
		doughnut_v0.signature = issuer.pair().sign(&doughnut_v0.payload()).into();

		let plug_doughnut = PlugDoughnut::<Runtime>::new(Doughnut::V0(doughnut_v0));
		assert_eq!(
			<PlugDoughnut<_> as SignedExtension>::validate(
				&plug_doughnut,
				&holder.to_account_id(), // who
				&(), // Call
				Default::default(), // DispatchInfo
				0usize // len
			),
			Err(InvalidTransaction::Custom(error_code::VERIFY_UNSUPPORTED_VERSION).into())
		);
	}

	#[test]
	fn plug_doughnut_proxies_to_inner_doughnut() {
		let issuer = [0u8; 32];
		let holder = [1u8; 32];
		let expiry = 55555;
		let not_before = 123;
		let signature = [1u8; 64];
		let signature_version = 1;

		let doughnut_v0 = DoughnutV0 {
			issuer,
			holder,
			expiry,
			not_before,
			payload_version: 0,
			signature: signature.into(),
			signature_version,
			domains: vec![("test".to_string(), vec![0u8])],
		};

		let doughnut = PlugDoughnut::<Runtime>::new(Doughnut::V0(doughnut_v0.clone()));

		assert_eq!(Into::<[u8; 32]>::into(doughnut.issuer()), issuer);
		assert_eq!(Into::<[u8; 32]>::into(doughnut.holder()), holder);
		assert_eq!(doughnut.expiry(), expiry);
		assert_eq!(doughnut.not_before(), not_before);
		assert_eq!(doughnut.signature_version(), signature_version);
		assert_eq!(&doughnut.signature()[..], &signature[..]);
		assert_eq!(doughnut.payload(), doughnut_v0.payload());
	}

	#[test]
	fn plug_doughnut_does_not_verify_invalid_signature() {
		let (issuer, holder) = (AccountKeyring::Alice, AccountKeyring::Bob);
		let mut doughnut = DoughnutV0 {
			issuer: issuer.to_raw_public(),
			holder: holder.to_raw_public(),
			expiry: 0,
			not_before: 0,
			payload_version: 0,
			signature: [1u8; 64].into(),
			signature_version: 0,
			domains: vec![("test".to_string(), vec![0u8])],
		};
		// Signed by holder!
		doughnut.signature = holder.pair().sign(&doughnut.payload()).into();

		let plug_doughnut = PlugDoughnut::<Runtime>::new(Doughnut::V0(doughnut));

		assert_eq!(
			<PlugDoughnut<_> as SignedExtension>::validate(
				&plug_doughnut,
				&holder.to_account_id(), // who
				&(), // Call
				Default::default(), // DispatchInfo
				0usize // len
			),
			Err(InvalidTransaction::Custom(error_code::VERIFY_INVALID).into())
		);
	}
}
