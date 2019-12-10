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
use balances::Call as BalancesCall;
use codec::Encode;
use keyring::AccountKeyring;
use primitives::{crypto::UncheckedFrom, H256};
use prml_doughnut::{DoughnutRuntime, PlugDoughnut};
use sr_primitives::{
	DispatchError, DoughnutV0, MultiSignature,
	generic::{self, Era}, Perbill, testing::{Block, Digest, Header},
	traits::{IdentifyAccount, IdentityLookup, Header as HeaderT, BlakeTwo256, Verify, ConvertInto, DoughnutApi},
	transaction_validity::{InvalidTransaction, TransactionValidity, TransactionValidityError, UnknownTransaction},
};
#[allow(deprecated)]
use sr_primitives::traits::ValidateUnsigned;
use support::{
	impl_outer_event, impl_outer_origin, parameter_types, impl_outer_dispatch,
	additional_traits::{DelegatedDispatchVerifier},
	traits::{Currency, Time},
};
type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
type Address = AccountId;
type Index = u32;
type Signature = MultiSignature;
type System = system::Module<Runtime>;
type Balances = balances::Module<Runtime>;

impl_outer_origin! {
	pub enum Origin for Runtime { }
}

impl_outer_event!{
	pub enum MetaEvent for Runtime {
		balances<T>,
	}
}
impl_outer_dispatch! {
	pub enum Call for Runtime where origin: Origin {
		system::System,
		balances::Balances,
	}
}

pub struct MockDelegatedDispatchVerifier<T: system::Trait>(rstd::marker::PhantomData<T>);
impl<T: system::Trait> DelegatedDispatchVerifier<T::Doughnut> for MockDelegatedDispatchVerifier<T> {
	const DOMAIN: &'static str = "test";
	fn verify_dispatch(
		doughnut: &T::Doughnut,
		_module: &str,
		_method: &str,
	) -> Result<(), &'static str> {
		// Check the "test" domain has a byte set to `1` for Ok, fail otherwise
		let verify = doughnut.get_domain(Self::DOMAIN).unwrap()[0];
		if verify == 1 {
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
impl system::Trait for Runtime {
	type Origin = Origin;
	type Index = Index;
	type Call = Call;
	type BlockNumber = u64;
	type Hash = primitives::H256;
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
	type Doughnut = PlugDoughnut<DoughnutV0, Runtime>;
	type DelegatedDispatchVerifier = MockDelegatedDispatchVerifier<Runtime>;
}
pub struct TimestampProvider;
impl Time for TimestampProvider {
	type Moment = u64;
	fn now() -> Self::Moment {
		// Return a fixed timestamp
		// It should be > 0 to allow doughnut timestamp validation checks to pass
		123
	}
}
impl DoughnutRuntime for Runtime {
	type AccountId = <Self as system::Trait>::AccountId;
	type Call = <Self as system::Trait>::Call;
	type Doughnut = <Self as system::Trait>::Doughnut;
	type TimestampProvider = TimestampProvider;
}
parameter_types! {
	pub const ExistentialDeposit: u64 = 0;
	pub const TransferFee: u64 = 0;
	pub const CreationFee: u64 = 0;
}
impl balances::Trait for Runtime {
	type Balance = u64;
	type OnFreeBalanceZero = ();
	type OnNewAccount = ();
	type Event = MetaEvent;
	type DustRemoval = ();
	type TransferPayment = ();
	type ExistentialDeposit = ExistentialDeposit;
	type TransferFee = TransferFee;
	type CreationFee = CreationFee;
}

parameter_types! {
	pub const TransactionBaseFee: u64 = 10;
	pub const TransactionByteFee: u64 = 0;
}
impl transaction_payment::Trait for Runtime {
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

	fn validate_unsigned(call: &Self::Call) -> TransactionValidity {
		match call {
			Call::Balances(BalancesCall::set_balance(_, _, _)) => Ok(Default::default()),
			_ => UnknownTransaction::NoUnsignedValidator.into(),
		}
	}
}
type SignedExtra = (
	Option<PlugDoughnut<DoughnutV0, Runtime>>,
	system::CheckVersion<Runtime>,
	system::CheckGenesis<Runtime>,
	system::CheckEra<Runtime>,
	system::CheckNonce<Runtime>,
	system::CheckWeight<Runtime>,
	transaction_payment::ChargeTransactionPayment<Runtime>
);

type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>; // Just a `CheckedExtrinsic` with type parameters set
type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>; // Just an `UnheckedExtrinsic` with type parameters set
type Executive = frame_executive::Executive<Runtime, Block<UncheckedExtrinsic>, system::ChainContext<Runtime>, Runtime, ()>;

/// Returns transaction extra.
fn signed_extra(nonce: Index, fee: u64, doughnut: Option<PlugDoughnut<DoughnutV0, Runtime>>) -> SignedExtra {
	(
		doughnut,
		system::CheckVersion::new(),
		system::CheckGenesis::new(),
		system::CheckEra::from(Era::mortal(256, 0)),
		system::CheckNonce::from(nonce),
		system::CheckWeight::new(),
		transaction_payment::ChargeTransactionPayment::from(fee),
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
					key.sign(&runtime_io::hashing::blake2_256(payload))
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
fn make_doughnut(issuer: AccountId, holder: AccountId, not_before: Option<u32>, expiry: Option<u32>, permission_domain_verify: bool) -> DoughnutV0 {
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
	doughnut.signature = issuer_key.sign(&doughnut.payload()).into();
	doughnut
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
	let mut t = system::GenesisConfig::default().build_storage::<Runtime>().unwrap();
	balances::GenesisConfig::<Runtime> {
		balances: vec![(issuer_alice.clone(), 10_011), (holder_bob.clone(), 10_011)],
		vesting: vec![],
	}.assimilate_storage(&mut t).unwrap();

	// The doughnut proof is wrapped for embeddeding in extrinsic
	let doughnut = PlugDoughnut::<DoughnutV0, Runtime>::new(
		make_doughnut(
			issuer_alice.clone(),
			holder_bob.clone(),
			None,
			None,
			true,
		)
	);

	let mut t = runtime_io::TestExternalities::new(t);
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
		assert_eq!(<balances::Module<Runtime>>::total_balance(&issuer_alice), 10_011 - 69); // 69 transferred
		assert_eq!(<balances::Module<Runtime>>::total_balance(&holder_bob), 10_011 - 10 - 1024); // fees deducted
		assert_eq!(<balances::Module<Runtime>>::total_balance(&receiver_charlie), 69); // Received 69
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
	let mut t = system::GenesisConfig::default().build_storage::<Runtime>().unwrap();
	balances::GenesisConfig::<Runtime> {
		balances: vec![(issuer_alice.clone(), 10_011), (holder_bob.clone(), 10_011)],
		vesting: vec![],
	}.assimilate_storage(&mut t).unwrap();

	let doughnut = PlugDoughnut::<DoughnutV0, Runtime>::new(
		make_doughnut(
			issuer_alice.clone(),
			holder_bob.clone(),
			None,
			None,
			true,
		)
	);

	let mut t = runtime_io::TestExternalities::new(t);
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

		let r = Executive::apply_extrinsic(uxt);
		assert_eq!(r, Err(TransactionValidityError::Invalid(InvalidTransaction::Custom(171))));
	});
}

#[test]
fn delegated_dispatch_fails_when_doughnut_is_expired() {
	let issuer_alice: AccountId = AccountKeyring::Alice.into();
	let holder_bob: AccountId = AccountKeyring::Bob.into();
	let receiver_charlie: AccountId = AccountKeyring::Charlie.into();

	let mut t = system::GenesisConfig::default().build_storage::<Runtime>().unwrap();
	balances::GenesisConfig::<Runtime> {
		balances: vec![(issuer_alice.clone(), 10_011), (holder_bob.clone(), 10_011)],
		vesting: vec![],
	}.assimilate_storage(&mut t).unwrap();

	let doughnut = PlugDoughnut::<DoughnutV0, Runtime>::new(
		make_doughnut(
			issuer_alice.clone(),
			holder_bob.clone(),
			None,
			Some(100), // some expired timestamp
			true,
		)
	);

	let mut t = runtime_io::TestExternalities::new(t);
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
		assert_eq!(r, Err(TransactionValidityError::Invalid(InvalidTransaction::Custom(171))));
	});
}

#[test]
fn delegated_dispatch_fails_when_doughnut_is_premature() {
	let issuer_alice: AccountId = AccountKeyring::Alice.into();
	let holder_bob: AccountId = AccountKeyring::Bob.into();
	let receiver_charlie: AccountId = AccountKeyring::Charlie.into();

	let mut t = system::GenesisConfig::default().build_storage::<Runtime>().unwrap();
	balances::GenesisConfig::<Runtime> {
		balances: vec![(issuer_alice.clone(), 10_011), (holder_bob.clone(), 10_011)],
		vesting: vec![],
	}.assimilate_storage(&mut t).unwrap();

	let doughnut = PlugDoughnut::<DoughnutV0, Runtime>::new(
		make_doughnut(
			issuer_alice.clone(),
			holder_bob.clone(),
			Some(1_000), // Some future timestamp
			None,
			true,
		)
	);

	let mut t = runtime_io::TestExternalities::new(t);
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
		assert_eq!(r, Err(TransactionValidityError::Invalid(InvalidTransaction::Custom(171))));
	});
}

#[test]
fn delegated_dispatch_fails_when_doughnut_domain_permission_is_unverified() {
	let issuer_alice: AccountId = AccountKeyring::Alice.into();
	let holder_bob: AccountId = AccountKeyring::Bob.into();
	let receiver_charlie: AccountId = AccountKeyring::Charlie.into();

	let mut t = system::GenesisConfig::default().build_storage::<Runtime>().unwrap();
	balances::GenesisConfig::<Runtime> {
		balances: vec![(issuer_alice.clone(), 10_011), (holder_bob.clone(), 10_011)],
		vesting: vec![],
	}.assimilate_storage(&mut t).unwrap();

	let doughnut = PlugDoughnut::<DoughnutV0, Runtime>::new(
		make_doughnut(
			issuer_alice.clone(),
			holder_bob.clone(),
			None,
			None,
			false,
		)
	);

	let mut t = runtime_io::TestExternalities::new(t);
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
		assert_eq!(r, Ok(Err(DispatchError { module: Some(1), error: 0, message: Some("dispatch unverified") })));
	});
}