// Copyright (C) 2019 Centrality Investments Limited
// This file is part of PLUG.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

//!
//! A doughnut enabled executive impl
//!
use rstd::prelude::*;
use rstd::marker::PhantomData;
use rstd::result;
use sr_primitives::traits::{
	self, Applyable, Checkable, Doughnuted, Header,
	OffchainWorker, OnFinalize, OnInitialize,
	ValidateUnsigned, DoughnutApi,
};
use sr_primitives::{ApplyOutcome, ApplyError};
use sr_primitives::transaction_validity::TransactionValidity;
use sr_primitives::weights::GetDispatchInfo;
use srml_support::{Dispatchable, storage, additional_traits::ChargeExtrinsicFee};
use parity_codec::{Codec, Encode};

use system::DigestOf;
use substrate_primitives::storage::well_known_keys;

use crate::{Executive, CheckedOf, CallOf, OriginOf};

mod internal {
	use sr_primitives::traits::DispatchError;

	#[cfg_attr(feature = "std", derive(Debug))]
	pub enum ApplyError {
		BadSignature(&'static str),
		Stale,
		Future,
		CantPay,
		FullBlock,
		SignerHolderMismatch,
	}

	pub enum ApplyOutcome {
		Success,
		Fail(&'static str),
	}

	impl From<DispatchError> for ApplyError {
		fn from(d: DispatchError) -> Self {
			match d {
				DispatchError::Payment => ApplyError::CantPay,
				DispatchError::Resource => ApplyError::FullBlock,
				DispatchError::NoPermission => ApplyError::CantPay,
				DispatchError::BadState => ApplyError::CantPay,
				DispatchError::Stale => ApplyError::Stale,
				DispatchError::Future => ApplyError::Future,
				DispatchError::BadProof => ApplyError::BadSignature(""),
			}
		}
	}
}

/// A Doughnut aware executive type
/// It proxies to the standard Executive implementation for the most part
pub struct DoughnutExecutive<System, Block, Context, Payment, UnsignedValidator, AllModules>(
	PhantomData<(System, Block, Context, Payment, UnsignedValidator, AllModules)>
);

impl<
	System: system::Trait,
	Block: traits::Block<Header=System::Header, Hash=System::Hash>,
	Context: Default,
	Payment: ChargeExtrinsicFee<System::AccountId, <Block::Extrinsic as Checkable<Context>>::Checked>,
	UnsignedValidator,
	AllModules: OnInitialize<System::BlockNumber> + OnFinalize<System::BlockNumber> + OffchainWorker<System::BlockNumber>,
> DoughnutExecutive<System, Block, Context, Payment, UnsignedValidator, AllModules>
where
	Block::Extrinsic: Checkable<Context> + Codec,
	CheckedOf<Block::Extrinsic, Context>: Applyable<AccountId=System::AccountId> + Doughnuted + GetDispatchInfo,
	CallOf<Block::Extrinsic, Context>: Dispatchable,
	OriginOf<Block::Extrinsic, Context>: From<Option<System::AccountId>>,
	UnsignedValidator: ValidateUnsigned<Call=<<Block::Extrinsic as Checkable<Context>>::Checked as Applyable>::Call>,
	<CheckedOf<Block::Extrinsic, Context> as Applyable>::AccountId: AsRef<[u8]> + Sized,
	<<CheckedOf<Block::Extrinsic, Context> as Doughnuted>::Doughnut as DoughnutApi>::AccountId: AsRef<[u8]> + Sized,
	<<CheckedOf<Block::Extrinsic, Context> as Doughnuted>::Doughnut as DoughnutApi>::Signature: AsRef<[u8]> + Sized,
{
	/// Start the execution of a particular block.
	pub fn initialize_block(header: &System::Header) {
		let mut digests = <DigestOf<System>>::default();
		header.digest().logs().iter().for_each(|d| if d.as_pre_runtime().is_some() { digests.push(d.clone()) });
		Executive::<System, Block, Context, UnsignedValidator, AllModules>::initialize_block_impl(header.number(), header.parent_hash(), header.extrinsics_root(), &digests);
	}

	/// Actually execute all transitions for `block`.
	pub fn execute_block(block: Block) {
		Executive::<System, Block, Context, UnsignedValidator, AllModules>::initialize_block(block.header());

		// any initial checks
		Executive::<System, Block, Context, UnsignedValidator, AllModules>::initial_checks(&block);

		// execute extrinsics
		let (header, extrinsics) = block.deconstruct();
		extrinsics.into_iter().for_each(Self::apply_extrinsic_no_note);

		// post-extrinsics book-keeping
		<system::Module<System>>::note_finished_extrinsics();
		<AllModules as OnFinalize<System::BlockNumber>>::on_finalize(*header.number());

		// any final checks
		Executive::<System, Block, Context, UnsignedValidator, AllModules>::final_checks(&header);
	}

	/// Finalize the block - it is up the caller to ensure that all header fields are valid
	/// except state-root.
	pub fn finalize_block() -> System::Header {
		Executive::<System, Block, Context, UnsignedValidator, AllModules>::finalize_block()
	}

	/// Apply extrinsic outside of the block execution function.
	/// This doesn't attempt to validate anything regarding the block, but it builds a list of uxt
	/// hashes.
	pub fn apply_extrinsic(uxt: Block::Extrinsic) -> result::Result<ApplyOutcome, ApplyError> {
		let encoded = uxt.encode();
		let encoded_len = encoded.len();
		match Self::apply_extrinsic_with_len(uxt, encoded_len, Some(encoded)) {
			Ok(internal::ApplyOutcome::Success) => Ok(ApplyOutcome::Success),
			Ok(internal::ApplyOutcome::Fail(_)) => Ok(ApplyOutcome::Fail),
			Err(internal::ApplyError::CantPay) => Err(ApplyError::CantPay),
			Err(internal::ApplyError::BadSignature(_)) => Err(ApplyError::BadSignature),
			Err(internal::ApplyError::Stale) => Err(ApplyError::Stale),
			Err(internal::ApplyError::Future) => Err(ApplyError::Future),
			Err(internal::ApplyError::FullBlock) => Err(ApplyError::FullBlock),
			Err(internal::ApplyError::SignerHolderMismatch) => Err(ApplyError::SignerHolderMismatch),
		}
	}

	/// Apply an extrinsic inside the block execution function.
	fn apply_extrinsic_no_note(uxt: Block::Extrinsic) {
		let l = uxt.encode().len();
		match Self::apply_extrinsic_with_len(uxt, l, None) {
			Ok(internal::ApplyOutcome::Success) => (),
			Ok(internal::ApplyOutcome::Fail(e)) => runtime_io::print(e),
			Err(internal::ApplyError::CantPay) => panic!("All extrinsics should have sender able to pay their fees"),
			Err(internal::ApplyError::BadSignature(_)) => panic!("All extrinsics should be properly signed"),
			Err(internal::ApplyError::Stale) | Err(internal::ApplyError::Future) => panic!("All extrinsics should have the correct nonce"),
			Err(internal::ApplyError::FullBlock) => panic!("Extrinsics should not exceed block limit"),
			Err(internal::ApplyError::SignerHolderMismatch) => panic!("Attached doughnut should have the same signer and holder"),
		}
	}

	/// Actually apply an extrinsic given its `encoded_len`; this doesn't note its hash.
	fn apply_extrinsic_with_len(uxt: Block::Extrinsic, encoded_len: usize, to_note: Option<Vec<u8>>) -> result::Result<internal::ApplyOutcome, internal::ApplyError> {

		// Verify that the signature is good.
		let xt = uxt.check(&Default::default()).map_err(internal::ApplyError::BadSignature)?;

		// We don't need to make sure to `note_extrinsic` only after we know it's going to be
		// executed to prevent it from leaking in storage since at this point, it will either
		// execute or panic (and revert storage changes).
		if let Some(encoded) = to_note {
			<system::Module<System>>::note_extrinsic(encoded);
		}

		if let Some(doughnut) = xt.doughnut() {
			// This extrinsic has a doughnut. Store it so that the doughnut is accessible
			// by the runtime during execution
			storage::unhashed::put(well_known_keys::DOUGHNUT_KEY, &doughnut);
		} else {
			// Ensure doughnut state is empty
			storage::unhashed::kill(well_known_keys::DOUGHNUT_KEY);
		}

		// Decode parameters and dispatch
		// TODO: Doughnut needs to use issuer as origin
		let dispatch_info = xt.get_dispatch_info();
		let result = Applyable::dispatch(xt, dispatch_info, encoded_len).map_err(internal::ApplyError::from)?;

		<system::Module<System>>::note_applied_extrinsic(&result, encoded_len as u32);

		result.map(|_| internal::ApplyOutcome::Success).or_else(|e| match e {
			sr_primitives::BLOCK_FULL => Err(internal::ApplyError::FullBlock),
			e => Ok(internal::ApplyOutcome::Fail(e))
		})
	}

	pub fn validate_transaction(uxt: Block::Extrinsic) -> TransactionValidity {
		Executive::<System, Block, Context, UnsignedValidator, AllModules>::validate_transaction(uxt)
	}

	pub fn offchain_worker(n: System::BlockNumber) {
		Executive::<System, Block, Context, UnsignedValidator, AllModules>::offchain_worker(n)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use balances::Call;
	use runtime_io::with_externalities;
	use substrate_primitives::{H256, Blake2Hasher};
	use primitives::ApplyError;
	use primitives::traits::{Header as HeaderT, BlakeTwo256, IdentityLookup};
	use primitives::testing::{Block, Digest, Header};
	use primitives::testing::doughnut::{DummyDoughnut, TestAccountId, TestXt as DoughnutedTestXt};
	use srml_support::{additional_traits::DispatchVerifier, assert_err, traits::Currency, impl_outer_origin, impl_outer_event, parameter_types};
	use system;
	use hex_literal::hex;

	impl_outer_origin! {
		pub enum Origin for Runtime {
		}
	}

	impl_outer_event!{
		pub enum MetaEvent for Runtime {
			balances<T>,
		}
	}

	/// A no-op verifier
	pub struct DummyDispatchVerifier;

	impl DispatchVerifier<DummyDoughnut> for DummyDispatchVerifier {
		const DOMAIN: &'static str = "test";
		fn verify(
			_doughnut: &DummyDoughnut,
			_module: &str,
			_method: &str,
		) -> Result<(), &'static str> {
			Ok(())
		}
	}

	// Workaround for https://github.com/rust-lang/rust/issues/26925 . Remove when sorted.
	#[derive(Clone, Eq, PartialEq)]
	pub struct Runtime;
	parameter_types! {
		pub const BlockHashCount: u64 = 250;
		pub const ExistentialDeposit: u64 = 0;
		pub const TransferFee: u64 = 0;
		pub const CreationFee: u64 = 0;
		pub const TransactionBaseFee: u64 = 10;
		pub const TransactionByteFee: u64 = 1;
	}
	impl system::Trait for Runtime {
		type Origin = Origin;
		type Index = u64;
		type BlockNumber = u64;
		type Hash = substrate_primitives::H256;
		type Hashing = BlakeTwo256;
		type AccountId = TestAccountId;
		type Lookup = IdentityLookup<TestAccountId>;
		type Header = Header;
		type Event = MetaEvent;
		type BlockHashCount = BlockHashCount;
		type Doughnut = DummyDoughnut;
		type DispatchVerifier = DummyDispatchVerifier;
	}
	impl balances::Trait for Runtime {
		type Balance = u64;
		type OnFreeBalanceZero = ();
		type OnNewAccount = ();
		type Event = MetaEvent;
		type TransactionPayment = ();
		type DustRemoval = ();
		type TransferPayment = ();
		type ExistentialDeposit = ExistentialDeposit;
		type TransferFee = TransferFee;
		type CreationFee = CreationFee;
		type TransactionBaseFee = TransactionBaseFee;
		type TransactionByteFee = TransactionByteFee;
	}

	impl ValidateUnsigned for Runtime {
		type Call = Call<Runtime>;

		fn validate_unsigned(call: &Self::Call) -> TransactionValidity {
			match call {
				Call::set_balance(_, _, _) => TransactionValidity::Valid {
					priority: 0,
					requires: vec![],
					provides: vec![],
					longevity: std::u64::MAX,
					propagate: false,
				},
				_ => TransactionValidity::Invalid(0),
			}
		}
	}

	type TestXt = DoughnutedTestXt<Call<Runtime>, DummyDoughnut>;
	type Executive = super::DoughnutExecutive<
		Runtime,
		Block<TestXt>,
		system::ChainContext<Runtime>,
		balances::Module<Runtime>,
		Runtime,
		()
	>;

	#[test]
	fn balance_transfer_dispatch_works() {
		let mut t = system::GenesisConfig::default().build_storage::<Runtime>().unwrap();
		balances::GenesisConfig::<Runtime> {
			balances: vec![(TestAccountId::new(1), 234)],
			vesting: vec![],
		}.assimilate_storage(&mut t.0, &mut t.1).unwrap();
		let xt = DoughnutedTestXt::new(Some(1), 0, Call::transfer(TestAccountId::new(2), 69), None);
		let mut t = runtime_io::TestExternalities::<Blake2Hasher>::new_with_children(t);
		with_externalities(&mut t, || {
			Executive::initialize_block(&Header::new(
				1,
				H256::default(),
				H256::default(),
				[69u8; 32].into(),
				Digest::default(),
			));
			Executive::apply_extrinsic(xt).unwrap();
			assert_eq!(<balances::Module<Runtime>>::total_balance(&TestAccountId::new(1)), 126);
			assert_eq!(<balances::Module<Runtime>>::total_balance(&TestAccountId::new(2)), 69);
		});
	}

	fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
		let mut t = system::GenesisConfig::default().build_storage::<Runtime>().unwrap().0;
		t.extend(balances::GenesisConfig::<Runtime> {
			balances: vec![(TestAccountId::new(1), 1111), (TestAccountId::new(2), 2222)],
			vesting: vec![],
		}.build_storage().unwrap().0);
		t.into()
	}

	#[test]
	fn block_import_works() {
		with_externalities(&mut new_test_ext(), || {
			Executive::execute_block(Block {
				header: Header {
					parent_hash: [69u8; 32].into(),
					number: 1,
					state_root: hex!("5439e5e595d8ddad50dc2674a9d7b20775ed498f53bff6e73f1fd7704b23a887").into(),
					extrinsics_root: hex!("03170a2e7597b7b7e3d84c05391d139a62b157e78786d8c082f29dcf4c111314").into(),
					digest: Digest { logs: vec![], },
				},
				extrinsics: vec![],
			});
		});
	}

	#[test]
	fn block_import_works_with_doughnut() {
		let doughnut = DummyDoughnut {
			issuer: TestAccountId::new(1),
			holder: TestAccountId::new(2),
		};
		with_externalities(&mut new_test_ext(), || {
			Executive::execute_block(Block {
				header: Header {
					parent_hash: [69u8; 32].into(),
					number: 1,
					state_root: hex!("e490996846ed723d9ab6910c9b2534c3924c76dd85072317eb40ad354ad3b6c7").into(),
					extrinsics_root: hex!("0015f7b954b12470f63106b1a70b4f6634ad5261f5c434c7990e47325943fd21").into(),
					digest: Digest { logs: vec![], },
				},
				extrinsics: vec![
					DoughnutedTestXt::new(Some(2), 0, Call::transfer(TestAccountId::new(1), 50), Some(doughnut))
				],
			});
		});
	}

	#[test]
	#[should_panic]
	fn block_import_of_bad_state_root_fails() {
		with_externalities(&mut new_test_ext(), || {
			Executive::execute_block(Block {
				header: Header {
					parent_hash: [69u8; 32].into(),
					number: 1,
					state_root: [0u8; 32].into(),
					extrinsics_root: hex!("03170a2e7597b7b7e3d84c05391d139a62b157e78786d8c082f29dcf4c111314").into(),
					digest: Digest { logs: vec![], },
				},
				extrinsics: vec![],
			});
		});
	}

	#[test]
	#[should_panic]
	fn block_import_of_bad_extrinsic_root_fails() {
		with_externalities(&mut new_test_ext(), || {
			Executive::execute_block(Block {
				header: Header {
					parent_hash: [69u8; 32].into(),
					number: 1,
					state_root: hex!("49cd58a254ccf6abc4a023d9a22dcfc421e385527a250faec69f8ad0d8ed3e48").into(),
					extrinsics_root: [0u8; 32].into(),
					digest: Digest { logs: vec![], },
				},
				extrinsics: vec![],
			});
		});
	}

	#[test]
	fn bad_extrinsic_not_inserted() {
		let mut t = new_test_ext();
		let xt = DoughnutedTestXt::new(Some(1), 42, Call::transfer(TestAccountId::new(33), 69), None);
		with_externalities(&mut t, || {
			Executive::initialize_block(&Header::new(
				1,
				H256::default(),
				H256::default(),
				[69u8; 32].into(),
				Digest::default(),
			));
			assert!(Executive::apply_extrinsic(xt).is_err());
			assert_eq!(<system::Module<Runtime>>::extrinsic_index(), Some(0));
		});
	}

	#[test]
	fn block_weight_limit_enforced() {
		let run_test = |should_fail: bool| {
			let mut t = new_test_ext();
			let xt = DoughnutedTestXt::new(Some(1), 0, Call::transfer(TestAccountId::new(33), 69), None);
			let xt2 = DoughnutedTestXt::new(Some(1), 1, Call::transfer(TestAccountId::new(33), 69), None);
			let encoded = xt2.encode();
			let len = if should_fail { (internal::MAX_TRANSACTIONS_WEIGHT - 1) as usize } else { encoded.len() };
			with_externalities(&mut t, || {
				Executive::initialize_block(&Header::new(
					1,
					H256::default(),
					H256::default(),
					[69u8; 32].into(),
					Digest::default(),
				));
				assert_eq!(<system::Module<Runtime>>::all_extrinsics_weight(), 0);

				Executive::apply_extrinsic(xt).unwrap();
				let res = Executive::apply_extrinsic_with_len(xt2, len, Some(encoded));

				if should_fail {
					assert!(res.is_err());
					assert_eq!(<system::Module<Runtime>>::all_extrinsics_weight(), 29);
					assert_eq!(<system::Module<Runtime>>::extrinsic_index(), Some(1));
				} else {
					assert!(res.is_ok());
					assert_eq!(<system::Module<Runtime>>::all_extrinsics_weight(), 58);
					assert_eq!(<system::Module<Runtime>>::extrinsic_index(), Some(2));
				}
			});
		};

		run_test(false);
		run_test(true);
	}

	#[test]
	fn default_block_weight() {
		let xt = DoughnutedTestXt::new(None, 0, Call::set_balance(TestAccountId::new(33), 69, 69), None);
		let mut t = new_test_ext();
		with_externalities(&mut t, || {
			Executive::apply_extrinsic(xt.clone()).unwrap();
			Executive::apply_extrinsic(xt.clone()).unwrap();
			Executive::apply_extrinsic(xt.clone()).unwrap();
			assert_eq!(
				<system::Module<Runtime>>::all_extrinsics_weight(),
				3 * (10 /*base*/ + 13 /*len*/ * 1 /*byte*/)
			);
		});
	}

	#[test]
	fn balance_transfer_dispatch_works_with_doughnut() {
		let mut t = system::GenesisConfig::default().build_storage::<Runtime>().unwrap();
		// The doughnut is not semantically verified in this runtime.
		// This just checks the execution flow up to dispatch path with a valid doughnut
		let alice = TestAccountId::new(1);
		let bob = TestAccountId::new(2);
		let charlie = TestAccountId::new(3);
		balances::GenesisConfig::<Runtime> {
			balances: vec![
				(alice.clone(), 1000),
				(bob.clone(), 500),
				(charlie.clone(), 0),
			],
			vesting: vec![],
		}.assimilate_storage(&mut t.0, &mut t.1).unwrap();

		let doughnut = DummyDoughnut {
			issuer: alice.clone(),
			holder: bob.clone(),
		};
		// Bob signs a tx to send 30 of alice's balance to charlie
		let xt = DoughnutedTestXt::new(Some(2), 0, Call::transfer(charlie.clone(), 30), Some(doughnut));
		let mut t = runtime_io::TestExternalities::<Blake2Hasher>::new_with_children(t);
		with_externalities(&mut t, || {
			Executive::initialize_block(&Header::new(1, H256::default(), H256::default(),
				[69u8; 32].into(), Digest::default()));
			Executive::apply_extrinsic(xt).unwrap();
			assert_eq!(<balances::Module<Runtime>>::total_balance(&alice), 970);
			assert_eq!(<balances::Module<Runtime>>::total_balance(&bob), 446);
			assert_eq!(<balances::Module<Runtime>>::total_balance(&charlie), 30);
		});
	}

	#[test]
	fn it_fails_when_xt_sender_and_doughnut_holder_are_mismatched() {
		let alice = TestAccountId::new(1);
		let bob = TestAccountId::new(2);
		let doughnut = DummyDoughnut {
			issuer: alice.clone(),
			holder: bob.clone(),
		};
		// signer id `3`/charlie != holder id `2`/bob
		let xt = DoughnutedTestXt::new(Some(3), 0, Call::transfer(bob.clone(), 30), Some(doughnut.clone()));

		let mut t = new_test_ext();
		with_externalities(&mut t, || {
			Executive::initialize_block(&Header::new(1, H256::default(), H256::default(), [69u8; 32].into(), Digest::default()));
			assert_err!(Executive::apply_extrinsic(xt), ApplyError::SignerHolderMismatch);
		});
	}
}
