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

#![allow(unused_must_use)]

use crate::{
	ComputeDispatchFee, ContractAddressFor, GenesisConfig, Module, RawEvent, Trait,
	TrieId, Schedule, TrieIdGenerator,
};
use codec::{Encode, Decode};
use hex_literal::*;
use sp_core::storage::well_known_keys;
use sp_runtime::{
	Perbill, traits::{BlakeTwo256, Hash, IdentityLookup, PlugDoughnutApi},
	testing::{Header, H256},
};
use frame_support::{
	assert_ok, assert_err, impl_outer_dispatch, impl_outer_event, impl_outer_origin,
	parameter_types, StorageValue, traits::{Currency, Get}, weights::Weight,
	additional_traits::DelegatedDispatchVerifier,
};
use std::{cell::RefCell, any::Any};
use frame_system::{self as system, EventRecord, Phase, RawOrigin};
use pallet_balances as balances;

mod contract {
	// Re-export contents of the root. This basically
	// needs to give a name for the current crate.
	// This hack is required for `impl_outer_event!`.
	pub use super::super::*;
}
impl_outer_event! {
	pub enum MetaEvent for Test {
		system, pallet_balances<T>, contract<T>,
	}
}
impl_outer_origin! {
	pub enum Origin for Test { }
}
impl_outer_dispatch! {
	pub enum Call for Test where origin: Origin {
		balances::Balances,
		contract::Contract,
	}
}

thread_local! {
	static EXISTENTIAL_DEPOSIT: RefCell<u64> = RefCell::new(0);
	static TRANSFER_FEE: RefCell<u64> = RefCell::new(0);
	static INSTANTIATION_FEE: RefCell<u64> = RefCell::new(0);
	static BLOCK_GAS_LIMIT: RefCell<u64> = RefCell::new(0);
}

pub struct ExistentialDeposit;
impl Get<u64> for ExistentialDeposit {
	fn get() -> u64 { EXISTENTIAL_DEPOSIT.with(|v| *v.borrow()) }
}

pub struct CreationFee;
impl Get<u64> for CreationFee {
	fn get() -> u64 { INSTANTIATION_FEE.with(|v| *v.borrow()) }
}

pub struct BlockGasLimit;
impl Get<u64> for BlockGasLimit {
	fn get() -> u64 { BLOCK_GAS_LIMIT.with(|v| *v.borrow()) }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Test;

#[derive(Default, Clone, Eq, PartialEq, Debug, Encode, Decode)]
pub struct MockDoughnut {
	/// A mock branching point for verify_runtime_to_contract_call, as a doughnut is verified at different level.
	/// A doughnut is first verified at runtime via Contract::call().
	runtime_verifiable: bool,
	/// A mock branching point for verify_contract_to_contract_call, as a doughnut is verified at different level.
	/// A doughnut is then verified at contract execution via ext_call().
	contract_verifiable: bool,
}
impl MockDoughnut {
	pub fn set_runtime_verifiable(mut self, verifiable: bool) -> Self {
		self.runtime_verifiable = verifiable;
		self
	}
	pub fn set_contract_verifiable(mut self, verifiable: bool) -> Self {
		self.contract_verifiable = verifiable;
		self
	}
}

impl PlugDoughnutApi for MockDoughnut {
	type PublicKey = [u8; 32];
	type Timestamp = u32;
	type Signature = ();
	fn holder(&self) -> Self::PublicKey { Default::default() }
	fn issuer(&self) -> Self::PublicKey { Default::default() }
	fn expiry(&self) -> Self::Timestamp { 0 }
	fn not_before(&self) -> Self::Timestamp { 0 }
	fn payload(&self) -> Vec<u8> { Vec::default() }
	fn signature(&self) -> Self::Signature {}
	fn signature_version(&self) -> u8 { 0 }
	fn get_domain(&self, _domain: &str) -> Option<&[u8]> { None }
	fn validate<Q: AsRef<[u8]>, R: TryInto<u32>>(&self, who: Q, now: R) -> Result<(), ValidationError> {
		// Assume it is valid (not under test here)
		Ok(())
	}
}

pub struct MockDispatchVerifier;
impl DelegatedDispatchVerifier for MockDispatchVerifier {
	type Doughnut = MockDoughnut;
	type AccountId = u64;
	const DOMAIN: &'static str = "";
	fn verify_dispatch(
		_doughnut: &Self::Doughnut,
		_module: &str,
		_method: &str,
		_args: Vec<(&str, &dyn Any)>,
	) -> Result<(), &'static str> {
		Ok(())
	}
	fn verify_runtime_to_contract_call(
		_caller: &Self::AccountId,
		doughnut: &Self::Doughnut,
		_contract_addr: &Self::AccountId,
	) -> Result<(), &'static str> {
		if doughnut.runtime_verifiable {
			Ok(())
		} else {
			Err("Doughnut runtime to contract call verification is not implemented for this domain")
		}
	}
	fn verify_contract_to_contract_call(
		_caller: &Self::AccountId,
		doughnut: &Self::Doughnut,
		_contract_addr: &Self::AccountId,
	) -> Result<(), &'static str> {
		if doughnut.contract_verifiable {
			Ok(())
		} else {
			Err("Doughnut contract to contract call verification is not implemented for this domain")
		}
	}
}

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: Weight = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
}
impl frame_system::Trait for Test {
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Call = ();
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = MetaEvent;
	type BlockHashCount = BlockHashCount;
	type MaximumBlockWeight = MaximumBlockWeight;
	type AvailableBlockRatio = AvailableBlockRatio;
	type MaximumBlockLength = MaximumBlockLength;
	type Version = ();
	type ModuleToIndex = ();
	type Doughnut = MockDoughnut;
	type DelegatedDispatchVerifier = MockDispatchVerifier;
}
impl pallet_balances::Trait for Test {
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
	pub const MinimumPeriod: u64 = 1;
}
impl pallet_timestamp::Trait for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
}
parameter_types! {
	pub const SignedClaimHandicap: u64 = 2;
	pub const TombstoneDeposit: u64 = 16;
	pub const StorageSizeOffset: u32 = 8;
	pub const RentByteFee: u64 = 4;
	pub const RentDepositOffset: u64 = 10_000;
	pub const SurchargeReward: u64 = 150;
	pub const TransactionBaseFee: u64 = 2;
	pub const TransactionByteFee: u64 = 6;
	pub const ContractFee: u64 = 21;
	pub const CallBaseFee: u64 = 135;
	pub const InstantiateBaseFee: u64 = 175;
	pub const MaxDepth: u32 = 100;
	pub const MaxValueSize: u32 = 16_384;
}
impl Trait for Test {
	type Currency = Balances;
	type Time = Timestamp;
	type Randomness = Randomness;
	type Call = Call;
	type DetermineContractAddress = DummyContractAddressFor;
	type Event = MetaEvent;
	type ComputeDispatchFee = DummyComputeDispatchFee;
	type TrieIdGenerator = DummyTrieIdGenerator;
	type GasPayment = ();
	type GasHandler = ();
	type RentPayment = ();
	type SignedClaimHandicap = SignedClaimHandicap;
	type TombstoneDeposit = TombstoneDeposit;
	type StorageSizeOffset = StorageSizeOffset;
	type RentByteFee = RentByteFee;
	type RentDepositOffset = RentDepositOffset;
	type SurchargeReward = SurchargeReward;
	type CreationFee = CreationFee;
	type TransactionBaseFee = TransactionBaseFee;
	type TransactionByteFee = TransactionByteFee;
	type ContractFee = ContractFee;
	type CallBaseFee = CallBaseFee;
	type InstantiateBaseFee = InstantiateBaseFee;
	type MaxDepth = MaxDepth;
	type MaxValueSize = MaxValueSize;
	type BlockGasLimit = BlockGasLimit;
}

type Balances = pallet_balances::Module<Test>;
type Timestamp = pallet_timestamp::Module<Test>;
type Contract = Module<Test>;
type System = frame_system::Module<Test>;
type Randomness = pallet_randomness_collective_flip::Module<Test>;

pub struct DummyContractAddressFor;
impl ContractAddressFor<H256, u64> for DummyContractAddressFor {
	fn contract_address_for(_code_hash: &H256, _data: &[u8], origin: &u64) -> u64 {
		*origin + 1
	}
}

pub struct DummyTrieIdGenerator;
impl TrieIdGenerator<u64> for DummyTrieIdGenerator {
	fn trie_id(account_id: &u64) -> TrieId {
		let new_seed = super::AccountCounter::mutate(|v| {
			*v = v.wrapping_add(1);
			*v
		});

		// TODO: see https://github.com/paritytech/substrate/issues/2325
		let mut res = vec![];
		res.extend_from_slice(well_known_keys::CHILD_STORAGE_KEY_PREFIX);
		res.extend_from_slice(b"default:");
		res.extend_from_slice(&new_seed.to_le_bytes());
		res.extend_from_slice(&account_id.to_le_bytes());
		res
	}
}

pub struct DummyComputeDispatchFee;
impl ComputeDispatchFee<Call, u64> for DummyComputeDispatchFee {
	fn compute_dispatch_fee(_call: &Call) -> u64 {
		69
	}
}

const ALICE: u64 = 1;
const BOB: u64 = 2;
const CHARLIE: u64 = 3;
const DJANGO: u64 = 4;

pub struct ExtBuilder {
	existential_deposit: u64,
	gas_price: u64,
	block_gas_limit: u64,
	transfer_fee: u64,
	instantiation_fee: u64,
}
impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			existential_deposit: 0,
			gas_price: 2,
			block_gas_limit: 100_000_000,
			transfer_fee: 0,
			instantiation_fee: 0,
		}
	}
}
impl ExtBuilder {
	pub fn existential_deposit(mut self, existential_deposit: u64) -> Self {
		self.existential_deposit = existential_deposit;
		self
	}
	pub fn set_associated_consts(&self) {
		EXISTENTIAL_DEPOSIT.with(|v| *v.borrow_mut() = self.existential_deposit);
		TRANSFER_FEE.with(|v| *v.borrow_mut() = self.transfer_fee);
		INSTANTIATION_FEE.with(|v| *v.borrow_mut() = self.instantiation_fee);
		BLOCK_GAS_LIMIT.with(|v| *v.borrow_mut() = self.block_gas_limit);
	}
	pub fn build(self) -> sp_io::TestExternalities {
		self.set_associated_consts();
		let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
		pallet_balances::GenesisConfig::<Test> {
			balances: vec![],
		}.assimilate_storage(&mut t).unwrap();
		GenesisConfig::<Test> {
			current_schedule: Schedule {
				enable_println: true,
				..Default::default()
			},
			gas_price: self.gas_price,
		}.assimilate_storage(&mut t).unwrap();
		sp_io::TestExternalities::new(t)
	}
}

/// Generate Wasm binary and code hash from wabt source.
fn compile_module<T>(wabt_module: &str)
	-> Result<(Vec<u8>, <T::Hashing as Hash>::Output), wabt::Error>
	where T: frame_system::Trait
{
	let wasm = wabt::wat2wasm(wabt_module)?;
	let code_hash = T::Hashing::hash(&wasm);
	Ok((wasm, code_hash))
}

const CODE_RETURN_WITH_DATA: &str = r#"
(module
	(import "env" "ext_scratch_size" (func $ext_scratch_size (result i32)))
	(import "env" "ext_scratch_read" (func $ext_scratch_read (param i32 i32 i32)))
	(import "env" "ext_scratch_write" (func $ext_scratch_write (param i32 i32)))
	(import "env" "memory" (memory 1 1))

	;; Deploy routine is the same as call.
	(func (export "deploy") (result i32)
		(call $call)
	)

	;; Call reads the first 4 bytes (LE) as the exit status and returns the rest as output data.
	(func $call (export "call") (result i32)
		(local $buf_size i32)
		(local $exit_status i32)

		;; Find out the size of the scratch buffer
		(set_local $buf_size (call $ext_scratch_size))

		;; Copy scratch buffer into this contract memory.
		(call $ext_scratch_read
			(i32.const 0)			;; The pointer where to store the scratch buffer contents,
			(i32.const 0)			;; Offset from the start of the scratch buffer.
			(get_local $buf_size)	;; Count of bytes to copy.
		)

		;; Copy all but the first 4 bytes of the input data as the output data.
		(call $ext_scratch_write
			(i32.const 4)	;; Pointer to the data to return.
			(i32.sub		;; Count of bytes to copy.
				(get_local $buf_size)
				(i32.const 4)
			)
		)

		;; Return the first 4 bytes of the input data as the exit status.
		(i32.load (i32.const 0))
	)
)
"#;

const CODE_CALLER_CONTRACT: &str = r#"
(module
	(import "env" "ext_call" (func $ext_call (param i32 i32 i64 i32 i32 i32 i32) (result i32)))
	(import "env" "ext_instantiate" (func $ext_instantiate (param i32 i32 i64 i32 i32 i32 i32) (result i32)))
	(import "env" "ext_scratch_read" (func $ext_scratch_read (param i32 i32 i32)))
	(import "env" "memory" (memory 1 1))

	(func $assert (param i32)
		(block $ok
			(br_if $ok
				(get_local 0)
			)
			(unreachable)
		)
	)

	(func (export "deploy"))

	(func (export "call")
		;; Declare local variables.
		(local $exit_code i32)

		;; Copy code hash from scratch buffer into this contract's memory.
		(call $ext_scratch_read
			(i32.const 24)		;; The pointer where to store the scratch buffer contents,
			(i32.const 0)		;; Offset from the start of the scratch buffer.
			(i32.const 32)		;; Count of bytes to copy.
		)

		;; Deploy the contract successfully.
		(set_local $exit_code
			(call $ext_instantiate
				(i32.const 24)	;; Pointer to the code hash.
				(i32.const 32)	;; Length of the code hash.
				(i64.const 0)	;; How much gas to devote for the execution. 0 = all.
				(i32.const 0)	;; Pointer to the buffer with value to transfer
				(i32.const 8)	;; Length of the buffer with value to transfer.
				(i32.const 8)	;; Pointer to input data buffer address
				(i32.const 8)	;; Length of input data buffer
			)
		)

		;; Check for success exit status.
		(call $assert
			(i32.eq (get_local $exit_code) (i32.const 0x00))
		)

		;; Call the contract successfully.
		(set_local $exit_code
			(call $ext_call
				(i32.const 16)	;; Pointer to "callee" address.
				(i32.const 8)	;; Length of "callee" address.
				(i64.const 0)	;; How much gas to devote for the execution. 0 = all.
				(i32.const 0)	;; Pointer to the buffer with value to transfer
				(i32.const 8)	;; Length of the buffer with value to transfer.
				(i32.const 8)	;; Pointer to input data buffer address
				(i32.const 8)	;; Length of input data buffer
			)
		)
		;; Check for success exit status.
		(call $assert
			(i32.eq (get_local $exit_code) (i32.const 0x00))
		)
	)

	(data (i32.const 0) "\00\80")
	(data (i32.const 8) "\00\11\22\33\44\55\66\77")
)
"#;

#[test]
fn contract_to_contract_call_executes_with_verifiable_doughnut() {
	let (callee_wasm, callee_code_hash) = compile_module::<Test>(CODE_RETURN_WITH_DATA).unwrap();
	let (caller_wasm, caller_code_hash) = compile_module::<Test>(CODE_CALLER_CONTRACT).unwrap();
	let verifiable_doughnut = MockDoughnut::default()
		.set_runtime_verifiable(true)
		.set_contract_verifiable(true);
	let delegated_origin = RawOrigin::from((Some(ALICE), Some(verifiable_doughnut.clone())));

	ExtBuilder::default().existential_deposit(50).build().execute_with(|| {
		Balances::deposit_creating(&ALICE, 1_000_000);
		assert_ok!(Contract::put_code(Origin::signed(ALICE), 100_000, callee_wasm));
		assert_ok!(Contract::put_code(Origin::signed(ALICE), 100_000, caller_wasm));
		assert_ok!(Contract::instantiate(
			Origin::signed(ALICE),
			100_000,
			100_000,
			caller_code_hash.into(),
			vec![],
		));
		// Call BOB contract, which attempts to instantiate and call the callee contract
		assert_ok!(Contract::call(
			delegated_origin.clone().into(),
			BOB,
			0,
			200_000,
			callee_code_hash.as_ref().to_vec(),
		));
	});
}

#[test]
fn contract_to_contract_call_executes_without_doughnut() {
	let (callee_wasm, callee_code_hash) = compile_module::<Test>(CODE_RETURN_WITH_DATA).unwrap();
	let (caller_wasm, caller_code_hash) = compile_module::<Test>(CODE_CALLER_CONTRACT).unwrap();
	let delegated_origin = RawOrigin::from((Some(ALICE), None));

	ExtBuilder::default().existential_deposit(50).build().execute_with(|| {
		Balances::deposit_creating(&ALICE, 1_000_000);
		assert_ok!(Contract::put_code(Origin::signed(ALICE), 100_000, callee_wasm));
		assert_ok!(Contract::put_code(Origin::signed(ALICE), 100_000, caller_wasm));
		assert_ok!(Contract::instantiate(
			Origin::signed(ALICE),
			100_000,
			100_000,
			caller_code_hash.into(),
			vec![],
		));
		// Call BOB contract, which attempts to instantiate and call the callee contract
		assert_ok!(Contract::call(
			delegated_origin.into(),
			BOB,
			0,
			200_000,
			callee_code_hash.as_ref().to_vec(),
		));
	});
}

#[test]
fn contract_to_contract_call_returns_error_with_unverifiable_doughnut() {
	let (callee_wasm, callee_code_hash) = compile_module::<Test>(CODE_RETURN_WITH_DATA).unwrap();
	let (caller_wasm, caller_code_hash) = compile_module::<Test>(CODE_CALLER_CONTRACT).unwrap();

	// Doughnut is first verified at runtime before execution,
	// hence runtime_verifiable set to true to bypass the check.
	let unverifiable_doughnut = MockDoughnut::default()
		.set_runtime_verifiable(true)
		.set_contract_verifiable(false);
	let delegated_origin = RawOrigin::from((Some(ALICE), Some(unverifiable_doughnut.clone())));

	ExtBuilder::default().existential_deposit(50).build().execute_with(|| {
		Balances::deposit_creating(&ALICE, 1_000_000);
		assert_ok!(Contract::put_code(Origin::signed(ALICE), 100_000, callee_wasm));
		assert_ok!(Contract::put_code(Origin::signed(ALICE), 100_000, caller_wasm));
		assert_ok!(Contract::instantiate(
			Origin::signed(ALICE),
			100_000,
			100_000,
			caller_code_hash.into(),
			vec![],
		));
		// Call BOB contract, which attempts to instantiate and call the callee contract
		assert_err!(
			Contract::call(
				delegated_origin.into(),
				BOB,
				0,
				200_000,
				callee_code_hash.as_ref().to_vec(),
			),
			"contract trapped during execution", // expected as $exit_code = 0x11 from $ext_call
		);
	});
}

const CODE_DELEGATED_DISPATCH_CALL: &str = r#"
(module
	(import "env" "ext_delegated_dispatch_call" (func $ext_delegated_dispatch_call (param i32 i32)))
	(import "env" "memory" (memory 1 1))

	(func (export "call")
		(call $ext_delegated_dispatch_call
			(i32.const 8)	;; Pointer to the start of encoded call buffer
			(i32.const 11)	;; Length of the buffer
		)
	)
	(func (export "deploy"))

	;; Encoding of balance transfer of 50 to Charlie
	(data (i32.const 8) "\00\00\03\00\00\00\00\00\00\00\C8")
)
"#;

#[test]
fn delegated_contract_to_runtime_call_executes_with_verifiable_doughnut() {
	// Ensure we are using the correct encoding (of a call) above to test
	let encoded = Encode::encode(&Call::Balances(pallet_balances::Call::transfer(CHARLIE, 50)));
	assert_eq!(&encoded[..], &hex!("00000300000000000000C8")[..]);

	let (wasm, code_hash) = compile_module::<Test>(CODE_DELEGATED_DISPATCH_CALL).unwrap();
	let verifiable_doughnut = MockDoughnut::default()
		.set_runtime_verifiable(true)
		.set_contract_verifiable(true);
	let delegated_origin = RawOrigin::from((Some(DJANGO), Some(verifiable_doughnut.clone())));

	ExtBuilder::default().existential_deposit(50).build().execute_with(|| {
		Balances::deposit_creating(&ALICE, 1_000_000);
		Balances::deposit_creating(&DJANGO, 1_000_000);

		assert_ok!(Contract::put_code(Origin::signed(ALICE), 100_000, wasm));

		// contract account BOB is created by instantiating the contract.
		assert_ok!(Contract::instantiate(
			Origin::signed(ALICE),
			100,
			100_000,
			code_hash.into(),
			vec![],
		));

		// DJANGO is calling the contract BOB.
		assert_ok!(Contract::call(
			delegated_origin.clone().into(),
			BOB,
			0,
			100_000,
			vec![],
		));

		assert_eq!(
			System::events(),
			vec![
				EventRecord {
					phase: Phase::Initialization,
					event: MetaEvent::pallet_balances(balances::RawEvent::NewAccount(ALICE, 1000000)),
					topics: vec![]
				},
				EventRecord {
					phase: Phase::Initialization,
					event: MetaEvent::pallet_balances(balances::RawEvent::NewAccount(DJANGO, 1000000)),
					topics: vec![]
				},
				// Events from Contract::put_code
				EventRecord {
					phase: Phase::Initialization,
					event: MetaEvent::contract(RawEvent::CodeStored(code_hash.into())),
					topics: vec![],
				},

				// Contract::instantiate
				EventRecord {
					phase: Phase::Initialization,
					event: MetaEvent::pallet_balances(pallet_balances::RawEvent::NewAccount(BOB, 100)),
					topics: vec![],
				},
				EventRecord {
					phase: Phase::Initialization,
					event: MetaEvent::contract(RawEvent::Transfer(ALICE, BOB, 100)),
					topics: vec![],
				},
				EventRecord {
					phase: Phase::Initialization,
					event: MetaEvent::contract(RawEvent::Instantiated(ALICE, BOB)),
					topics: vec![],
				},

				// Dispatching the call.
				EventRecord {
					phase: Phase::Initialization,
					event: MetaEvent::pallet_balances(pallet_balances::RawEvent::NewAccount(CHARLIE, 50)),
					topics: vec![],
				},
				EventRecord {
					phase: Phase::Initialization,
					event: MetaEvent::pallet_balances(pallet_balances::RawEvent::Transfer(DJANGO, CHARLIE, 50, 0)),
					topics: vec![],
				},

				// Event emited as a result of dispatch.
				EventRecord {
					phase: Phase::Initialization,
					event: MetaEvent::contract(RawEvent::DelegatedDispatched(DJANGO, verifiable_doughnut, true)),
					topics: vec![],
				}
			]
		);
	});
}

#[test]
fn delegated_runtime_to_contract_call_returns_error_with_unverifiable_doughnut() {
	// Ensure we are using the correct encoding (of a call) above to test
	let encoded = Encode::encode(&Call::Balances(pallet_balances::Call::transfer(CHARLIE, 50)));
	assert_eq!(&encoded[..], &hex!("00000300000000000000C8")[..]);

	let (wasm, code_hash) = compile_module::<Test>(CODE_DELEGATED_DISPATCH_CALL).unwrap();

	// Because doughnut is first verified at runtime before contract call execution,
	// Contract::call should return error even if it's verifiable at ext_call level
	let unverifiable_doughnut = MockDoughnut::default()
		.set_runtime_verifiable(false)
		.set_contract_verifiable(true);
	let delegated_origin = RawOrigin::from((Some(DJANGO), Some(unverifiable_doughnut.clone())));

	ExtBuilder::default().existential_deposit(50).build().execute_with(|| {
		Balances::deposit_creating(&ALICE, 1_000_000);
		Balances::deposit_creating(&DJANGO, 1_000_000);

		assert_ok!(Contract::put_code(Origin::signed(ALICE), 100_000, wasm));

		// contract account BOB is created by instantiating the contract.
		assert_ok!(Contract::instantiate(
			Origin::signed(ALICE),
			100,
			100_000,
			code_hash.into(),
			vec![],
		));

		// DJANGO is calling the contract BOB.
		assert_err!(
			Contract::call(
				delegated_origin.clone().into(),
				BOB,
				0,
				100_000,
				vec![],
			),
			"Doughnut runtime to contract call verification is not implemented for this domain",
		);
	});
}

/// Simple test to ensure that gas charges are made to the issuer
/// (who becomes the caller of a delegated origin)
#[test]
fn contract_call_charges_gas_to_issuer() {
	let verifiable_doughnut = MockDoughnut::default()
		.set_runtime_verifiable(true);
	let delegated_origin = RawOrigin::from((Some(ALICE), Some(verifiable_doughnut.clone())));
	let gas_limit = 200_000;
	let initial_balance = 1_000_000;
	let gas_cost = 970;

	ExtBuilder::default().existential_deposit(50).build().execute_with(|| {
		Balances::deposit_creating(&ALICE, initial_balance);
		// Call BOB contract, which attempts to instantiate and call the callee contract
		assert_ok!(Contract::call(
			delegated_origin.into(),
			BOB,
			500,
			gas_limit,
			vec![],
		));

		assert_eq!(
			Balances::free_balance(&ALICE),
			initial_balance - gas_cost
		);
	});
}
