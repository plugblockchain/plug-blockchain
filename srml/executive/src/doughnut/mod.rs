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
use primitives::traits::{
	self, Applyable, Checkable, Doughnuted, Header,
	OffchainWorker, OnFinalize, OnInitialize,
};
use srml_support::{Dispatchable, storage, additional_traits::ChargeExtrinsicFee};
use parity_codec::{Codec, Decode, Encode};
use primitives::{ApplyOutcome, ApplyError};
use primitives::traits::DoughnutApi;
use primitives::transaction_validity::TransactionValidity;
use substrate_primitives::storage::well_known_keys;

use crate::Executive;

mod internal {
	pub const MAX_TRANSACTIONS_SIZE: u32 = 4 * 1024 * 1024;
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
}

/// A Doughnut aware executive type
/// It proxies to the standard Executive implementation for the most part
pub struct DoughnutExecutive<System, Block, Context, Payment, AllModules>(
	PhantomData<(System, Block, Context, Payment, AllModules)>
);

impl<
	System: system::Trait,
	Block: traits::Block<Header=System::Header, Hash=System::Hash>,
	Context: Default,
	Payment: ChargeExtrinsicFee<System::AccountId, <Block::Extrinsic as Checkable<Context>>::Checked>,
	AllModules: OnInitialize<System::BlockNumber> + OnFinalize<System::BlockNumber> + OffchainWorker<System::BlockNumber>,
> DoughnutExecutive<System, Block, Context, Payment, AllModules>
where
	Block::Extrinsic: Checkable<Context> + Codec,
	<Block::Extrinsic as Checkable<Context>>::Checked: Applyable<Index=System::Index, AccountId=System::AccountId> + Doughnuted,
	<<Block::Extrinsic as Checkable<Context>>::Checked as Applyable>::Call: Dispatchable,
	<<Block::Extrinsic as Checkable<Context>>::Checked as Applyable>::AccountId: AsRef<[u8]> + Sized,
	<<<Block::Extrinsic as Checkable<Context>>::Checked as Doughnuted>::Doughnut as DoughnutApi>::AccountId: AsRef<[u8]> + Sized,
	<<<Block::Extrinsic as Checkable<Context>>::Checked as Doughnuted>::Doughnut as DoughnutApi>::Signature: AsRef<[u8]> + Sized,
	<<<Block::Extrinsic as Checkable<Context>>::Checked as Applyable>::Call as Dispatchable>::Origin: From<Option<System::AccountId>>,
{
	/// Start the execution of a particular block.
	pub fn initialize_block(header: &System::Header) {
		Executive::<System, Block, Context, Payment, AllModules>::initialize_block_impl(header.number(), header.parent_hash(), header.extrinsics_root());
	}

	/// Actually execute all transitions for `block`.
	pub fn execute_block(block: Block) {
		Executive::<System, Block, Context, Payment, AllModules>::execute_block(block)
	}

	/// Finalize the block - it is up the caller to ensure that all header fields are valid
	/// except state-root.
	pub fn finalize_block() -> System::Header {
		Executive::<System, Block, Context, Payment, AllModules>::finalize_block()
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

	/// Actually apply an extrinsic given its `encoded_len`; this doesn't note its hash.
	fn apply_extrinsic_with_len(uxt: Block::Extrinsic, encoded_len: usize, to_note: Option<Vec<u8>>) -> result::Result<internal::ApplyOutcome, internal::ApplyError> {

		// Verify that the signature is good.
		let xt = uxt.check(&Default::default()).map_err(internal::ApplyError::BadSignature)?;

		// Check the size of the block if that extrinsic is applied.
		if <system::Module<System>>::all_extrinsics_len() + encoded_len as u32 > internal::MAX_TRANSACTIONS_SIZE {
			return Err(internal::ApplyError::FullBlock);
		}

		if let (Some(sender), Some(index)) = (xt.sender(), xt.index()) {

			// check extrinsic signer is doughnut holder
			if let Some(ref doughnut) = xt.doughnut() {
				if sender.as_ref() != doughnut.holder().as_ref() {
					return Err(internal::ApplyError::SignerHolderMismatch);
				};
			}

			let expected_index = <system::Module<System>>::account_nonce(sender);
			if index != &expected_index { return Err(
				if index < &expected_index { internal::ApplyError::Stale } else { internal::ApplyError::Future }
			) }

			// pay any fees
			Payment::charge_extrinsic_fee(sender, encoded_len, &xt).map_err(|_| internal::ApplyError::CantPay)?;

			// AUDIT: Under no circumstances may this function panic from here onwards.

			// increment nonce in storage
			<system::Module<System>>::inc_account_nonce(sender)
		}

		// Make sure to `note_extrinsic` only after we know it's going to be executed
		// to prevent it from leaking in storage.
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
		let result = if let Some(doughnut) = xt.doughnut() {
			// Doughnut needs to use issuer as origin
			let issuer = System::AccountId::decode(&mut doughnut.issuer().as_ref()); // TODO: from/into would be better...
			let (f, _) = xt.deconstruct();
			f.dispatch(issuer.into())
		} else {
			// Decode parameters and dispatch
			let (f, s) = xt.deconstruct();
			f.dispatch(s.into())
		};

		<system::Module<System>>::note_applied_extrinsic(&result, encoded_len as u32);

		result.map(|_| internal::ApplyOutcome::Success).or_else(|e| match e {
			primitives::BLOCK_FULL => Err(internal::ApplyError::FullBlock),
			e => Ok(internal::ApplyOutcome::Fail(e))
		})
	}

	pub fn validate_transaction(uxt: Block::Extrinsic) -> TransactionValidity {
		Executive::<System, Block, Context, Payment, AllModules>::validate_transaction(uxt)
	}

	pub fn offchain_worker(n: System::BlockNumber) {
		Executive::<System, Block, Context, Payment, AllModules>::offchain_worker(n)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use balances::Call;
	use runtime_io::with_externalities;
	use substrate_primitives::{H256, Blake2Hasher};
	use primitives::{ApplyError, BuildStorage};
	use primitives::traits::{Header as HeaderT, BlakeTwo256, IdentityLookup};
	use primitives::testing::{Block, Digest, DigestItem, Header};
	use primitives::testing::doughnut::{DummyDoughnut, TestAccountId, TestXt as DoughnutedTestXt};
	use srml_support::{additional_traits::DispatchVerifier, assert_err, traits::Currency, impl_outer_origin, impl_outer_event};
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
	impl system::Trait for Runtime {
		type Origin = Origin;
		type Index = u64;
		type BlockNumber = u64;
		type Hash = substrate_primitives::H256;
		type Hashing = BlakeTwo256;
		type Digest = Digest;
		type AccountId = TestAccountId;
		type Lookup = IdentityLookup<TestAccountId>;
		type Header = Header;
		type Event = MetaEvent;
		type Log = DigestItem;
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
	}

	type TestXt = DoughnutedTestXt<Call<Runtime>, DummyDoughnut>;
	type Executive = super::DoughnutExecutive<Runtime, Block<TestXt>, system::ChainContext<Runtime>, balances::Module<Runtime>, ()>;

	#[test]
	fn balance_transfer_dispatch_works() {
		let mut t = system::GenesisConfig::<Runtime>::default().build_storage().unwrap().0;
		t.extend(balances::GenesisConfig::<Runtime> {
			transaction_base_fee: 0,
			transaction_byte_fee: 0,
			balances: vec![(TestAccountId::new(1), 111)],
			existential_deposit: 0,
			transfer_fee: 0,
			creation_fee: 0,
			vesting: vec![],
		}.build_storage().unwrap().0);
		let xt = DoughnutedTestXt::new(Some(1), 0, Call::transfer(TestAccountId::new(2), 69), None);
		let mut t = runtime_io::TestExternalities::<Blake2Hasher>::new(t);
		with_externalities(&mut t, || {
			Executive::initialize_block(&Header::new(1, H256::default(), H256::default(),
				[69u8; 32].into(), Digest::default()));
			Executive::apply_extrinsic(xt).unwrap();
			assert_eq!(<balances::Module<Runtime>>::total_balance(&TestAccountId::new(1)), 42);
			assert_eq!(<balances::Module<Runtime>>::total_balance(&TestAccountId::new(2)), 69);
		});
	}

	fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
		let mut t = system::GenesisConfig::<Runtime>::default().build_storage().unwrap().0;
		t.extend(balances::GenesisConfig::<Runtime>::default().build_storage().unwrap().0);
		t.into()
	}

	#[test]
	fn block_import_works() {
		with_externalities(&mut new_test_ext(), || {
			Executive::execute_block(Block {
				header: Header {
					parent_hash: [69u8; 32].into(),
					number: 1,
					state_root: hex!("ac2840371d51ff2e036c8fc05af7313b7a030f735c38b2f03b94cbe87bfbb7c9").into(),
					extrinsics_root: hex!("03170a2e7597b7b7e3d84c05391d139a62b157e78786d8c082f29dcf4c111314").into(),
					digest: Digest { logs: vec![], },
				},
				extrinsics: vec![],
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
			Executive::initialize_block(&Header::new(1, H256::default(), H256::default(), [69u8; 32].into(), Digest::default()));
			assert!(Executive::apply_extrinsic(xt).is_err());
			assert_eq!(<system::Module<Runtime>>::extrinsic_index(), Some(0));
		});
	}

	#[test]
	fn block_size_limit_enforced() {
		let run_test = |should_fail: bool| {
			let mut t = new_test_ext();
			let xt = DoughnutedTestXt::new(Some(1), 0, Call::transfer(TestAccountId::new(33), 69), None);
			let xt2 = DoughnutedTestXt::new(Some(1), 1, Call::transfer(TestAccountId::new(33), 69), None);
			let encoded = xt2.encode();
			let len = if should_fail { (internal::MAX_TRANSACTIONS_SIZE - 1) as usize } else { encoded.len() };
			with_externalities(&mut t, || {
				Executive::initialize_block(&Header::new(1, H256::default(), H256::default(), [69u8; 32].into(), Digest::default()));
				assert_eq!(<system::Module<Runtime>>::all_extrinsics_len(), 0);

				Executive::apply_extrinsic(xt).unwrap();
				let res = Executive::apply_extrinsic_with_len(xt2, len, Some(encoded));

				// +1 byte for doughnut
				if should_fail {
					assert!(res.is_err());
					assert_eq!(<system::Module<Runtime>>::all_extrinsics_len(), 29);
					assert_eq!(<system::Module<Runtime>>::extrinsic_index(), Some(1));
				} else {
					assert!(res.is_ok());
					assert_eq!(<system::Module<Runtime>>::all_extrinsics_len(), 58);
					assert_eq!(<system::Module<Runtime>>::extrinsic_index(), Some(2));
				}
			});
		};

		run_test(false);
		run_test(true);
	}

	#[test]
	fn balance_transfer_dispatch_works_with_doughnut() {
		let mut t = system::GenesisConfig::<Runtime>::default().build_storage().unwrap().0;
		// The doughnut is not semantically verified in this runtime.
		// This just checks the execution flow up to dispatch path with a valid doughnut
		let alice = TestAccountId::new(1);
		let bob = TestAccountId::new(2);
		let charlie = TestAccountId::new(3);
		t.extend(balances::GenesisConfig::<Runtime> {
			transaction_base_fee: 0,
			transaction_byte_fee: 0,
			balances: vec![
				(alice.clone(), 100),
				(bob.clone(), 50),
				(charlie.clone(), 0),
			],
			existential_deposit: 0,
			transfer_fee: 0,
			creation_fee: 0,
			vesting: vec![],
		}.build_storage().unwrap().0);

		let doughnut = DummyDoughnut {
			issuer: alice.clone(),
			holder: bob.clone(),
		};
		// Bob signs a tx to send 30 of alice's balance to charlie
		let xt = DoughnutedTestXt::new(Some(2), 0, Call::transfer(charlie.clone(), 30), Some(doughnut));
		let mut t = runtime_io::TestExternalities::<Blake2Hasher>::new(t);
		with_externalities(&mut t, || {
			Executive::initialize_block(&Header::new(1, H256::default(), H256::default(),
				[69u8; 32].into(), Digest::default()));
			Executive::apply_extrinsic(xt).unwrap();
			assert_eq!(<balances::Module<Runtime>>::total_balance(&alice), 70);
			assert_eq!(<balances::Module<Runtime>>::total_balance(&bob), 50);
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
