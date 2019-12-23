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

use crate::{DoughnutRuntime, PlugDoughnut};
use primitives::{
	ed25519::{self},
	sr25519::{self},
};
use rstd::{self, prelude::*};
use sp_runtime::traits::{DoughnutApi, DoughnutVerify, Member, SignedExtension, Verify, VerifyError};
use sp_runtime::transaction_validity::{InvalidTransaction, TransactionValidityError, ValidTransaction};
use support::{
	dispatch::DispatchInfo,
	Parameter,
	traits::Time,
};

// Proxy calls to the inner Doughnut type and provide Runtime type conversions where required.
impl<Doughnut, Runtime> DoughnutApi for PlugDoughnut<Doughnut, Runtime>
where
	Doughnut: DoughnutApi<Signature=[u8; 64]>,
	<Doughnut as DoughnutApi>::PublicKey: Into<[u8; 32]>,
	Runtime: DoughnutRuntime,
	Runtime::AccountId: AsRef<[u8]> + From<[u8; 32]>,
{
	type PublicKey = Runtime::AccountId;
	type Signature = <Doughnut as DoughnutApi>::Signature;
	type Timestamp = <Doughnut as DoughnutApi>::Timestamp;

	fn holder(&self) -> Self::PublicKey {
		From::from(self.0.holder().into())
	}
	fn issuer(&self) -> Self::PublicKey {
		From::from(self.0.issuer().into())
	}
	fn not_before(&self) -> Self::Timestamp {
		self.0.not_before()
	}
	fn expiry(&self) -> Self::Timestamp {
		self.0.expiry()
	}
	fn signature(&self) -> Self::Signature {
		self.0.signature()
	}
	fn signature_version(&self) -> u8 {
		self.0.signature_version()
	}
	fn payload(&self) -> Vec<u8> {
		self.0.payload()
	}
	fn get_domain(&self, domain: &str) -> Option<&[u8]> {
		self.0.get_domain(domain)
	}
}

// Re-implemented here due to sr25519 verification requiring an external
// wasm VM call when using `no std`
impl<Doughnut, Runtime> DoughnutVerify for PlugDoughnut<Doughnut, Runtime>
where
	Doughnut: DoughnutApi<Signature=[u8; 64]>,
	<Doughnut as DoughnutApi>::PublicKey: Into<[u8; 32]>,
	Runtime: DoughnutRuntime,
{
	/// Verify the doughnut signature. Returns `true` on success, false otherwise
	fn verify(&self) -> Result<(), VerifyError> {
		// TODO: This is starting to look like `MultiSignature`, maybe worth refactoring
		match self.0.signature_version() {
			// sr25519
			0 => {
				let signature = sr25519::Signature(self.0.signature());
				let issuer = sr25519::Public(self.0.issuer().into());
				match signature.verify(&self.0.payload()[..], &issuer) {
					true => Ok(()),
					false => Err(VerifyError::Invalid),
				}
			},
			// ed25519
			1 => {
				let signature = ed25519::Signature(self.0.signature());
				let issuer = ed25519::Public(self.0.issuer().into());
				match signature.verify(&self.0.payload()[..], &issuer) {
					true => Ok(()),
					false => Err(VerifyError::Invalid),
				}
			},
			// signature version unsupported.
			_ => Err(VerifyError::UnsupportedVersion),
		}
	}
}

impl<Doughnut, Runtime> SignedExtension for PlugDoughnut<Doughnut, Runtime>
where
	Doughnut: DoughnutApi<Signature=[u8; 64]> + Member + Parameter,
	<Doughnut as DoughnutApi>::PublicKey: Into<[u8; 32]>,
	Runtime: DoughnutRuntime + Eq + Clone + Send + Sync,
	Runtime::AccountId: AsRef<[u8]>,
{
	type AccountId = Runtime::AccountId;
	type AdditionalSigned = ();
	type Call = Runtime::Call;
	type DispatchInfo = DispatchInfo;
	type Pre = ();
	fn additional_signed(&self) -> rstd::result::Result<(), TransactionValidityError> { Ok(()) }
	fn validate(&self, who: &Self::AccountId, _call: &Self::Call, _info: Self::DispatchInfo, _len: usize) -> Result<ValidTransaction, TransactionValidityError>
	{
		if self.verify().is_err() {
			// 170 == invalid signature on doughnut
			return Err(InvalidTransaction::Custom(170).into())
		}
		if let Err(_) = self.0.validate(who, Runtime::TimestampProvider::now()) {
			// 171 == use of doughnut by who at the current timestamp is invalid
			return Err(InvalidTransaction::Custom(171).into())
		}
		Ok(ValidTransaction::default())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use primitives::crypto::Pair;
	use runtime_io::TestExternalities;
	use sp_runtime::{DoughnutV0, Doughnut, MultiSignature, traits::IdentifyAccount};

	type Signature = MultiSignature;
	type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

	#[derive(Clone, Eq, PartialEq)]
	pub struct Runtime;

	pub struct TimestampProvider;
	impl Time for TimestampProvider {
		type Moment = u64;
		fn now() -> Self::Moment {
			0
		}
	}
	impl DoughnutRuntime for Runtime {
		type AccountId = AccountId;
		type Call = ();
		type Doughnut = PlugDoughnut<Doughnut, Self>;
		type TimestampProvider = TimestampProvider;
	}

	// TODO: Re-write using `sp-keyring`
	#[test]
	fn plug_doughnut_validates() {
		let issuer = sr25519::Pair::from_string("//Alice", None).unwrap();
		let holder = sr25519::Pair::from_string("//Bob", None).unwrap();
		let doughnut = Doughnut::V0(DoughnutV0 {
			issuer: issuer.public().into(),
			holder: holder.public().into(),
			expiry: 3000,
			not_before: 0,
			payload_version: 0,
			signature: [1u8; 64].into(),
			signature_version: 0,
			domains: vec![("test".to_string(), vec![0u8])],
		});
		let plug_doughnut = PlugDoughnut::<_, Runtime>::new(doughnut);
		assert!(
			<PlugDoughnut<_, _> as DoughnutApi>::validate(&plug_doughnut, holder.public(), 100).is_ok()
		);
	}

	#[test]
	fn plug_doughnut_does_not_validate() {
		let issuer = sr25519::Pair::from_string("//Alice", None).unwrap();
		let holder = sr25519::Pair::from_string("//Bob", None).unwrap();
		let signer = sr25519::Pair::from_string("//Charlie", None).unwrap();
		let doughnut = Doughnut::V0(DoughnutV0 {
			issuer: issuer.public().into(),
			holder: holder.public().into(),
			expiry: 3000,
			not_before: 1000,
			payload_version: 0,
			signature: [1u8; 64].into(),
			signature_version: 0,
			domains: vec![("test".to_string(), vec![0u8])],
		});
		let plug_doughnut = PlugDoughnut::<_, Runtime>::new(doughnut);
		// premature
		assert!(
			<PlugDoughnut<_, _> as DoughnutApi>::validate(&plug_doughnut, holder.public(), 999).is_err()
		);
		// expired
		assert!(
			<PlugDoughnut<_, _> as DoughnutApi>::validate(&plug_doughnut, holder.public(), 3001).is_err()
		);
		// signer is not holder
		assert!(
			<PlugDoughnut<_, _> as DoughnutApi>::validate(&plug_doughnut, signer.public(), 100).is_err()
		);
	}

	#[test]
	fn plug_doughnut_verifies_sr25519_signature() {
		let issuer = sr25519::Pair::from_string("//Alice", None).unwrap();
		let holder = sr25519::Pair::from_string("//Bob", None).unwrap();
		let mut doughnut_v0 = DoughnutV0 {
			issuer: issuer.public().into(),
			holder: holder.public().into(),
			expiry: 0,
			not_before: 0,
			payload_version: 0,
			signature: [1u8; 64].into(),
			signature_version: 0,
			domains: vec![("test".to_string(), vec![0u8])],
		};
		doughnut_v0.signature = issuer.sign(&doughnut_v0.payload()).into();

		let doughnut = Doughnut::V0(doughnut_v0);
		let plug_doughnut = PlugDoughnut::<_, Runtime>::new(doughnut);
		assert!(plug_doughnut.verify().is_ok());
	}

	#[test]
	fn plug_doughnut_does_not_verify_sr25519_signature() {
		let issuer = sr25519::Pair::from_string("//Alice", None).unwrap();
		let holder = sr25519::Pair::from_string("//Bob", None).unwrap();
		let mut doughnut_v0 = DoughnutV0 {
			issuer: issuer.public().into(),
			holder: holder.public().into(),
			expiry: 0,
			not_before: 0,
			payload_version: 0,
			signature: [1u8; 64].into(),
			signature_version: 0,
			domains: vec![("test".to_string(), vec![0u8])],
		};
		doughnut_v0.signature = holder.sign(&doughnut_v0.payload()).into();

		let doughnut = Doughnut::V0(doughnut_v0);
		let plug_doughnut = PlugDoughnut::<_, Runtime>::new(doughnut);
		assert_eq!(plug_doughnut.verify(), Err(VerifyError::Invalid));
	}

	#[test]
	fn plug_doughnut_verifies_ed25519_signature() {
		let issuer = ed25519::Pair::from_legacy_string("//Alice", None);
		let holder = ed25519::Pair::from_legacy_string("//Bob", None);
		let mut doughnut_v0 = DoughnutV0 {
			issuer: issuer.public().into(),
			holder: holder.public().into(),
			expiry: 0,
			not_before: 0,
			payload_version: 0,
			signature: [1u8; 64].into(),
			signature_version: 1,
			domains: vec![("test".to_string(), vec![0u8])],
		};
		doughnut.signature = issuer.sign(&doughnut.payload()).into();
		doughnut_v0.signature = issuer.sign(&doughnut_v0.payload()).into();

		let doughnut = Doughnut::V0(doughnut_v0);
		let plug_doughnut = PlugDoughnut::<_, Runtime>::new(doughnut);

		// Externalities is required for ed25519 signature verification
		TestExternalities::default().execute_with(|| {
			assert!(plug_doughnut.verify().is_ok());
		});
	}

	#[test]
	fn plug_doughnut_does_not_verify_ed25519_signature() {
		let issuer = ed25519::Pair::from_legacy_string("//Alice", None);
		let holder = ed25519::Pair::from_legacy_string("//Bob", None);
		let mut doughnut_v0 = DoughnutV0 {
			issuer: issuer.public().into(),
			holder: holder.public().into(),
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
		let plug_doughnut = PlugDoughnut::<_, Runtime>::new(doughnut);

		// Externalities is required for ed25519 signature verification
		TestExternalities::default().execute_with(|| {
			assert_eq!(plug_doughnut.verify(), Err(VerifyError::Invalid));
		});
	}

	#[test]
	fn plug_doughnut_does_not_verify_unknown_signature_version() {
		let issuer = ed25519::Pair::from_legacy_string("//Alice", None);
		let holder = ed25519::Pair::from_legacy_string("//Bob", None);
		let mut doughnut_v0 = DoughnutV0 {
			issuer: issuer.public().into(),
			holder: holder.public().into(),
			expiry: 0,
			not_before: 0,
			payload_version: 0,
			signature: [1u8; 64].into(),
			signature_version: 200,
			domains: vec![("test".to_string(), vec![0u8])],
		};
		doughnut_v0.signature = issuer.sign(&doughnut_v0.payload()).into();

		let doughnut = Doughnut::V0(doughnut_v0);
		let plug_doughnut = PlugDoughnut::<_, Runtime>::new(doughnut);
		assert_eq!(plug_doughnut.verify(), Err(VerifyError::UnsupportedVersion));
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

		let doughnut = PlugDoughnut::<_, Runtime>::new(Doughnut::V0(doughnut_v0));

		assert_eq!(Into::<[u8; 32]>::into(doughnut.issuer()), issuer);
		assert_eq!(Into::<[u8; 32]>::into(doughnut.holder()), holder);
		assert_eq!(doughnut.expiry(), expiry);
		assert_eq!(doughnut.not_before(), not_before);
		assert_eq!(doughnut.signature_version(), signature_version);
		assert_eq!(&doughnut.signature()[..], &signature[..]);
		assert_eq!(doughnut.payload(), doughnut.0.payload());
	}
}
