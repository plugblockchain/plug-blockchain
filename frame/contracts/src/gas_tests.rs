// Copyright 2019-2020
//     by  Centrality Investments Ltd.
//     and Parity Technologies (UK) Ltd.
// This file is part of Substrate.
//
// This file tests an alternative implementation of GasHandler trait in gas.rs
// This trait allows customized implementation when filling and emptying gas meters
//
// The Default substrate charges the user upfront when gas is filled, then refund the user
// on when emptying unused gas.
//

#![cfg(test)]
#![allow(unused)]

// Set up the GasTest struct (Test Environment)
use crate::{
    gas::{buy_gas, Gas, GasHandler, GasMeter, Token},
    tests::ExtBuilder,
    BalanceOf, ComputeDispatchFee, ContractAddressFor, Module, Trait, TrieId, TrieIdGenerator,
};

use frame_support::{
    dispatch::DispatchError,
    impl_outer_dispatch, impl_outer_event, impl_outer_origin, parameter_types,
    traits::{Currency, Get},
    weights::Weight,
    StorageValue,
};
use frame_system::{self as system};
use sp_runtime::{
    testing::{Header, H256},
    traits::{BlakeTwo256, IdentityLookup},
    Perbill,
};
use sp_std::cell::RefCell;

mod contract {
    // Re-export contents of the root. This basically
    // needs to give a name for the current crate.
    // This hack is required for `impl_outer_event!`.
    pub use super::super::*;
}

use pallet_balances as balances;
impl_outer_event! {
    pub enum MetaEvent for GasTest {
        balances<T>, contract<T>,
    }
}

impl_outer_origin! {
    pub enum Origin for GasTest { }
}
impl_outer_dispatch! {
    pub enum Call for GasTest where origin: Origin {
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
    fn get() -> u64 {
        EXISTENTIAL_DEPOSIT.with(|v| *v.borrow())
    }
}

pub struct TransferFee;
impl Get<u64> for TransferFee {
    fn get() -> u64 {
        TRANSFER_FEE.with(|v| *v.borrow())
    }
}

pub struct CreationFee;
impl Get<u64> for CreationFee {
    fn get() -> u64 {
        INSTANTIATION_FEE.with(|v| *v.borrow())
    }
}

pub struct BlockGasLimit;
impl Get<u64> for BlockGasLimit {
    fn get() -> u64 {
        BLOCK_GAS_LIMIT.with(|v| *v.borrow())
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct GasTest;
parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: Weight = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::one();
}
impl system::Trait for GasTest {
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
    type Doughnut = ();
    type DelegatedDispatchVerifier = ();
}
impl balances::Trait for GasTest {
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
impl pallet_timestamp::Trait for GasTest {
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
impl Trait for GasTest {
    type Currency = Balances;
    type Time = Timestamp;
    type Randomness = Randomness;
    type Call = Call;
    type DetermineContractAddress = DummyContractAddressFor;
    type Event = MetaEvent;
    type ComputeDispatchFee = DummyComputeDispatchFee;
    type TrieIdGenerator = DummyTrieIdGenerator;
    type GasPayment = ();
    type GasHandler = TestGasHandler;
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

type Balances = balances::Module<GasTest>;
type Timestamp = pallet_timestamp::Module<GasTest>;
type Contract = Module<GasTest>;
type System = system::Module<GasTest>;
type Randomness = pallet_randomness_collective_flip::Module<GasTest>;

pub struct DummyContractAddressFor;
impl ContractAddressFor<H256, u64> for DummyContractAddressFor {
    fn contract_address_for(_code_hash: &H256, _data: &[u8], origin: &u64) -> u64 {
        *origin + 1
    }
}

pub struct DummyTrieIdGenerator;
impl TrieIdGenerator<u64> for DummyTrieIdGenerator {
    fn trie_id(account_id: &u64) -> TrieId {
        use sp_core::storage::well_known_keys;

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
    fn compute_dispatch_fee(call: &Call) -> u64 {
        69
    }
}

// A trivial token that has a 1:1 cost with gas
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
struct SimpleToken(u64);
impl Token<GasTest> for SimpleToken {
    type Metadata = ();
    fn calculate_amount(&self, _metadata: &()) -> u64 {
        self.0
    }
}

const ALICE: u64 = 1;
// End of GasTest setup

///
/// This is an alternative implementation of GasHandler trait used for testing
///
/// `fill_gas` will simply fill the gas meter without charging the user
/// `empty_unused_gas` will charge the user based on the actual amount of gas spent
///
pub struct TestGasHandler;
impl GasHandler<GasTest> for TestGasHandler {
    fn fill_gas(
        _transactor: &<GasTest as system::Trait>::AccountId,
        gas_limit: Gas,
    ) -> Result<GasMeter<GasTest>, DispatchError> {
        // fills the gas meter without charging the user
        Ok(GasMeter::with_limit(gas_limit, 1))
    }

    fn empty_unused_gas(
        transactor: &<GasTest as system::Trait>::AccountId,
        gas_meter: GasMeter<GasTest>,
    ) {
        // charge the users based on the amount of gas used
        buy_gas::<GasTest>(transactor, gas_meter.spent());
    }
}

pub struct NoChargeGasHandler;
impl GasHandler<GasTest> for NoChargeGasHandler {
    fn fill_gas(
        _transactor: &<GasTest as frame_system::Trait>::AccountId,
        gas_limit: Gas,
    ) -> Result<GasMeter<GasTest>, DispatchError> {
        // fills the gas meter without charging the user
        Ok(GasMeter::with_limit(gas_limit, 1))
    }

    fn empty_unused_gas(
        transactor: &<GasTest as frame_system::Trait>::AccountId,
        gas_meter: GasMeter<GasTest>,
    ) {
        // Do not charge the transactor. Give gas for free.
    }
}

#[test]
// Tests that the user is not charged when filling up gas meters
fn customized_fill_gas_does_not_charge_the_user() {
    ExtBuilder::default()
        .existential_deposit(50)
        .gas_price(1)
        .build()
        .execute_with(|| {
            // Create test account
            Balances::deposit_creating(&ALICE, 1000);

            let gas_limit = 500;
            let mut gas_meter = NoChargeGasHandler::fill_gas(&ALICE, gas_limit).unwrap();
            // Charge as if the whole gas_limit is used
            gas_meter.charge(&(), SimpleToken(gas_limit));
            NoChargeGasHandler::empty_unused_gas(&ALICE, gas_meter);

            // Check the user is not charged
            assert_eq!(Balances::free_balance(&ALICE), 1000);
        });
}

#[test]
// Tests that the user is charged on "emptying" unused gas
fn user_is_charged_on_empty_unused_gas() {
    ExtBuilder::default()
        .existential_deposit(50)
        .gas_price(1)
        .build()
        .execute_with(|| {
            // Fill the meter
            let gas_used = 250;
            Balances::deposit_creating(&ALICE, 1000);
            let gas_meter_result = TestGasHandler::fill_gas(&ALICE, 500);
            assert!(gas_meter_result.is_ok());
            let mut gas_meter = gas_meter_result.unwrap();

            // Estimate the cost of gas used. Gas price is set to 1
            let expected_remaining_balance = Balances::free_balance(&ALICE) - gas_used;

            // Spend half the gas, then empty the gas meter, the user should be charged here
            gas_meter.charge(&(), SimpleToken(gas_used));
            TestGasHandler::empty_unused_gas(&ALICE, gas_meter);

            assert_eq!(Balances::free_balance(&ALICE), expected_remaining_balance);
        });
}

#[test]
// Tests that if the gas meter run out of gas, the user is charged the full amount of gas consumed.
fn user_is_charged_if_out_of_gas() {
    ExtBuilder::default()
        .existential_deposit(50)
        .gas_price(1)
        .build()
        .execute_with(|| {
            // Fill the meter
            let gas_limit = 500;
            Balances::deposit_creating(&ALICE, 1000);
            let gas_meter_result = TestGasHandler::fill_gas(&ALICE, gas_limit);
            let mut gas_meter = gas_meter_result.unwrap();

            // We expect all the gas will be used up
            let expected_remaining_balance = Balances::free_balance(&ALICE) - gas_limit;

            // Spend more gas than the `gas_limit`, then empty the gas meter, the user should be charged here
            gas_meter.charge(&(), SimpleToken(gas_limit + 1));
            TestGasHandler::empty_unused_gas(&ALICE, gas_meter);

            assert_eq!(Balances::free_balance(&ALICE), expected_remaining_balance);
        });
}
