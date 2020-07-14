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

//!
//! Doughnut + Extrinsic + Executive integration tests
//!
#![cfg(test)]
use pallet_balances::Call as BalancesCall;
use codec::{Encode};
use prml_doughnut::{DoughnutRuntime, PlugDoughnut, error_code};
use sp_core::{crypto::UncheckedFrom, H256};
use sp_keyring::AccountKeyring;
use sp_runtime::{
	DispatchError, Doughnut, DoughnutV0, MultiSignature,
	generic::{self, Era}, Perbill, testing::{Block, Digest, Header},
	traits::{IdentifyAccount, IdentityLookup, Header as HeaderT, BlakeTwo256, Verify, ConvertInto, PlugDoughnutApi, DoughnutSigning},
	transaction_validity::{InvalidTransaction, TransactionValidity, TransactionValidityError, UnknownTransaction, TransactionSource},
};
#[allow(deprecated)]
use sp_runtime::traits::ValidateUnsigned;
use frame_support::{
	impl_outer_event, impl_outer_origin, parameter_types, impl_outer_dispatch,
	additional_traits::{DelegatedDispatchVerifier},
	traits::{Currency, Time},
};
use frame_system as system;
use sp_std::any::Any;

type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
type Address = AccountId;
type Index = u32;
type Signature = MultiSignature;
type System = frame_system::Module<Runtime>;
type Balances = pallet_balances::Module<Runtime>;

impl_outer_origin! {
	pub enum Origin for Runtime {}
}

impl_outer_event!{
	pub enum MetaEvent for Runtime {
		system, pallet_balances<T>,
	}
}
impl_outer_dispatch! {
	pub enum Call for Runtime where origin: Origin {
		frame_system::System,
		pallet_balances::Balances,
	}
}

pub struct MockDelegatedDispatchVerifier<T: frame_system::Trait>(sp_std::marker::PhantomData<T>);
impl<T: frame_system::Trait> DelegatedDispatchVerifier for MockDelegatedDispatchVerifier<T> {
	type Doughnut = T::Doughnut;
	type AccountId = T::AccountId;
	const DOMAIN: &'static str = "test";
	fn verify_dispatch(
		doughnut: &T::Doughnut,
		_module: &str,
		_method: &str,
		args: Vec<(&str, &dyn Any)>,
	) -> Result<(), &'static str> {
		// Check the "test" domain has a byte set to `1` for Ok, fail otherwise
		let verify = doughnut.get_domain(Self::DOMAIN).unwrap()[0];
		let mut verify_args = true;
		for (type_string, value_any) in args {
			match type_string {
				"T::Balance" => {
					if value_any.downcast_ref::<u64>() == Some(&0x1234567890) {
						verify_args = false;
					}
				}
				_ => {}
			}
		}

		if verify == 1 && verify_args {
			Ok(())
		} else {
			Err("dispatch unverified")
		}
	}
}

// Create a minimal runtime to verify doughnut's are properly integrated
// This means we are using full-blown types i.e sr25519 Public Keys for AccountId and CheckedExtrinsic
// Workaround for https://github.com/rust-lang/rust/issues/26925 . Remove when sorted.
#[derive(Clone, Eq, PartialEq)]
pub struct Runtime;
parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: u32 = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
}
impl frame_system::Trait for Runtime {
	type Origin = Origin;
	type Index = Index;
	type Call = Call;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = MetaEvent;
	type BlockHashCount = BlockHashCount;
	type MaximumBlockWeight = MaximumBlockWeight;
	type AvailableBlockRatio = AvailableBlockRatio;
	type MaximumBlockLength = MaximumBlockLength;
	type Version = ();
	type ModuleToIndex = ();
	type Doughnut = PlugDoughnut<Runtime>;
	type DelegatedDispatchVerifier = MockDelegatedDispatchVerifier<Runtime>;
}
pub struct TimestampProvider;
impl Time for TimestampProvider {
	type Moment = u64;
	fn now() -> Self::Moment {
		// Return a fixed timestamp (ms)
		// It should be > 0 to allow doughnut timestamp validation checks to pass
		10_000
	}
}
impl DoughnutRuntime for Runtime {
	type AccountId = <Self as frame_system::Trait>::AccountId;
	type Call = <Self as frame_system::Trait>::Call;
	type Doughnut = <Self as frame_system::Trait>::Doughnut;
	type TimestampProvider = TimestampProvider;
}
parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
	pub const TransferFee: u64 = 0;
	pub const CreationFee: u64 = 0;
}
impl pallet_balances::Trait for Runtime {
	type Balance = u64;
	type OnReapAccount = System;
	type OnNewAccount = ();
	type Event = MetaEvent;
	type DustRemoval = ();
	type TransferPayment = ();
	type ExistentialDeposit = ExistentialDeposit;
	type CreationFee = CreationFee;
}

parameter_types! {
	pub const TransactionBaseFee: u64 = 10;
	pub const TransactionByteFee: u64 = 0;
}
impl pallet_transaction_payment::Trait for Runtime {
	type Currency = Balances;
	type OnTransactionPayment = ();
	type TransactionBaseFee = TransactionBaseFee;
	type TransactionByteFee = TransactionByteFee;
	type WeightToFee = ConvertInto;
	type FeeMultiplierUpdate = ();
}

#[allow(deprecated)] // Allow ValidateUnsigned
impl ValidateUnsigned for Runtime {
	type Call = Call;

	fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
		match call {
			Call::Balances(BalancesCall::set_balance(_, _, _)) => Ok(Default::default()),
			_ => UnknownTransaction::NoUnsignedValidator.into(),
		}
	}
}
type SignedExtra = (
	Option<PlugDoughnut<Runtime>>,
	frame_system::CheckVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_transaction_payment::ChargeTransactionPayment<Runtime>
);

type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>; // Just a `CheckedExtrinsic` with type parameters set
type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>; // Just an `UnheckedExtrinsic` with type parameters set
type Executive = frame_executive::Executive<Runtime, Block<UncheckedExtrinsic>, frame_system::ChainContext<Runtime>, Runtime, ()>;

/// Returns transaction extra.
fn signed_extra(nonce: Index, fee: u64, doughnut: Option<PlugDoughnut<Runtime>>) -> SignedExtra {
	(
		doughnut,
		frame_system::CheckVersion::new(),
		frame_system::CheckGenesis::new(),
		frame_system::CheckEra::from(Era::mortal(256, 0)),
		frame_system::CheckNonce::from(nonce),
		frame_system::CheckWeight::new(),
		pallet_transaction_payment::ChargeTransactionPayment::from(fee),
	)
}

/// Sign a given `CheckedExtrinsic` (lifted from `node/keyring`)
fn sign_extrinsic(xt: CheckedExtrinsic) -> UncheckedExtrinsic {
	match xt.signed {
		Some((signed, extra)) => {
			let raw_payload = generic::SignedPayload::new(xt.function, extra.clone()).expect("signed payload is valid");
			let signed_ = UncheckedFrom::<[u8; 32]>::unchecked_from(signed.clone().into()); // `AccountId32` => `sr25519::Public`
			let key = AccountKeyring::from_public(&signed_).unwrap();
			let signature = raw_payload.using_encoded(|payload| {
				if payload.len() > 256 {
					key.sign(&sp_io::hashing::blake2_256(payload))
				} else {
					key.sign(payload)
				}
			}).into();
			let (function, extra, _) = raw_payload.deconstruct();
			UncheckedExtrinsic {
				signature: Some((signed, signature, extra)),
				function,
			}
		}
		None => UncheckedExtrinsic {
			signature: None,
			function: xt.function,
		},
	}
}

/// Create a valid `DoughnutV0` given an `issuer` and `holder`
fn make_doughnut(issuer: AccountId, holder: AccountId, not_before: Option<u32>, expiry: Option<u32>, permission_domain_verify: bool) -> Doughnut {
	let issuer_pk = UncheckedFrom::<[u8; 32]>::unchecked_from(issuer.clone().into()); // `AccountId32` => `sr25519::Public`
	let issuer_key = AccountKeyring::from_public(&issuer_pk).unwrap();
	let mut doughnut = DoughnutV0 {
		issuer: issuer.into(),
		holder: holder.into(),
		expiry: expiry.unwrap_or(u32::max_value()),
		not_before: not_before.unwrap_or(0),
		payload_version: 0,
		signature_version: 0, // sr25519
		signature: [0u8; 64].into(),
		domains: vec![("test".to_string(), vec![permission_domain_verify as u8])],
	};
	assert!(doughnut.sign_sr25519(&issuer_key.to_ed25519_bytes()).is_ok());
	Doughnut::V0(doughnut)
}

fn transaction_error_from_code(code: u8) -> TransactionValidityError {
	TransactionValidityError::Invalid(
		InvalidTransaction::Custom(code)
	)
}

// TODO: These tests are very repitious, could be DRYed up with macros
#[test]
fn delegated_dispatch_works() {
	// Submit an extrinsic with attached doughnut proof to the test Runtime.
	// The extrinsic is instigated by Bob (doughnut holder) on Alice's authority (doughnut issuer)
	// funds are transferred from Alice to Charlie (receiver)
	// Note: We do not verify the contents of the doughnut's permission `domains` section
	let issuer_alice: AccountId = AccountKeyring::Alice.into();
	let holder_bob: AccountId = AccountKeyring::Bob.into();
	let receiver_charlie: AccountId = AccountKeyring::Charlie.into();

	// Setup storage
	let mut t = frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap();
	pallet_balances::GenesisConfig::<Runtime> {
		balances: vec![(issuer_alice.clone(), 10_011), (holder_bob.clone(), 10_011)],
	}.assimilate_storage(&mut t).unwrap();

	// The doughnut proof is wrapped for embeddeding in extrinsic
	let doughnut = PlugDoughnut::<Runtime>::new(
		make_doughnut(
			issuer_alice.clone(),
			holder_bob.clone(),
			None,
			None,
			true,
		)
	);

	let mut t = sp_io::TestExternalities::new(t);
	t.execute_with(|| {
		// Setup extrinsic
		let xt = CheckedExtrinsic {
			signed: Some((
				holder_bob.clone(),
				signed_extra(0, 0, Some(doughnut)),
			)),
			function: Call::Balances(BalancesCall::transfer(receiver_charlie.clone().into(), 69)),
		};
		let uxt = sign_extrinsic(xt);

		Executive::initialize_block(&Header::new(
			1,
			H256::default(),
			H256::default(),
			[69u8; 32].into(),
			Digest::default(),
		));

		// Submit an extrinsic with attached doughnut proof to the test Runtime.
		let r = Executive::apply_extrinsic(uxt);
		assert!(r.is_ok());
		assert_eq!(<pallet_balances::Module<Runtime>>::total_balance(&issuer_alice), 10_011 - 69); // 69 transferred
		assert_eq!(<pallet_balances::Module<Runtime>>::total_balance(&holder_bob), 10_011 - 10 - 1024); // fees deducted
		assert_eq!(<pallet_balances::Module<Runtime>>::total_balance(&receiver_charlie), 69); // Received 69
	});
}

#[test]
fn delegated_dispatch_fails_when_extrinsic_signer_is_not_doughnut_holder() {
	// Submit an extrinsic with attached doughnut proof to the test Runtime.
	// The extrinsic is instigated by Charlie, however the doughnut proof is to Bob (doughnut holder) on Alice's authority (doughnut issuer)
	// Funds are not transferred
	let issuer_alice: AccountId = AccountKeyring::Alice.into();
	let holder_bob: AccountId = AccountKeyring::Bob.into();
	let receiver_charlie: AccountId = AccountKeyring::Charlie.into();

	// Setup storage
	let mut t = frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap();
	pallet_balances::GenesisConfig::<Runtime> {
		balances: vec![(issuer_alice.clone(), 10_011), (holder_bob.clone(), 10_011)],
	}.assimilate_storage(&mut t).unwrap();

	let doughnut = PlugDoughnut::<Runtime>::new(
		make_doughnut(
			issuer_alice.clone(),
			holder_bob.clone(),
			None,
			None,
			true,
		)
	);

	let mut t = sp_io::TestExternalities::new(t);
	t.execute_with(|| {
		let xt = CheckedExtrinsic {
			signed: Some((
				receiver_charlie.clone(),
				signed_extra(0, 0, Some(doughnut)),
			)),
			function: Call::Balances(BalancesCall::transfer(receiver_charlie.clone().into(), 69)),
		};
		let uxt = sign_extrinsic(xt);

		Executive::initialize_block(&Header::new(
			1,
			H256::default(),
			H256::default(),
			[69u8; 32].into(),
			Digest::default(),
		));

		assert_eq!(
			Executive::apply_extrinsic(uxt),
			Err(transaction_error_from_code(error_code::VALIDATION_HOLDER_SIGNER_IDENTITY_MISMATCH))
		);
	});
}

#[test]
fn delegated_dispatch_fails_when_doughnut_is_expired() {
	let issuer_alice: AccountId = AccountKeyring::Alice.into();
	let holder_bob: AccountId = AccountKeyring::Bob.into();
	let receiver_charlie: AccountId = AccountKeyring::Charlie.into();

	let mut t = frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap();
	pallet_balances::GenesisConfig::<Runtime> {
		balances: vec![(issuer_alice.clone(), 10_011), (holder_bob.clone(), 10_011)],
	}.assimilate_storage(&mut t).unwrap();

	let doughnut = PlugDoughnut::<Runtime>::new(
		make_doughnut(
			issuer_alice.clone(),
			holder_bob.clone(),
			None,
			Some(9), // some expired timestamp (s)
			true,
		)
	);

	let mut t = sp_io::TestExternalities::new(t);
	t.execute_with(|| {
		let xt = CheckedExtrinsic {
			signed: Some((
				holder_bob.clone(),
				signed_extra(0, 0, Some(doughnut)),
			)),
			function: Call::Balances(BalancesCall::transfer(receiver_charlie.clone().into(), 69)),
		};
		let uxt = sign_extrinsic(xt);

		Executive::initialize_block(&Header::new(
			1,
			H256::default(),
			H256::default(),
			[69u8; 32].into(),
			Digest::default(),
		));

		assert_eq!(
			Executive::apply_extrinsic(uxt),
			Err(transaction_error_from_code(error_code::VALIDATION_EXPIRED))
		);
	});
}

#[test]
fn delegated_dispatch_fails_when_doughnut_is_premature() {
	let issuer_alice: AccountId = AccountKeyring::Alice.into();
	let holder_bob: AccountId = AccountKeyring::Bob.into();
	let receiver_charlie: AccountId = AccountKeyring::Charlie.into();

	let mut t = frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap();
	pallet_balances::GenesisConfig::<Runtime> {
		balances: vec![(issuer_alice.clone(), 10_011), (holder_bob.clone(), 10_011)],
	}.assimilate_storage(&mut t).unwrap();

	let doughnut = PlugDoughnut::<Runtime>::new(
		make_doughnut(
			issuer_alice.clone(),
			holder_bob.clone(),
			Some(11), // Some future timestamp (s)
			None,
			true,
		)
	);

	let mut t = sp_io::TestExternalities::new(t);
	t.execute_with(|| {
		let xt = CheckedExtrinsic {
			signed: Some((
				holder_bob.clone(),
				signed_extra(0, 0, Some(doughnut)),
			)),
			function: Call::Balances(BalancesCall::transfer(receiver_charlie.clone().into(), 69)),
		};
		let uxt = sign_extrinsic(xt);
		Executive::initialize_block(&Header::new(
			1,
			H256::default(),
			H256::default(),
			[69u8; 32].into(),
			Digest::default(),
		));

		assert_eq!(
			Executive::apply_extrinsic(uxt),
			Err(transaction_error_from_code(error_code::VALIDATION_PREMATURE))
		);
	});
}

#[test]
fn delegated_dispatch_fails_when_doughnut_domain_permission_is_unverified() {
	let issuer_alice: AccountId = AccountKeyring::Alice.into();
	let holder_bob: AccountId = AccountKeyring::Bob.into();
	let receiver_charlie: AccountId = AccountKeyring::Charlie.into();

	let mut t = frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap();
	pallet_balances::GenesisConfig::<Runtime> {
		balances: vec![(issuer_alice.clone(), 10_011), (holder_bob.clone(), 10_011)],
	}.assimilate_storage(&mut t).unwrap();

	let doughnut = PlugDoughnut::<Runtime>::new(
		make_doughnut(
			issuer_alice.clone(),
			holder_bob.clone(),
			None,
			None,
			false,
		)
	);

	let mut t = sp_io::TestExternalities::new(t);
	t.execute_with(|| {
		let xt = CheckedExtrinsic {
			signed: Some((
				holder_bob.clone(),
				signed_extra(0, 0, Some(doughnut)),
			)),
			function: Call::Balances(BalancesCall::transfer(receiver_charlie.clone().into(), 69)),
		};
		let uxt = sign_extrinsic(xt);
		Executive::initialize_block(&Header::new(
			1,
			H256::default(),
			H256::default(),
			[69u8; 32].into(),
			Digest::default(),
		));
		let r = Executive::apply_extrinsic(uxt);
		assert_eq!(r, Ok(Err(DispatchError::Other("dispatch unverified"))));
	});
}

#[test]
fn delegated_dispatch_fails_with_bad_argument() {
	let issuer_alice: AccountId = AccountKeyring::Alice.into();
	let holder_bob: AccountId = AccountKeyring::Bob.into();
	let receiver_charlie: AccountId = AccountKeyring::Charlie.into();

	let mut t = frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap();
	pallet_balances::GenesisConfig::<Runtime> {
		balances: vec![(issuer_alice.clone(), 0x100_0000_0000), (holder_bob.clone(), 0x100_0000_0000)],
	}.assimilate_storage(&mut t).unwrap();

	let doughnut = PlugDoughnut::<Runtime>::new(
		make_doughnut(
			issuer_alice.clone(),
			holder_bob.clone(),
			None,
			None,
			true,
		)
	);

	let this_argument_will_fail = 0x12_3456_7890;
	let mut t = sp_io::TestExternalities::new(t);
	t.execute_with(|| {
		let xt = CheckedExtrinsic {
			signed: Some((
				holder_bob.clone(),
				signed_extra(0, 0, Some(doughnut)),
			)),
			function: Call::Balances(BalancesCall::transfer(receiver_charlie.clone().into(), this_argument_will_fail)),
		};
		let uxt = sign_extrinsic(xt);
		Executive::initialize_block(&Header::new(
			1,
			H256::default(),
			H256::default(),
			[69u8; 32].into(),
			Digest::default(),
		));
		let r = Executive::apply_extrinsic(uxt);
		assert_eq!(r, Ok(Err(DispatchError::Other("dispatch unverified"))));
	});
}
