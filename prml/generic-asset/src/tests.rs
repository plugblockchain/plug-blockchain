// Copyright 2019-2021
//     by  Centrality Investments Ltd.
//     and Parity Technologies (UK) Ltd.
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

//! Tests for the module.

#![cfg(test)]

use super::*;
use crate::mock::{
	new_test_ext_with_balance, new_test_ext_with_default, new_test_ext_with_next_asset_id,
	new_test_ext_with_permissions, Event as TestEvent, GenericAsset, NegativeImbalanceOf, Origin, PositiveImbalanceOf,
	System, Test, TreasuryModuleId, ALICE, ASSET_ID, BOB, CHARLIE, ID_1, ID_2, INITIAL_BALANCE, INITIAL_ISSUANCE,
	SPENDING_ASSET_ID, STAKING_ASSET_ID, TEST1_ASSET_ID, TEST2_ASSET_ID,
};
use crate::CheckedImbalance;
use frame_support::{
	assert_noop, assert_ok,
	traits::{Imbalance, OnRuntimeUpgrade},
};
use sp_runtime::traits::AccountIdConversion;

fn asset_options(permissions: PermissionLatest<u64>, decimal_place: u8) -> AssetOptions<u64, u64> {
	let decimal_offset = 10u128.saturating_pow(decimal_place.into());
	AssetOptions {
		initial_issuance: (INITIAL_ISSUANCE as u128 / decimal_offset) as u64,
		permissions,
	}
}

#[test]
fn issuing_asset_units_to_issuer_should_work() {
	new_test_ext_with_balance(STAKING_ASSET_ID, ALICE, INITIAL_BALANCE).execute_with(|| {
		let permissions = PermissionLatest::new(ALICE);
		let asset_info = AssetInfo::default();

		assert_eq!(GenericAsset::next_asset_id(), ASSET_ID);
		assert_ok!(GenericAsset::create(
			Origin::root(),
			ALICE,
			asset_options(permissions, asset_info.decimal_places()),
			asset_info
		));
		assert_eq!(GenericAsset::next_asset_id(), ASSET_ID + 1);

		assert_eq!(GenericAsset::total_issuance(&ASSET_ID), INITIAL_ISSUANCE);
		assert_eq!(GenericAsset::free_balance(ASSET_ID, &ALICE), INITIAL_ISSUANCE);
		assert_eq!(GenericAsset::free_balance(STAKING_ASSET_ID, &ALICE), INITIAL_BALANCE);
	});
}

#[test]
fn issuing_with_next_asset_id_overflow_should_fail() {
	new_test_ext_with_balance(STAKING_ASSET_ID, ALICE, INITIAL_BALANCE).execute_with(|| {
		let permissions = PermissionLatest::new(ALICE);
		NextAssetId::<Test>::put(u32::max_value());
		let asset_info = AssetInfo::default();

		assert_noop!(
			GenericAsset::create(
				Origin::root(),
				ALICE,
				asset_options(permissions, asset_info.decimal_places()),
				asset_info
			),
			Error::<Test>::AssetIdExhausted
		);
		assert_eq!(GenericAsset::next_asset_id(), u32::max_value());
	});
}

#[test]
fn querying_total_supply_should_work() {
	new_test_ext_with_balance(STAKING_ASSET_ID, ALICE, INITIAL_BALANCE).execute_with(|| {
		let permissions = PermissionLatest::new(ALICE);
		let transfer_amount = 50;
		let asset_info = AssetInfo::default();

		assert_ok!(GenericAsset::create(
			Origin::root(),
			ALICE,
			asset_options(permissions, asset_info.decimal_places()),
			asset_info
		));
		assert_eq!(GenericAsset::free_balance(ASSET_ID, &ALICE), INITIAL_ISSUANCE);
		assert_eq!(GenericAsset::total_issuance(ASSET_ID), INITIAL_ISSUANCE);

		assert_ok!(GenericAsset::transfer(
			Origin::signed(ALICE),
			ASSET_ID,
			BOB,
			transfer_amount
		));
		assert_eq!(
			GenericAsset::free_balance(ASSET_ID, &ALICE),
			INITIAL_ISSUANCE - transfer_amount
		);
		assert_eq!(GenericAsset::free_balance(ASSET_ID, &BOB), transfer_amount);
		assert_eq!(GenericAsset::total_issuance(ASSET_ID), INITIAL_ISSUANCE);

		assert_ok!(GenericAsset::transfer(
			Origin::signed(BOB),
			ASSET_ID,
			CHARLIE,
			transfer_amount / 2
		));
		assert_eq!(GenericAsset::free_balance(ASSET_ID, &BOB), transfer_amount / 2);
		assert_eq!(GenericAsset::free_balance(ASSET_ID, &CHARLIE), transfer_amount / 2);
		assert_eq!(GenericAsset::total_issuance(ASSET_ID), INITIAL_ISSUANCE);
	});
}

// Given
// - The next asset id as `asset_id` = 1000.
// - AssetOptions with all permissions.
// - GenesisStore has sufficient free balance.
//
// When
// - Create an asset from `origin` as 1.
// Then
// - free_balance of next asset id = 1000.
//
// When
// - After transferring 40 from account 1 to account 2.
// Then
// - Origin account's `free_balance` = 60.
// - account 2's `free_balance` = 40.
#[test]
fn transferring_amount_should_work() {
	new_test_ext_with_balance(STAKING_ASSET_ID, ALICE, INITIAL_BALANCE).execute_with(|| {
		let permissions = PermissionLatest::new(ALICE);
		let transfer_ammount = 40;
		let asset_info = AssetInfo::default();

		assert_ok!(GenericAsset::create(
			Origin::root(),
			ALICE,
			asset_options(permissions, asset_info.decimal_places()),
			asset_info
		));
		assert_eq!(GenericAsset::free_balance(ASSET_ID, &ALICE), INITIAL_ISSUANCE);
		assert_ok!(GenericAsset::transfer(
			Origin::signed(ALICE),
			ASSET_ID,
			BOB,
			transfer_ammount
		));
		assert_eq!(
			GenericAsset::free_balance(ASSET_ID, &ALICE),
			INITIAL_ISSUANCE - transfer_ammount
		);
		assert_eq!(GenericAsset::free_balance(ASSET_ID, &BOB), transfer_ammount);
	});
}

// Given
// - The next asset id as `asset_id` = 1000.
// - AssetOptions with all permissions.
// - GenesisStore has sufficient free balance.
//
// When
// - Create an asset from `origin` as 1.
// Then
// - free_balance of next asset id = 1000.
//
// When
// - After transferring amount more than free balance of 1.
// Then
// - throw error with insufficient balance.
#[test]
fn transferring_amount_more_than_free_balance_should_fail() {
	new_test_ext_with_balance(STAKING_ASSET_ID, ALICE, INITIAL_BALANCE).execute_with(|| {
		let permissions = PermissionLatest::new(ALICE);
		let asset_info = AssetInfo::default();

		assert_ok!(GenericAsset::create(
			Origin::root(),
			ALICE,
			asset_options(permissions, asset_info.decimal_places()),
			asset_info
		));
		assert_eq!(GenericAsset::free_balance(ASSET_ID, &ALICE), INITIAL_ISSUANCE);
		assert_noop!(
			GenericAsset::transfer(Origin::signed(ALICE), ASSET_ID, BOB, INITIAL_ISSUANCE + 1),
			Error::<Test>::InsufficientBalance
		);
	});
}

#[test]
fn transferring_less_than_one_unit_should_fail() {
	new_test_ext_with_balance(STAKING_ASSET_ID, ALICE, INITIAL_BALANCE).execute_with(|| {
		let permissions = PermissionLatest::new(ALICE);
		let asset_info = AssetInfo::default();

		assert_ok!(GenericAsset::create(
			Origin::root(),
			ALICE,
			asset_options(permissions, asset_info.decimal_places()),
			asset_info
		));
		assert_eq!(GenericAsset::free_balance(ASSET_ID, &ALICE), INITIAL_ISSUANCE);
		assert_noop!(
			GenericAsset::transfer(Origin::signed(ALICE), ASSET_ID, BOB, 0),
			Error::<Test>::ZeroAmount
		);
	});
}

#[test]
fn transfer_extrinsic_allows_death() {
	new_test_ext_with_balance(STAKING_ASSET_ID, ALICE, INITIAL_BALANCE).execute_with(|| {
		GenericAsset::set_free_balance(STAKING_ASSET_ID, &BOB, INITIAL_BALANCE);
		assert!(<Test as Config>::AccountStore::get(&BOB).should_exist());
		assert!(System::account_exists(&BOB));
		assert_ok!(GenericAsset::transfer(
			Origin::signed(BOB),
			STAKING_ASSET_ID,
			ALICE,
			INITIAL_BALANCE
		));
		assert!(!<Test as Config>::AccountStore::get(&BOB).should_exist());

		// TODO Enable the following check after https://github.com/plugblockchain/plug-blockchain/issues/191
		// assert!(!System::account_exists(&BOB));

		assert!(!<FreeBalance<Test>>::contains_key(STAKING_ASSET_ID, &BOB));
	});
}

#[test]
fn transfer_dust_balance_can_create_an_account() {
	new_test_ext_with_balance(STAKING_ASSET_ID, ALICE, INITIAL_BALANCE).execute_with(|| {
		let asset_info = AssetInfo::new(b"TST1".to_vec(), 1, 11);
		assert_ok!(GenericAsset::create(
			Origin::root(),
			ALICE,
			asset_options(PermissionLatest::new(ALICE), 4),
			asset_info.clone()
		));
		assert!(!<Test as Config>::AccountStore::get(&BOB).should_exist());
		assert!(!System::account_exists(&BOB));

		// Transfer dust balance to BOB
		assert_ok!(GenericAsset::transfer(
			Origin::signed(ALICE),
			STAKING_ASSET_ID,
			BOB,
			asset_info.existential_deposit() - 1
		));

		assert!(<Test as Config>::AccountStore::get(&BOB).should_exist());
		assert!(System::account_exists(&BOB));
	});
}

#[test]
fn an_account_with_a_consumer_should_persist_in_system() {
	new_test_ext_with_balance(STAKING_ASSET_ID, ALICE, INITIAL_BALANCE).execute_with(|| {
		GenericAsset::set_free_balance(STAKING_ASSET_ID, &BOB, INITIAL_BALANCE);
		assert!(<Test as Config>::AccountStore::get(&BOB).should_exist());
		assert!(System::account_exists(&BOB));
		assert_ok!(System::inc_consumers(&BOB));
		assert_ok!(GenericAsset::transfer(
			Origin::signed(BOB),
			STAKING_ASSET_ID,
			ALICE,
			INITIAL_BALANCE
		));
		assert!(!<FreeBalance<Test>>::contains_key(STAKING_ASSET_ID, &BOB));
		assert!(!<Test as Config>::AccountStore::get(&BOB).should_exist());
		assert!(System::account_exists(&BOB));
	});
}

#[test]
fn transfer_with_keep_existential_requirement() {
	new_test_ext_with_balance(STAKING_ASSET_ID, ALICE, INITIAL_BALANCE).execute_with(|| {
		GenericAsset::set_free_balance(STAKING_ASSET_ID, &BOB, INITIAL_BALANCE);
		assert!(<Test as Config>::AccountStore::get(&BOB).should_exist());
		assert!(System::account_exists(&BOB));
		assert_ok!(StakingAssetCurrency::<Test>::transfer(
			&BOB,
			&ALICE,
			INITIAL_BALANCE,
			ExistenceRequirement::KeepAlive
		));
		assert!(<Test as Config>::AccountStore::get(&BOB).should_exist());
		assert!(System::account_exists(&BOB));
		assert!(<FreeBalance<Test>>::contains_key(STAKING_ASSET_ID, &BOB));
	});
}

#[test]
fn transfer_with_allow_death_existential_requirement() {
	new_test_ext_with_balance(STAKING_ASSET_ID, ALICE, INITIAL_BALANCE).execute_with(|| {
		GenericAsset::set_free_balance(STAKING_ASSET_ID, &BOB, INITIAL_BALANCE);
		assert!(<Test as Config>::AccountStore::get(&BOB).should_exist());
		assert!(System::account_exists(&BOB));
		assert_ok!(StakingAssetCurrency::<Test>::transfer(
			&BOB,
			&ALICE,
			INITIAL_BALANCE,
			ExistenceRequirement::AllowDeath
		));
		assert!(!<Test as Config>::AccountStore::get(&BOB).should_exist());

		// TODO Enable the following check after https://github.com/plugblockchain/plug-blockchain/issues/191
		// assert!(!System::account_exists(&BOB));

		assert!(!<FreeBalance<Test>>::contains_key(STAKING_ASSET_ID, &BOB));
	});
}

#[test]
fn any_reserved_balance_prevent_purging() {
	new_test_ext_with_balance(STAKING_ASSET_ID, ALICE, INITIAL_BALANCE).execute_with(|| {
		GenericAsset::set_free_balance(STAKING_ASSET_ID, &BOB, INITIAL_BALANCE);
		GenericAsset::set_reserved_balance(STAKING_ASSET_ID, &BOB, INITIAL_BALANCE);
		assert!(<Test as Config>::AccountStore::get(&BOB).should_exist());
		assert!(System::account_exists(&BOB));
		assert_ok!(GenericAsset::transfer(
			Origin::signed(BOB),
			STAKING_ASSET_ID,
			ALICE,
			INITIAL_BALANCE
		));
		assert!(<Test as Config>::AccountStore::get(&BOB).should_exist());
		assert!(System::account_exists(&BOB));
		assert!(<FreeBalance<Test>>::contains_key(STAKING_ASSET_ID, &BOB));
	});
}

#[test]
fn an_asset_with_some_lock_should_not_be_purged_even_when_dust() {
	new_test_ext_with_balance(STAKING_ASSET_ID, ALICE, INITIAL_BALANCE).execute_with(|| {
		let asset_info = AssetInfo::new(b"TST1".to_vec(), 1, 11);

		assert_ok!(GenericAsset::create(
			Origin::root(),
			ALICE,
			asset_options(PermissionLatest::new(ALICE), asset_info.decimal_places()),
			asset_info.clone()
		));

		GenericAsset::set_free_balance(ASSET_ID, &BOB, INITIAL_BALANCE);
		GenericAsset::set_free_balance(STAKING_ASSET_ID, &BOB, INITIAL_BALANCE);

		let lock_amount = asset_info.existential_deposit() - 1;
		GenericAsset::set_lock(ID_1, ASSET_ID, &BOB, lock_amount, WithdrawReasons::TRANSACTION_PAYMENT);

		assert_ok!(GenericAsset::transfer(
			Origin::signed(BOB),
			STAKING_ASSET_ID,
			ALICE,
			INITIAL_BALANCE
		));

		assert_ok!(GenericAsset::transfer(
			Origin::signed(BOB),
			ASSET_ID,
			ALICE,
			INITIAL_BALANCE - lock_amount
		));

		// BOB's staking asset should be purged as it had no locks
		assert!(!<FreeBalance<Test>>::contains_key(STAKING_ASSET_ID, &BOB));
		// BOB's ASSET_ID should not be purged due to the lock
		assert!(<FreeBalance<Test>>::contains_key(ASSET_ID, &BOB));

		// BOB's account should continue to exist as there is one asset not purged
		assert!(<Test as Config>::AccountStore::get(&BOB).should_exist());

		// Check the left over of ASSET_ID for BOB is non significant even though we have kept it due to the lock
		assert!(<FreeBalance<Test>>::get(&ASSET_ID, &BOB) < asset_info.existential_deposit());
	});
}

#[test]
fn balance_falls_below_a_non_default_existential_deposit() {
	new_test_ext_with_balance(STAKING_ASSET_ID, ALICE, INITIAL_BALANCE).execute_with(|| {
		let asset_info = AssetInfo::new(b"TST1".to_vec(), 1, 11);
		assert_ok!(GenericAsset::create(
			Origin::root(),
			ALICE,
			asset_options(PermissionLatest::new(ALICE), asset_info.decimal_places()),
			asset_info.clone()
		));
		GenericAsset::set_free_balance(ASSET_ID, &BOB, INITIAL_BALANCE);
		assert!(<Test as Config>::AccountStore::get(&BOB).should_exist());
		assert!(System::account_exists(&BOB));
		assert_ok!(GenericAsset::transfer(
			Origin::signed(BOB),
			ASSET_ID,
			ALICE,
			INITIAL_BALANCE - asset_info.existential_deposit()
		));
		assert!(<Test as Config>::AccountStore::get(&BOB).should_exist());
		assert!(System::account_exists(&BOB));
		assert!(<FreeBalance<Test>>::contains_key(ASSET_ID, &BOB));
		assert_ok!(GenericAsset::transfer(Origin::signed(BOB), ASSET_ID, ALICE, 1));
		assert!(!<Test as Config>::AccountStore::get(&BOB).should_exist());
		// TODO Enable the following check after https://github.com/plugblockchain/plug-blockchain/issues/191
		// assert!(!System::account_exists(&BOB));
		assert!(!<FreeBalance<Test>>::contains_key(ASSET_ID, &BOB));
	});
}

#[test]
fn minimum_balance_is_existential_deposit() {
	new_test_ext_with_permissions(vec![(STAKING_ASSET_ID, ALICE), (SPENDING_ASSET_ID, ALICE)]).execute_with(|| {
		let stk_min = 11u64;
		let spd_min = 17u64;
		let staking_asset_info = AssetInfo::new(b"STK".to_vec(), 1, stk_min);
		let spending_asset_info = AssetInfo::new(b"SPD".to_vec(), 2, spd_min);
		assert_ok!(GenericAsset::create_asset(
			Some(STAKING_ASSET_ID),
			Some(ALICE),
			asset_options(PermissionLatest::new(ALICE), staking_asset_info.decimal_places()),
			staking_asset_info
		));
		assert_ok!(GenericAsset::create_asset(
			Some(SPENDING_ASSET_ID),
			Some(ALICE),
			asset_options(PermissionLatest::new(ALICE), spending_asset_info.decimal_places()),
			spending_asset_info
		));
		assert_eq!(StakingAssetCurrency::<Test>::minimum_balance(), stk_min);
		assert_eq!(SpendingAssetCurrency::<Test>::minimum_balance(), spd_min);
	});
}

#[test]
fn purge_happens_per_asset() {
	new_test_ext_with_balance(STAKING_ASSET_ID, ALICE, INITIAL_BALANCE).execute_with(|| {
		let asset_info = AssetInfo::default();

		assert_ok!(GenericAsset::create(
			Origin::root(),
			ALICE,
			asset_options(PermissionLatest::new(ALICE), asset_info.decimal_places()),
			asset_info
		));
		GenericAsset::set_free_balance(STAKING_ASSET_ID, &BOB, INITIAL_BALANCE);
		GenericAsset::set_free_balance(ASSET_ID, &BOB, INITIAL_BALANCE);
		assert!(<Test as Config>::AccountStore::get(&BOB).should_exist());
		assert!(System::account_exists(&BOB));
		assert_ok!(GenericAsset::transfer(
			Origin::signed(BOB),
			STAKING_ASSET_ID,
			ALICE,
			INITIAL_BALANCE
		));
		assert!(<Test as Config>::AccountStore::get(&BOB).should_exist());
		assert!(System::account_exists(&BOB));
		assert!(!<FreeBalance<Test>>::contains_key(STAKING_ASSET_ID, &BOB));
		assert!(!<ReservedBalance<Test>>::contains_key(STAKING_ASSET_ID, &BOB));
		assert_ok!(GenericAsset::transfer(
			Origin::signed(BOB),
			ASSET_ID,
			ALICE,
			INITIAL_BALANCE
		));
		assert!(!<Test as Config>::AccountStore::get(&BOB).should_exist());

		// TODO Enable the following check after https://github.com/plugblockchain/plug-blockchain/issues/191
		// assert!(!System::account_exists(&BOB));

		assert!(!<FreeBalance<Test>>::contains_key(ASSET_ID, &BOB));
		assert!(!<ReservedBalance<Test>>::contains_key(ASSET_ID, &BOB));
		assert!(!<Locks<Test>>::contains_key(ASSET_ID, &BOB));
	});
}

#[test]
fn purged_dust_move_to_treasury() {
	new_test_ext_with_balance(STAKING_ASSET_ID, ALICE, INITIAL_BALANCE).execute_with(|| {
		let asset_info_1 = AssetInfo::new(b"TST1".to_vec(), 1, 11);
		let asset_info_2 = AssetInfo::new(b"TST2".to_vec(), 4, 7);
		assert_ok!(GenericAsset::create(
			Origin::root(),
			BOB,
			asset_options(PermissionLatest::new(BOB), asset_info_1.decimal_places()),
			asset_info_1.clone()
		));
		assert_ok!(GenericAsset::create(
			Origin::root(),
			BOB,
			asset_options(PermissionLatest::new(BOB), asset_info_2.decimal_places()),
			asset_info_2.clone()
		));

		assert_eq!(GenericAsset::total_issuance(ASSET_ID), INITIAL_ISSUANCE);
		assert_eq!(GenericAsset::total_issuance(ASSET_ID + 1), INITIAL_ISSUANCE);

		assert_ok!(GenericAsset::transfer(
			Origin::signed(BOB),
			ASSET_ID,
			ALICE,
			INITIAL_ISSUANCE - asset_info_1.existential_deposit() + 1
		));
		assert_ok!(GenericAsset::transfer(
			Origin::signed(BOB),
			ASSET_ID + 1,
			ALICE,
			INITIAL_ISSUANCE - asset_info_2.existential_deposit() + 1
		));

		// Test purge has happened
		assert!(!<Test as Config>::AccountStore::get(&BOB).should_exist());

		// TODO Enable the following check after https://github.com/plugblockchain/plug-blockchain/issues/191
		// assert!(!System::account_exists(&BOB));

		assert!(!<FreeBalance<Test>>::contains_key(ASSET_ID, &BOB));
		assert!(!<FreeBalance<Test>>::contains_key(ASSET_ID + 1, &BOB));

		assert_eq!(GenericAsset::total_issuance(ASSET_ID), INITIAL_ISSUANCE);
		assert_eq!(GenericAsset::total_issuance(ASSET_ID + 1), INITIAL_ISSUANCE);

		let treasury_account_id = TreasuryModuleId::get().into_account();
		assert_eq!(
			GenericAsset::free_balance(ASSET_ID, &treasury_account_id),
			asset_info_1.existential_deposit() - 1
		);
		assert_eq!(
			GenericAsset::free_balance(ASSET_ID + 1, &treasury_account_id),
			asset_info_2.existential_deposit() - 1
		);
	});
}

#[test]
fn on_runtime_upgrade() {
	new_test_ext_with_balance(STAKING_ASSET_ID, ALICE, INITIAL_BALANCE).execute_with(|| {
		let asset_info_1 = AssetInfo::new(b"TST1".to_vec(), 1, 11);
		let asset_info_2 = AssetInfo::new(b"TST2".to_vec(), 4, 7);
		assert_ok!(GenericAsset::create(
			Origin::root(),
			BOB,
			asset_options(PermissionLatest::new(BOB), asset_info_1.decimal_places()),
			asset_info_1.clone()
		));
		assert_ok!(GenericAsset::create(
			Origin::root(),
			BOB,
			asset_options(PermissionLatest::new(BOB), asset_info_2.decimal_places()),
			asset_info_2.clone()
		));
		GenericAsset::set_free_balance(ASSET_ID, &BOB, asset_info_1.existential_deposit() - 1);

		// Mess with the account store
		assert_ok!(<Test as Config>::AccountStore::remove(&ALICE));
		assert_ok!(<Test as Config>::AccountStore::remove(&BOB));

		// Make sure accounts are gone
		let alice_account = <Test as Config>::AccountStore::get(&ALICE);
		let bob_account = <Test as Config>::AccountStore::get(&BOB);
		assert!(!alice_account.exists());
		assert!(!bob_account.exists());

		// On runtime upgrade should be able to fix the account store
		let _ = GenericAsset::on_runtime_upgrade();

		// Test accounts are restored now
		let alice_account = <Test as Config>::AccountStore::get(&ALICE);
		let bob_account = <Test as Config>::AccountStore::get(&BOB);
		assert!(alice_account.exists());
		assert!(bob_account.exists());

		// Test assets of Alice are as before
		assert!(alice_account.existing_assets().contains(&STAKING_ASSET_ID));
		assert!(!alice_account.existing_assets().contains(&ASSET_ID));
		assert_eq!(<FreeBalance<Test>>::get(&STAKING_ASSET_ID, &ALICE), INITIAL_BALANCE);

		// Test assets of Bob are as before
		assert!(!bob_account.existing_assets().contains(&STAKING_ASSET_ID));
		assert!(bob_account.existing_assets().contains(&(ASSET_ID + 1)));
		assert_eq!(<FreeBalance<Test>>::get(&(ASSET_ID + 1), &BOB), INITIAL_ISSUANCE);

		// Test BOB's dust ASSET_ID is claimed
		assert!(!bob_account.existing_assets().contains(&ASSET_ID));
		assert!(!<FreeBalance<Test>>::contains_key(ASSET_ID, BOB));
	});
}

#[test]
fn migrate_locks_on_runtime_upgrade() {
	new_test_ext_with_balance(STAKING_ASSET_ID, ALICE, INITIAL_BALANCE).execute_with(|| {
		#[allow(dead_code)]
		mod inner {
			use super::Config;
			use crate::types::BalanceLock;
			pub struct Module<T>(sp_std::marker::PhantomData<T>);
			frame_support::decl_storage! {
				trait Store for Module<T: Config> as GenericAsset {
					pub Locks get(fn locks):
						map hasher(blake2_128_concat) u64 => Vec<BalanceLock<u64>>;
				}
			}
		}

		assert!(!<Locks<Test>>::contains_key(STAKING_ASSET_ID, ALICE));
		assert!(!<Locks<Test>>::contains_key(STAKING_ASSET_ID, BOB));

		let lock_1 = BalanceLock {
			id: ID_1,
			amount: 3u64,
			reasons: WithdrawReasons::TRANSACTION_PAYMENT,
		};
		let lock_2 = BalanceLock {
			id: ID_1,
			amount: 5u64,
			reasons: WithdrawReasons::TRANSFER,
		};
		let lock_3 = BalanceLock {
			id: ID_2,
			amount: 7u64,
			reasons: WithdrawReasons::TIP,
		};
		let alice_locks = vec![lock_1, lock_2, lock_3];
		inner::Locks::insert(ALICE, alice_locks.clone());

		let lock_4 = BalanceLock {
			id: ID_2,
			amount: 11u64,
			reasons: WithdrawReasons::FEE,
		};
		let bob_locks = vec![lock_4];
		inner::Locks::insert(BOB, bob_locks.clone());

		let _ = GenericAsset::on_runtime_upgrade();

		assert_eq!(<Locks<Test>>::get(STAKING_ASSET_ID, ALICE), alice_locks);
		assert_eq!(<Locks<Test>>::get(STAKING_ASSET_ID, BOB), bob_locks);
		assert_eq!(<Locks<Test>>::iter().count(), 2);
	});
}

#[test]
// Test GenericAsset::ensure_can_withdraw which is consulted in other main functions such as `transfer` or `Withdraw`
fn ensure_can_withdraw() {
	new_test_ext_with_balance(STAKING_ASSET_ID, ALICE, INITIAL_BALANCE).execute_with(|| {
		let lock_1 = BalanceLock {
			id: ID_1,
			amount: 3u64,
			reasons: WithdrawReasons::TRANSACTION_PAYMENT,
		};
		let lock_2 = BalanceLock {
			id: ID_1,
			amount: 5u64,
			reasons: WithdrawReasons::TRANSFER,
		};
		let lock_3 = BalanceLock {
			id: ID_2,
			amount: 7u64,
			reasons: WithdrawReasons::TIP,
		};
		let alice_locks = vec![lock_1.clone(), lock_2.clone(), lock_3.clone()];
		<Locks<Test>>::insert(STAKING_ASSET_ID, ALICE, alice_locks.clone());

		// A zero amount is always withdraw-able
		assert_ok!(GenericAsset::ensure_can_withdraw(
			STAKING_ASSET_ID,
			&ALICE,
			0,
			WithdrawReasons::all(),
			0
		));

		// Withdrawal is okay if we leave enough balance
		let alice_max_locked = alice_locks.iter().map(|x| x.amount).max().unwrap();
		assert_ok!(GenericAsset::ensure_can_withdraw(
			STAKING_ASSET_ID,
			&ALICE,
			1,
			WithdrawReasons::all(),
			alice_max_locked
		));
		assert_noop!(
			GenericAsset::ensure_can_withdraw(
				STAKING_ASSET_ID,
				&ALICE,
				1,
				WithdrawReasons::all(),
				alice_max_locked - 1
			),
			Error::<Test>::LiquidityRestrictions
		);

		// Withdrawal is okay if it's for a reason other than the reasons the current locks are created for.
		assert_ok!(GenericAsset::ensure_can_withdraw(
			STAKING_ASSET_ID,
			&ALICE,
			1,
			WithdrawReasons::FEE,
			0
		));

		// Withdrawal conflicts
		alice_locks.iter().for_each(|x| {
			assert_noop!(
				GenericAsset::ensure_can_withdraw(STAKING_ASSET_ID, &ALICE, 1, x.reasons, x.amount - 1),
				Error::<Test>::LiquidityRestrictions
			);
			assert_ok!(GenericAsset::ensure_can_withdraw(
				STAKING_ASSET_ID,
				&ALICE,
				1,
				x.reasons,
				x.amount
			));
		});
	});
}

// Given
// - Next asset id as `asset_id` = 1000.
// - Sufficient free balance.
// - initial balance = 100.
// When
// - After performing a self transfer from account 1 to 1.
// Then
// - Should not throw any errors.
// - Free balance after self transfer should equal to the free balance before self transfer.
#[test]
fn self_transfer_should_unchanged() {
	new_test_ext_with_balance(STAKING_ASSET_ID, ALICE, INITIAL_BALANCE).execute_with(|| {
		let permissions = PermissionLatest::new(ALICE);
		let transfer_ammount = 50;
		let asset_info = AssetInfo::default();

		assert_ok!(GenericAsset::create(
			Origin::root(),
			ALICE,
			asset_options(permissions, asset_info.decimal_places()),
			asset_info
		));
		assert_eq!(GenericAsset::free_balance(ASSET_ID, &ALICE), INITIAL_ISSUANCE);
		assert_ok!(GenericAsset::transfer(
			Origin::signed(ALICE),
			ASSET_ID,
			ALICE,
			transfer_ammount
		));
		assert_eq!(GenericAsset::free_balance(ASSET_ID, &ALICE), INITIAL_ISSUANCE);
		assert_eq!(GenericAsset::total_issuance(ASSET_ID), INITIAL_ISSUANCE);
	});
}

#[test]
fn transferring_more_units_than_total_supply_should_fail() {
	new_test_ext_with_balance(STAKING_ASSET_ID, ALICE, INITIAL_BALANCE).execute_with(|| {
		let permissions = PermissionLatest::new(ALICE);
		let asset_info = AssetInfo::default();

		assert_ok!(GenericAsset::create(
			Origin::root(),
			ALICE,
			asset_options(permissions, asset_info.decimal_places()),
			asset_info
		));
		assert_eq!(GenericAsset::total_issuance(ASSET_ID), INITIAL_ISSUANCE);
		assert_noop!(
			GenericAsset::transfer(Origin::signed(ALICE), ASSET_ID, BOB, INITIAL_ISSUANCE + 1),
			Error::<Test>::InsufficientBalance
		);
	});
}

// Ensures it uses fake money for staking asset id.
#[test]
fn staking_asset_id_should_correct() {
	new_test_ext_with_default().execute_with(|| {
		assert_eq!(GenericAsset::staking_asset_id(), STAKING_ASSET_ID);
	});
}

// Ensures it uses fake money for spending asset id.
#[test]
fn spending_asset_id_should_correct() {
	new_test_ext_with_default().execute_with(|| {
		assert_eq!(GenericAsset::spending_asset_id(), SPENDING_ASSET_ID);
	});
}

// Given
// -Â Free balance is 0 and the reserved balance is 0.
// Then
// -Â total_balance should return 0
#[test]
fn total_balance_should_be_zero() {
	new_test_ext_with_default().execute_with(|| {
		assert_eq!(GenericAsset::total_balance(ASSET_ID, &ALICE), 0);
	});
}

// Given
// -Â Free balance is 100 and the reserved balance 0.
// -Reserved 50
// When
// - After calling total_balance.
// Then
// -Â total_balance should equals to free balance + reserved balance.
#[test]
fn total_balance_should_be_equal_to_account_balance() {
	new_test_ext_with_balance(STAKING_ASSET_ID, ALICE, INITIAL_BALANCE).execute_with(|| {
		let permissions = PermissionLatest::new(ALICE);
		let reserved_amount = 50;
		let asset_info = AssetInfo::default();

		assert_ok!(GenericAsset::create(
			Origin::root(),
			ALICE,
			asset_options(permissions, asset_info.decimal_places()),
			asset_info
		));
		assert_eq!(GenericAsset::free_balance(ASSET_ID, &ALICE), INITIAL_ISSUANCE);
		assert_ok!(GenericAsset::reserve(ASSET_ID, &ALICE, reserved_amount));
		assert_eq!(GenericAsset::reserved_balance(ASSET_ID, &ALICE), reserved_amount);
		assert_eq!(
			GenericAsset::free_balance(ASSET_ID, &ALICE),
			INITIAL_ISSUANCE - reserved_amount
		);
		assert_eq!(GenericAsset::total_balance(ASSET_ID, &ALICE), INITIAL_ISSUANCE);
	});
}

// Given
// - An account presents with AccountId = 1
// -Â free_balance = 100.
// - Set reserved_balance = 50.
// When
// - After calling free_balance.
// Then
// -Â free_balance should return 50.
#[test]
fn free_balance_should_only_return_account_free_balance() {
	new_test_ext_with_balance(ASSET_ID, ALICE, INITIAL_BALANCE).execute_with(|| {
		GenericAsset::set_reserved_balance(ASSET_ID, &ALICE, 50);
		assert_eq!(GenericAsset::free_balance(ASSET_ID, &ALICE), INITIAL_BALANCE);
	});
}

// Given
// - An account presents with AccountId = 1.
// -Â Free balance > 0 and the reserved balance > 0.
// When
// - After calling total_balance.
// Then
// -Â total_balance should equals to account balance + free balance.
#[test]
fn total_balance_should_be_equal_to_sum_of_account_balance_and_free_balance() {
	new_test_ext_with_balance(ASSET_ID, ALICE, INITIAL_BALANCE).execute_with(|| {
		GenericAsset::set_reserved_balance(ASSET_ID, &ALICE, 50);
		assert_eq!(GenericAsset::total_balance(ASSET_ID, &ALICE), INITIAL_BALANCE + 50);
	});
}

// Given
// -Â free_balance > 0.
// - reserved_balance = 70.
// When
// - After calling reserved_balance.
// Then
// - reserved_balance should return 70.
#[test]
fn reserved_balance_should_only_return_account_reserved_balance() {
	new_test_ext_with_balance(ASSET_ID, ALICE, INITIAL_BALANCE).execute_with(|| {
		GenericAsset::set_reserved_balance(ASSET_ID, &ALICE, 70);
		assert_eq!(GenericAsset::reserved_balance(ASSET_ID, &ALICE), 70);
	});
}

// Given
// - A valid account presents.
// - Initial reserved_balance = 0
// When
// - After calls set_reserved_balance
// Then
// - Should persists the amount as reserved_balance.
// - reserved_balance = amount
#[test]
fn set_reserved_balance_should_add_balance_as_reserved() {
	new_test_ext_with_default().execute_with(|| {
		GenericAsset::set_reserved_balance(ASSET_ID, &ALICE, 50);
		assert_eq!(GenericAsset::reserved_balance(ASSET_ID, &ALICE), 50);
	});
}

// Given
// - A valid account presents.
// - Initial free_balance = 100.
// When
// - After calling set_free_balance.
// Then
// - Should persists the amount as free_balance.
// - New free_balance should replace older free_balance.
#[test]
fn set_free_balance_should_add_amount_as_free_balance() {
	new_test_ext_with_balance(ASSET_ID, ALICE, INITIAL_BALANCE).execute_with(|| {
		GenericAsset::set_free_balance(ASSET_ID, &ALICE, 50);
		assert_eq!(GenericAsset::free_balance(ASSET_ID, &ALICE), 50);
	});
}

// Given
// - free_balance is greater than the account balance.
// - free_balance = 100
// - reserved_balance = 0
// - reserve amount = 70
// When
// - After calling reserve
// Then
// - Funds should be removed from the account.
// - new free_balance = original free_balance - reserved amount
// - new reserved_balance = original free balance + reserved amount
#[test]
fn reserve_should_moves_amount_from_balance_to_reserved_balance() {
	new_test_ext_with_balance(ASSET_ID, ALICE, INITIAL_BALANCE).execute_with(|| {
		assert_ok!(GenericAsset::reserve(ASSET_ID, &ALICE, 70));
		assert_eq!(GenericAsset::free_balance(ASSET_ID, &ALICE), INITIAL_BALANCE - 70);
		assert_eq!(GenericAsset::reserved_balance(ASSET_ID, &ALICE), 70);
	});
}

// Given
// - Free balance is lower than the account balance.
// - free_balance = 100
// - reserved_balance = 0
// - reserve amount = 120
// When
// - After calling reverse function.
// Then
// - Funds should not be removed from the account.
// - Should throw an error.
#[test]
fn reserve_should_not_moves_amount_from_balance_to_reserved_balance() {
	new_test_ext_with_balance(ASSET_ID, ALICE, INITIAL_BALANCE).execute_with(|| {
		assert_noop!(
			GenericAsset::reserve(ASSET_ID, &ALICE, INITIAL_BALANCE + 20),
			Error::<Test>::InsufficientBalance
		);
		assert_eq!(GenericAsset::free_balance(ASSET_ID, &ALICE), INITIAL_BALANCE);
		assert_eq!(GenericAsset::reserved_balance(ASSET_ID, &ALICE), 0);
	});
}

// Given
// - unreserved_amount > reserved_balance.
// - reserved_balance = 100.
// - free_balance = 100.
// - unreserved_amount = 120.
// When
// - After calling unreserve function.
// Then
// - unreserved should return 20.
#[test]
fn unreserve_should_return_subtracted_value_from_unreserved_amount_by_actual_account_balance() {
	new_test_ext_with_balance(ASSET_ID, ALICE, INITIAL_BALANCE).execute_with(|| {
		GenericAsset::set_reserved_balance(ASSET_ID, &ALICE, 100);
		assert_eq!(GenericAsset::unreserve(ASSET_ID, &ALICE, 120), 20);
	});
}

// Given
// - unreserved_amount < reserved_balance.
// - reserved_balance = 100.
// - free_balance = 100.
// - unreserved_amount = 50.
// When
// - After calling unreserve function.
// Then
// - unreserved should return None.
#[test]
fn unreserve_should_return_none() {
	new_test_ext_with_balance(ASSET_ID, ALICE, INITIAL_BALANCE).execute_with(|| {
		GenericAsset::set_reserved_balance(ASSET_ID, &ALICE, 100);
		assert_eq!(GenericAsset::unreserve(ASSET_ID, &ALICE, 50), 0);
	});
}

// Given
// - unreserved_amount > reserved_balance.
// - reserved_balance = 100.
// - free_balance = 100.
// - unreserved_amount = 120.
// When
// - After calling unreserve function.
// Then
// - free_balance should be 200.
#[test]
fn unreserve_should_increase_free_balance_by_reserved_balance() {
	new_test_ext_with_balance(ASSET_ID, ALICE, INITIAL_BALANCE).execute_with(|| {
		GenericAsset::set_reserved_balance(ASSET_ID, &ALICE, 100);
		GenericAsset::unreserve(ASSET_ID, &ALICE, 120);
		assert_eq!(GenericAsset::free_balance(ASSET_ID, &ALICE), INITIAL_BALANCE + 100);
	});
}

// Given
// - unreserved_amount > reserved_balance.
// - reserved_balance = 100.
// - free_balance = 100.
// - unreserved_amount = 120.
// When
// - After calling unreserve function.
// Then
// - reserved_balance should be 0.
#[test]
fn unreserve_should_deduct_reserved_balance_by_reserved_amount() {
	new_test_ext_with_balance(ASSET_ID, ALICE, INITIAL_BALANCE).execute_with(|| {
		GenericAsset::unreserve(ASSET_ID, &ALICE, 120);
		assert_eq!(GenericAsset::reserved_balance(ASSET_ID, &ALICE), 0);
	});
}

// Given
// - slash amount < free_balance.
// - reserved_balance = 100.
// - free_balance = 100.
// - slash amount = 70.
// When
// - After calling slash function.
// Then
// - slash should return None.
#[test]
fn slash_should_return_slash_reserved_amount() {
	new_test_ext_with_balance(ASSET_ID, ALICE, INITIAL_BALANCE).execute_with(|| {
		let reserved_amount = 100;
		let slash_amount = 70;
		GenericAsset::set_reserved_balance(ASSET_ID, &ALICE, reserved_amount);
		assert_eq!(GenericAsset::slash(ASSET_ID, &ALICE, slash_amount), None);
		assert_eq!(
			GenericAsset::free_balance(ASSET_ID, &ALICE),
			INITIAL_BALANCE - slash_amount
		);
		assert_eq!(
			GenericAsset::total_balance(ASSET_ID, &ALICE),
			INITIAL_BALANCE + reserved_amount - slash_amount
		);
	});
}

// Given
// - slashed_amount > reserved_balance.
// When
// - After calling slashed_reverse function.
// Then
// - Should return slashed_reserved - reserved_balance.
#[test]
fn slash_reserved_should_deducts_up_to_amount_from_reserved_balance() {
	new_test_ext_with_default().execute_with(|| {
		GenericAsset::set_reserved_balance(ASSET_ID, &ALICE, 100);
		assert_eq!(GenericAsset::slash_reserved(ASSET_ID, &ALICE, 150), Some(50));
		assert_eq!(GenericAsset::reserved_balance(ASSET_ID, &ALICE), 0);
	});
}

// Given
// - slashed_amount equals to reserved_amount.
// When
// - After calling slashed_reverse function.
// Then
// - Should return None.
#[test]
fn slash_reserved_should_return_none() {
	new_test_ext_with_default().execute_with(|| {
		GenericAsset::set_reserved_balance(ASSET_ID, &ALICE, 100);
		assert_eq!(GenericAsset::slash_reserved(ASSET_ID, &ALICE, 100), None);
		assert_eq!(GenericAsset::reserved_balance(ASSET_ID, &ALICE), 0);
	});
}

// Given
// - reserved_balance = 100.
// - repatriate_reserved_amount > reserved_balance.
// When
// - After calling repatriate_reserved.
// Then
// - Should return `remaining`.
#[test]
fn repatriate_reserved_return_amount_subtracted_by_slash_amount() {
	new_test_ext_with_default().execute_with(|| {
		GenericAsset::set_reserved_balance(ASSET_ID, &ALICE, 100);
		assert_ok!(GenericAsset::repatriate_reserved(ASSET_ID, &ALICE, &ALICE, 130), 30);
		assert_eq!(GenericAsset::free_balance(ASSET_ID, &ALICE), 100);
	});
}

// Given
// - reserved_balance = 100.
// - repatriate_reserved_amount < reserved_balance.
// When
// - After calling repatriate_reserved.
// Then
// - Should return zero.
#[test]
fn repatriate_reserved_return_none() {
	new_test_ext_with_default().execute_with(|| {
		GenericAsset::set_reserved_balance(ASSET_ID, &ALICE, 100);
		assert_ok!(GenericAsset::repatriate_reserved(ASSET_ID, &ALICE, &ALICE, 90), 0);
		assert_eq!(GenericAsset::reserved_balance(ASSET_ID, &ALICE), 10);
		assert_eq!(GenericAsset::free_balance(ASSET_ID, &ALICE), 90);
	});
}

// Given
// - An asset with all permissions
// When
// - After calling `create_reserved` function.
// Then
// - Should create a new reserved asset.
#[test]
fn create_reserved_should_create_a_default_account_with_the_balance_given() {
	new_test_ext_with_next_asset_id(1001).execute_with(|| {
		let permissions = PermissionLatest::new(ALICE);
		let asset_info = AssetInfo::default();
		let options = asset_options(permissions, asset_info.decimal_places());

		assert_ok!(GenericAsset::create_reserved(
			Origin::root(),
			ASSET_ID,
			options,
			asset_info
		));
		assert_eq!(<TotalIssuance<Test>>::get(ASSET_ID), INITIAL_ISSUANCE);
		assert_eq!(<FreeBalance<Test>>::get(&ASSET_ID, &0), INITIAL_ISSUANCE);
	});
}

#[test]
fn create_reserved_with_non_reserved_asset_id_should_failed() {
	new_test_ext_with_next_asset_id(999).execute_with(|| {
		let permissions = PermissionLatest::new(ALICE);
		let asset_info = AssetInfo::default();
		let options = asset_options(permissions, asset_info.decimal_places());

		// create reserved asset with asset_id >= next_asset_id should fail
		assert_noop!(
			GenericAsset::create_reserved(Origin::root(), ASSET_ID, options.clone(), asset_info),
			Error::<Test>::AssetIdExists,
		);
	});
}

#[test]
fn create_reserved_with_a_taken_asset_id_should_failed() {
	new_test_ext_with_next_asset_id(1001).execute_with(|| {
		let permissions = PermissionLatest::new(ALICE);
		let asset_info = AssetInfo::default();
		let options = asset_options(permissions, asset_info.decimal_places());

		// create reserved asset with asset_id < next_asset_id should success
		assert_ok!(GenericAsset::create_reserved(
			Origin::root(),
			ASSET_ID,
			options.clone(),
			asset_info.clone()
		));
		assert_eq!(<TotalIssuance<Test>>::get(ASSET_ID), INITIAL_ISSUANCE);
		// all reserved assets belong to account: 0 which is the default value of `AccountId`
		assert_eq!(<FreeBalance<Test>>::get(&ASSET_ID, &0), INITIAL_ISSUANCE);
		// create reserved asset with existing asset_id: 9 should fail
		assert_noop!(
			GenericAsset::create_reserved(Origin::root(), ASSET_ID, options.clone(), asset_info),
			Error::<Test>::AssetIdExists,
		);
	});
}

// Given
// - ALICE is signed
// - ALICE does not have minting permission
// When
// - After calling mint function
// Then
// - Should throw a permission error
#[test]
fn mint_without_permission_should_throw_error() {
	new_test_ext_with_default().execute_with(|| {
		let amount = 100;

		assert_noop!(
			GenericAsset::mint(Origin::signed(ALICE), ASSET_ID, BOB, amount),
			Error::<Test>::NoMintPermission,
		);
	});
}

// Given
// - ALICE is signed.
// - ALICE has permissions.
// When
// - After calling mint function
// Then
// - Should increase `BOB` free_balance.
// - Should not change `origins` free_balance.
#[test]
fn mint_should_increase_asset() {
	new_test_ext_with_balance(STAKING_ASSET_ID, ALICE, INITIAL_BALANCE).execute_with(|| {
		let permissions = PermissionLatest::new(ALICE);
		let amount = 100;
		let asset_info = AssetInfo::default();

		assert_ok!(GenericAsset::create(
			Origin::root(),
			ALICE,
			asset_options(permissions, asset_info.decimal_places()),
			asset_info
		));
		assert_ok!(GenericAsset::mint(Origin::signed(ALICE), ASSET_ID, BOB, amount));
		assert_eq!(GenericAsset::free_balance(ASSET_ID, &BOB), amount);
		// Origin's free_balance should not change.
		assert_eq!(GenericAsset::free_balance(ASSET_ID, &ALICE), INITIAL_ISSUANCE);
		assert_eq!(GenericAsset::total_issuance(ASSET_ID), INITIAL_ISSUANCE + amount);
	});
}

// Given
// - Origin is signed.
// - Origin does not have burning permission.
// When
// - After calling burn function.
// Then
// - Should throw a permission error.
#[test]
fn burn_should_throw_permission_error() {
	new_test_ext_with_balance(STAKING_ASSET_ID, ALICE, INITIAL_BALANCE).execute_with(|| {
		let amount = 100;

		assert_noop!(
			GenericAsset::burn(Origin::signed(ALICE), ASSET_ID, BOB, amount),
			Error::<Test>::NoBurnPermission,
		);
	});
}

// Given
// - Origin is signed.
// - Origin has permissions.
// When
// - After calling burn function
// Then
// - Should decrease `to`'s  free_balance.
// - Should not change `origin`'s  free_balance.
#[test]
fn burn_should_burn_an_asset() {
	new_test_ext_with_balance(STAKING_ASSET_ID, ALICE, INITIAL_BALANCE).execute_with(|| {
		let permissions = PermissionLatest::new(ALICE);
		let mint_amount = 100;
		let burn_amount = 40;
		let asset_info = AssetInfo::default();

		assert_ok!(GenericAsset::create(
			Origin::root(),
			ALICE,
			asset_options(permissions, asset_info.decimal_places()),
			asset_info
		));
		assert_ok!(GenericAsset::mint(Origin::signed(ALICE), ASSET_ID, BOB, mint_amount));
		assert_eq!(GenericAsset::total_issuance(ASSET_ID), INITIAL_ISSUANCE + mint_amount);

		assert_ok!(GenericAsset::burn(Origin::signed(ALICE), ASSET_ID, BOB, burn_amount));
		assert_eq!(GenericAsset::free_balance(ASSET_ID, &BOB), mint_amount - burn_amount);
		assert_eq!(
			GenericAsset::total_issuance(ASSET_ID),
			INITIAL_ISSUANCE + mint_amount - burn_amount
		);
	});
}

// Given
// - `default_permissions` with all privileges.
// - All permissions for origin.
// When
// - After executing create function and check_permission function.
// Then
// - The account origin should have burn, mint and update permissions.
#[test]
fn check_permission_should_return_correct_permission() {
	new_test_ext_with_balance(STAKING_ASSET_ID, ALICE, INITIAL_BALANCE).execute_with(|| {
		let permissions = PermissionLatest::new(ALICE);
		let asset_info = AssetInfo::default();

		assert_ok!(GenericAsset::create(
			Origin::root(),
			ALICE,
			asset_options(permissions, asset_info.decimal_places()),
			asset_info
		));
		assert!(GenericAsset::check_permission(ASSET_ID, &ALICE, &PermissionType::Burn));
		assert!(GenericAsset::check_permission(ASSET_ID, &ALICE, &PermissionType::Mint));
		assert!(GenericAsset::check_permission(
			ASSET_ID,
			&ALICE,
			&PermissionType::Update
		));
	});
}

// Given
// - `default_permissions` with no privileges.
// - No permissions for origin.
// When
// - After executing create function and check_permission function.
// Then
// - The account origin should not have burn, mint and update permissions.
#[test]
fn check_permission_should_return_false_for_no_permission() {
	new_test_ext_with_balance(STAKING_ASSET_ID, ALICE, INITIAL_BALANCE).execute_with(|| {
		let permissions = PermissionLatest::default();
		let asset_info = AssetInfo::default();

		assert_ok!(GenericAsset::create(
			Origin::root(),
			ALICE,
			asset_options(permissions, asset_info.decimal_places()),
			asset_info
		));
		assert!(!GenericAsset::check_permission(ASSET_ID, &ALICE, &PermissionType::Burn));
		assert!(!GenericAsset::check_permission(ASSET_ID, &ALICE, &PermissionType::Mint));
		assert!(!GenericAsset::check_permission(
			ASSET_ID,
			&ALICE,
			&PermissionType::Update
		));
	});
}

// Given
// - `default_permissions` only with update.
// When
// - After executing update_permission function.
// Then
// - The account origin should not have the burn permission.
// - The account origin should have update and mint permissions.
#[test]
fn update_permission_should_change_permission() {
	new_test_ext_with_balance(STAKING_ASSET_ID, ALICE, INITIAL_BALANCE).execute_with(|| {
		let permissions = PermissionLatest {
			update: Owner::Address(ALICE),
			mint: Owner::None,
			burn: Owner::None,
		};

		let new_permission = PermissionLatest {
			update: Owner::Address(ALICE),
			mint: Owner::Address(ALICE),
			burn: Owner::None,
		};
		let asset_info = AssetInfo::default();

		assert_ok!(GenericAsset::create(
			Origin::root(),
			ALICE,
			asset_options(permissions, asset_info.decimal_places()),
			asset_info
		));
		assert_ok!(GenericAsset::update_permission(
			Origin::signed(ALICE),
			ASSET_ID,
			new_permission
		));
		assert!(GenericAsset::check_permission(ASSET_ID, &ALICE, &PermissionType::Mint));
		assert!(!GenericAsset::check_permission(ASSET_ID, &ALICE, &PermissionType::Burn));
	});
}

// Given
// - `default_permissions` without any permissions.
// When
// - After executing update_permission function.
// Then
// - Should throw an error stating "Origin does not have enough permission to update permissions."
#[test]
fn update_permission_should_throw_error_when_lack_of_permissions() {
	new_test_ext_with_balance(STAKING_ASSET_ID, ALICE, INITIAL_BALANCE).execute_with(|| {
		let permissions = PermissionLatest::default();

		let new_permission = PermissionLatest {
			update: Owner::Address(ALICE),
			mint: Owner::Address(ALICE),
			burn: Owner::None,
		};
		let asset_info = AssetInfo::default();

		assert_ok!(GenericAsset::create(
			Origin::root(),
			ALICE,
			asset_options(permissions, asset_info.decimal_places()),
			asset_info
		));
		assert_noop!(
			GenericAsset::update_permission(Origin::signed(ALICE), ASSET_ID, new_permission),
			Error::<Test>::NoUpdatePermission,
		);
	});
}

// Given
// - `asset_id` provided.
// - `from_account` is present.
// - All permissions for origin.
// When
// - After calling create_asset.
// Then
// - Should create a reserved token with provided id.
// - NextAssetId doesn't change.
// - TotalIssuance must equal to initial issuance.
// - FreeBalance must equal to initial issuance for the given account.
// - Permissions must have burn, mint and updatePermission for the given asset_id.
#[test]
fn create_asset_works_with_given_asset_id_and_from_account() {
	new_test_ext_with_next_asset_id(1001).execute_with(|| {
		let from_account: Option<<Test as frame_system::Config>::AccountId> = Some(ALICE);
		let permissions = PermissionLatest::new(ALICE);
		let expected_permission = PermissionVersions::V1(permissions.clone());
		let asset_info = AssetInfo::default();

		assert_ok!(GenericAsset::create_asset(
			Some(ASSET_ID),
			from_account,
			asset_options(permissions, asset_info.decimal_places()),
			asset_info
		));
		// Test for side effects.
		assert_eq!(<NextAssetId<Test>>::get(), 1001);
		assert_eq!(<TotalIssuance<Test>>::get(ASSET_ID), INITIAL_ISSUANCE);
		assert_eq!(<FreeBalance<Test>>::get(&ASSET_ID, &ALICE), INITIAL_ISSUANCE);
		assert_eq!(<Permissions<Test>>::get(&ASSET_ID), expected_permission);
	});
}

// Given
// - `asset_id` is an id for user generated assets.
// - Whatever other params.
// Then
// - `create_asset` should not work.
#[test]
fn create_asset_with_non_reserved_asset_id_should_fail() {
	new_test_ext_with_next_asset_id(999).execute_with(|| {
		let permissions = PermissionLatest::new(ALICE);
		let asset_info = AssetInfo::default();

		assert_noop!(
			GenericAsset::create_asset(
				Some(ASSET_ID),
				Some(ALICE),
				asset_options(permissions, asset_info.decimal_places()),
				asset_info
			),
			Error::<Test>::AssetIdExists,
		);
	});
}

// Given
// - `asset_id` is for reserved assets, but already taken.
// - Whatever other params.
// Then
// - `create_asset` should not work.
#[test]
fn create_asset_with_a_taken_asset_id_should_fail() {
	new_test_ext_with_next_asset_id(1001).execute_with(|| {
		let permissions = PermissionLatest::new(ALICE);

		assert_ok!(GenericAsset::create_asset(
			Some(ASSET_ID),
			Some(ALICE),
			asset_options(permissions.clone(), 4),
			AssetInfo::default()
		));
		assert_noop!(
			GenericAsset::create_asset(
				Some(ASSET_ID),
				Some(ALICE),
				asset_options(permissions, 4),
				AssetInfo::default()
			),
			Error::<Test>::AssetIdExists,
		);
	});
}

#[test]
fn create_asset_with_zero_existential_deposit_should_fail() {
	new_test_ext_with_next_asset_id(1001).execute_with(|| {
		let permissions = PermissionLatest::new(ALICE);
		assert_noop!(
			GenericAsset::create_asset(
				Some(ASSET_ID),
				Some(ALICE),
				asset_options(permissions, 4),
				AssetInfo::new(b"TST1".to_vec(), 1, 0)
			),
			Error::<Test>::ZeroExistentialDeposit,
		);
	});
}

// Given
// - `asset_id` provided.
// - `from_account` is None.
// - All permissions for origin.
// When
// - After calling create_asset.
// Then
// - Should create a reserved token.
#[test]
fn create_asset_should_create_a_reserved_asset_when_from_account_is_none() {
	new_test_ext_with_next_asset_id(1001).execute_with(|| {
		let from_account: Option<<Test as frame_system::Config>::AccountId> = None;
		let permissions = PermissionLatest::new(ALICE);
		let created_account_id = 0;
		let asset_info = AssetInfo::default();

		assert_ok!(GenericAsset::create_asset(
			Some(ASSET_ID),
			from_account,
			asset_options(permissions.clone(), asset_info.decimal_places()),
			asset_info
		));

		// Test for a side effect.
		assert_eq!(
			<FreeBalance<Test>>::get(&ASSET_ID, &created_account_id),
			INITIAL_ISSUANCE
		);
	});
}

// Given
// - `asset_id` not provided.
// - `from_account` is None.
// - All permissions for origin.
// When
// - After calling create_asset.
// Then
// - Should create a user token.
// - `NextAssetId`'s get should return a new value.
// - Should not create a `reserved_asset`.
#[test]
fn create_asset_should_create_a_user_asset() {
	new_test_ext_with_default().execute_with(|| {
		let from_account: Option<<Test as frame_system::Config>::AccountId> = None;
		let permissions = PermissionLatest::new(ALICE);
		let reserved_asset_id = 1001;
		let asset_info = AssetInfo::default();

		assert_ok!(GenericAsset::create_asset(
			None,
			from_account,
			asset_options(permissions, asset_info.decimal_places()),
			asset_info
		));

		// Test for side effects.
		assert_eq!(<FreeBalance<Test>>::get(&reserved_asset_id, &ALICE), 0);
		assert_eq!(<FreeBalance<Test>>::get(&ASSET_ID, &0), INITIAL_ISSUANCE);
		assert_eq!(<TotalIssuance<Test>>::get(ASSET_ID), INITIAL_ISSUANCE);
	});
}

#[test]
fn create_asset_with_big_decimal_place_should_fail() {
	new_test_ext_with_default().execute_with(|| {
		let from_account: Option<<Test as frame_system::Config>::AccountId> = None;
		let permissions = PermissionLatest::new(ALICE);
		let reserved_asset_id = 1001;
		let asset_info = AssetInfo::new(b"WEB3.0".to_vec(), 40, 7);

		assert_noop!(
			GenericAsset::create_asset(
				None,
				from_account,
				asset_options(permissions, asset_info.decimal_places()),
				asset_info
			),
			Error::<Test>::DecimalTooLarge
		);
	});
}

#[test]
fn create_asset_should_add_decimal_places() {
	new_test_ext_with_balance(STAKING_ASSET_ID, ALICE, INITIAL_BALANCE).execute_with(|| {
		let web3_asset_info = AssetInfo::new(b"WEB3.0".to_vec(), 0, 7);

		// Should succeed and set ALICE as the owner of ASSET_ID
		assert_ok!(GenericAsset::create(
			Origin::root(),
			ALICE,
			asset_options(PermissionLatest::new(ALICE), web3_asset_info.decimal_places()),
			web3_asset_info.clone()
		));

		// Should return the same info as ALICE set for the asset while creating it
		assert_eq!(<AssetMeta<Test>>::get(ASSET_ID), web3_asset_info);
		assert_eq!(GenericAsset::total_issuance(&ASSET_ID), INITIAL_ISSUANCE);

		let web3_asset_info = AssetInfo::new(b"WEB3.1".to_vec(), 18, 11);
		// Should succeed as ALICE is the owner of this asset
		assert_ok!(GenericAsset::update_asset_info(
			Origin::signed(ALICE),
			ASSET_ID,
			web3_asset_info.clone(),
		));

		assert_eq!(<AssetMeta<Test>>::get(ASSET_ID), web3_asset_info);
		assert_eq!(GenericAsset::total_issuance(&ASSET_ID), INITIAL_ISSUANCE);

		let web3_asset_info = AssetInfo::new(b"WEB3.2".to_vec(), 4, 11);
		// Should succeed as ALICE is the owner of this asset
		assert_ok!(GenericAsset::update_asset_info(
			Origin::signed(ALICE),
			ASSET_ID,
			web3_asset_info.clone(),
		));

		assert_eq!(<AssetMeta<Test>>::get(ASSET_ID), web3_asset_info);
		assert_eq!(GenericAsset::total_issuance(ASSET_ID), INITIAL_ISSUANCE);
	});
}

#[test]
fn update_permission_should_raise_event() {
	new_test_ext_with_balance(STAKING_ASSET_ID, ALICE, INITIAL_BALANCE).execute_with(|| {
		System::set_block_number(1);

		let permissions = PermissionLatest::new(ALICE);
		let asset_info = AssetInfo::default();

		assert_ok!(GenericAsset::create(
			Origin::root(),
			ALICE,
			asset_options(permissions.clone(), asset_info.decimal_places()),
			asset_info
		));
		assert_ok!(GenericAsset::update_permission(
			Origin::signed(ALICE),
			ASSET_ID,
			permissions.clone()
		));

		let expected_event = TestEvent::prml_generic_asset(RawEvent::PermissionUpdated(ASSET_ID, permissions));
		assert!(System::events().iter().any(|record| record.event == expected_event));
	});
}

#[test]
fn mint_should_raise_event() {
	new_test_ext_with_balance(STAKING_ASSET_ID, ALICE, INITIAL_BALANCE).execute_with(|| {
		System::set_block_number(1);

		let permissions = PermissionLatest::new(ALICE);
		let amount = 100;
		let asset_info = AssetInfo::default();

		assert_ok!(GenericAsset::create(
			Origin::root(),
			ALICE,
			asset_options(permissions, asset_info.decimal_places()),
			asset_info
		));
		assert_ok!(GenericAsset::mint(Origin::signed(ALICE), ASSET_ID, BOB, amount));

		let expected_event = TestEvent::prml_generic_asset(RawEvent::Minted(ASSET_ID, BOB, amount));
		assert!(System::events().iter().any(|record| record.event == expected_event));
	});
}

#[test]
fn burn_should_raise_event() {
	new_test_ext_with_balance(STAKING_ASSET_ID, ALICE, INITIAL_BALANCE).execute_with(|| {
		System::set_block_number(1);

		let permissions = PermissionLatest::new(ALICE);
		let amount = 100;
		let asset_info = AssetInfo::default();

		assert_ok!(GenericAsset::create(
			Origin::root(),
			ALICE,
			asset_options(permissions, asset_info.decimal_places()),
			asset_info
		));
		assert_ok!(GenericAsset::burn(Origin::signed(ALICE), ASSET_ID, ALICE, amount));

		let expected_event = TestEvent::prml_generic_asset(RawEvent::Burned(ASSET_ID, ALICE, amount));
		assert!(System::events().iter().any(|record| record.event == expected_event));
	});
}

#[test]
fn can_set_asset_owner_permissions_in_genesis() {
	new_test_ext_with_permissions(vec![(ASSET_ID, ALICE)]).execute_with(|| {
		let expected: PermissionVersions<_> = PermissionsV1::new(ALICE).into();
		let actual = GenericAsset::get_permission(ASSET_ID);
		assert_eq!(expected, actual);
	});
}

#[test]
fn zero_asset_id_should_updated_after_negative_imbalance_operations() {
	let asset_id = 16000;
	new_test_ext_with_default().execute_with(|| {
		// generate empty negative imbalance
		let negative_im = NegativeImbalanceOf::zero();
		let other = NegativeImbalanceOf::new(100, asset_id);
		assert_eq!(negative_im.asset_id(), 0);
		assert_eq!(negative_im.peek(), 0);
		assert_eq!(other.asset_id(), asset_id);
		// zero asset id should updated after merge
		let merged_im = negative_im.checked_merge(other).unwrap();
		assert_eq!(merged_im.asset_id(), asset_id);
		assert_eq!(merged_im.peek(), 100);

		let negative_im = NegativeImbalanceOf::new(100, asset_id);
		let other = NegativeImbalanceOf::new(100, asset_id);
		// If assets are same, the amount can be merged safely
		let merged_im = negative_im.checked_merge(other).unwrap();
		assert_eq!(merged_im.asset_id(), asset_id);
		assert_eq!(merged_im.peek(), 200);

		// merge other with same asset id should work
		let other = NegativeImbalanceOf::new(100, asset_id);
		let merged_im = merged_im.checked_merge(other).unwrap();
		assert_eq!(merged_im.peek(), 300);

		let mut negative_im = NegativeImbalanceOf::zero();
		assert_eq!(negative_im.asset_id(), 0);
		let other = NegativeImbalanceOf::new(100, asset_id);
		// zero asset id should updated after subsume
		negative_im.checked_subsume(other).unwrap();
		assert_eq!(negative_im.asset_id(), asset_id);
		assert_eq!(negative_im.peek(), 100);

		negative_im = NegativeImbalanceOf::new(100, asset_id);
		// subsume other with same asset id should work
		let other = NegativeImbalanceOf::new(100, asset_id);
		negative_im.checked_subsume(other).unwrap();
		assert_eq!(negative_im.peek(), 200);

		// offset opposite im with same asset id should work
		let offset_im = NegativeImbalanceOf::new(100, asset_id);
		let opposite_im = PositiveImbalanceOf::new(25, asset_id);
		let offset_im = offset_im.checked_offset(opposite_im);
		assert!(offset_im.is_ok());
	});
}

#[test]
fn zero_asset_id_should_updated_after_positive_imbalance_operations() {
	let asset_id = 16000;
	new_test_ext_with_default().execute_with(|| {
		// generate empty positive imbalance
		let positive_im = PositiveImbalanceOf::zero();
		let other = PositiveImbalanceOf::new(100, asset_id);
		assert_eq!(positive_im.asset_id(), 0);
		assert_eq!(positive_im.peek(), 0);
		// zero asset id should updated after merge
		let merged_im = positive_im.checked_merge(other).unwrap();
		assert_eq!(merged_im.asset_id(), asset_id);
		assert_eq!(merged_im.peek(), 100);

		let positive_im = PositiveImbalanceOf::new(10, asset_id);
		let other = PositiveImbalanceOf::new(100, asset_id);
		// If assets are same, the amount can be merged safely
		let merged_im = positive_im.checked_merge(other).unwrap();
		assert_eq!(merged_im.asset_id(), asset_id);
		assert_eq!(merged_im.peek(), 110);

		let other = PositiveImbalanceOf::new(100, asset_id);
		let merged_im = merged_im.checked_merge(other).unwrap();
		assert_eq!(merged_im.peek(), 210);

		// subsume
		let mut positive_im = PositiveImbalanceOf::zero();
		let other = PositiveImbalanceOf::new(100, asset_id);
		// zero asset id should updated after subsume
		positive_im.checked_subsume(other).unwrap();
		assert_eq!(positive_im.asset_id(), asset_id);
		assert_eq!(positive_im.peek(), 100);

		positive_im = PositiveImbalanceOf::new(100, asset_id);
		// subsume other with same asset id should work
		let other = PositiveImbalanceOf::new(100, asset_id);
		positive_im.checked_subsume(other).unwrap();
		assert_eq!(positive_im.peek(), 200);

		let positive_im = PositiveImbalanceOf::new(100, asset_id);
		let opposite_im = NegativeImbalanceOf::new(150, asset_id);
		assert_ok!(positive_im.checked_offset(opposite_im));

		// offset opposite im with same asset id should work
		let offset_im = PositiveImbalanceOf::new(100, asset_id);
		let opposite_im = NegativeImbalanceOf::new(25, asset_id);
		assert_ok!(offset_im.checked_offset(opposite_im));
	});
}

#[test]
fn negative_imbalance_merge_with_incompatible_asset_id_should_fail() {
	new_test_ext_with_default().execute_with(|| {
		// create two mew imbalances with different asset id
		let negative_im = NegativeImbalanceOf::new(100, 1);
		let other = NegativeImbalanceOf::new(50, 2);
		assert_eq!(
			negative_im.checked_merge(other).unwrap_err(),
			imbalances::Error::DifferentAssetIds,
		);
		let negative_im = NegativeImbalanceOf::new(100, 0);
		let other = NegativeImbalanceOf::new(50, 2);
		assert_eq!(
			negative_im.checked_merge(other).unwrap_err(),
			imbalances::Error::ZeroIdWithNonZeroAmount,
		);
	});
}

#[test]
fn positive_imbalance_merge_with_incompatible_asset_id_should_fail() {
	new_test_ext_with_default().execute_with(|| {
		// create two mew imbalances with different asset id
		let positive_im = PositiveImbalanceOf::new(100, 1);
		let other = PositiveImbalanceOf::new(50, 2);
		// merge
		assert_eq!(
			positive_im.checked_merge(other).unwrap_err(),
			imbalances::Error::DifferentAssetIds,
		);
		let positive_im = PositiveImbalanceOf::new(100, 0);
		let other = PositiveImbalanceOf::new(50, 2);
		assert_eq!(
			positive_im.checked_merge(other).unwrap_err(),
			imbalances::Error::ZeroIdWithNonZeroAmount,
		);
	});
}

#[test]
fn negative_imbalance_subsume_with_incompatible_asset_id_should_fail() {
	new_test_ext_with_default().execute_with(|| {
		// create two mew imbalances with different asset id
		let mut negative_im = NegativeImbalanceOf::new(100, 1);
		let other = NegativeImbalanceOf::new(50, 2);
		// subsume
		assert_eq!(
			negative_im.checked_subsume(other).unwrap_err(),
			imbalances::Error::DifferentAssetIds,
		);
		negative_im = NegativeImbalanceOf::new(10, 0);
		let other = NegativeImbalanceOf::new(50, 2);
		// subsume
		assert_eq!(
			negative_im.checked_subsume(other).unwrap_err(),
			imbalances::Error::ZeroIdWithNonZeroAmount,
		);
	});
}

#[test]
fn positive_imbalance_subsume_with_incompatible_asset_id_should_fail() {
	new_test_ext_with_default().execute_with(|| {
		// create two mew imbalances with different asset id
		let mut positive_im = PositiveImbalanceOf::new(100, 1);
		let other = PositiveImbalanceOf::new(50, 2);
		// subsume
		assert_eq!(
			positive_im.checked_subsume(other).unwrap_err(),
			imbalances::Error::DifferentAssetIds,
		);
		positive_im = PositiveImbalanceOf::new(100, 0);
		let other = PositiveImbalanceOf::new(50, 2);
		// subsume
		assert_eq!(
			positive_im.checked_subsume(other).unwrap_err(),
			imbalances::Error::ZeroIdWithNonZeroAmount,
		);
	});
}

#[test]
fn negative_imbalance_offset_with_incompatible_asset_id_should_fail() {
	new_test_ext_with_default().execute_with(|| {
		// create two mew imbalances with different asset id
		let negative_im = NegativeImbalanceOf::new(100, 1);
		let opposite_im = PositiveImbalanceOf::new(50, 2);
		assert_eq!(
			negative_im.checked_offset(opposite_im).unwrap_err(),
			imbalances::Error::DifferentAssetIds,
		);
		let negative_im = NegativeImbalanceOf::new(100, 0);
		let opposite_im = PositiveImbalanceOf::new(50, 2);
		assert_eq!(
			negative_im.checked_offset(opposite_im).unwrap_err(),
			imbalances::Error::ZeroIdWithNonZeroAmount,
		);
	});
}

#[test]
fn positive_imbalance_offset_with_incompatible_asset_id_should_fail() {
	new_test_ext_with_default().execute_with(|| {
		// create two mew imbalances with different asset id
		let positive_im = PositiveImbalanceOf::new(100, 1);
		let opposite_im = NegativeImbalanceOf::new(50, 2);
		assert_eq!(
			positive_im.checked_offset(opposite_im).unwrap_err(),
			imbalances::Error::DifferentAssetIds,
		);
		let positive_im = PositiveImbalanceOf::new(100, 0);
		let opposite_im = NegativeImbalanceOf::new(50, 2);
		assert_eq!(
			positive_im.checked_offset(opposite_im).unwrap_err(),
			imbalances::Error::ZeroIdWithNonZeroAmount,
		);
	});
}

#[test]
fn total_issuance_should_update_after_positive_imbalance_dropped() {
	let asset_id = 16000;
	let balance = 100000;
	new_test_ext_with_balance(asset_id, 1, balance).execute_with(|| {
		assert_eq!(GenericAsset::total_issuance(&asset_id), balance);
		// generate empty positive imbalance
		let positive_im = PositiveImbalanceOf::new(0, asset_id);
		let other = PositiveImbalanceOf::new(100, asset_id);
		// merge
		let merged_im = positive_im.checked_merge(other);
		// explitically drop `imbalance` so issuance is managed
		drop(merged_im);
		assert_eq!(GenericAsset::total_issuance(&asset_id), balance + 100);
	});
}

#[test]
fn total_issuance_should_update_after_negative_imbalance_dropped() {
	let asset_id = 16000;
	let balance = 100000;
	new_test_ext_with_balance(asset_id, 1, balance).execute_with(|| {
		assert_eq!(GenericAsset::total_issuance(&asset_id), balance);
		// generate empty positive imbalance
		let positive_im = NegativeImbalanceOf::new(0, asset_id);
		let other = NegativeImbalanceOf::new(100, asset_id);
		// merge
		let merged_im = positive_im.checked_merge(other);
		// explitically drop `imbalance` so issuance is managed
		drop(merged_im);
		assert_eq!(GenericAsset::total_issuance(&asset_id), balance - 100);
	});
}

#[test]
fn query_pre_existing_asset_info() {
	new_test_ext_with_balance(STAKING_ASSET_ID, ALICE, INITIAL_BALANCE).execute_with(|| {
		assert_eq!(
			GenericAsset::registered_assets(),
			vec![
				(TEST1_ASSET_ID, AssetInfo::new(b"TST1".to_vec(), 1, 3)),
				(TEST2_ASSET_ID, AssetInfo::new(b"TST 2".to_vec(), 2, 5)),
				(STAKING_ASSET_ID, AssetInfo::default()),
			]
		);
	});
}

#[test]
fn no_asset_info() {
	new_test_ext_with_balance(STAKING_ASSET_ID, ALICE, INITIAL_BALANCE).execute_with(|| {
		// Asset STAKING_ASSET_ID exists but no info is stored for that
		assert_eq!(<AssetMeta<Test>>::get(STAKING_ASSET_ID), AssetInfo::default());
		// Asset STAKING_ASSET_ID doesn't exist
		assert!(!<AssetMeta<Test>>::contains_key(ASSET_ID));
	});
}

#[test]
fn non_owner_not_permitted_update_asset_info() {
	new_test_ext_with_balance(STAKING_ASSET_ID, ALICE, INITIAL_BALANCE).execute_with(|| {
		let web3_asset_info = AssetInfo::new(b"WEB3.0".to_vec(), 3, 7);

		// Should fail as ASSET_ID doesn't exist
		assert_noop!(
			GenericAsset::update_asset_info(Origin::signed(ALICE), ASSET_ID, web3_asset_info.clone()),
			Error::<Test>::AssetIdNotExist
		);

		// Should fail as ALICE hasn't got the permission to update this asset's info
		assert_noop!(
			GenericAsset::update_asset_info(Origin::signed(ALICE), STAKING_ASSET_ID, web3_asset_info,),
			Error::<Test>::NoUpdatePermission
		);
	});
}

#[test]
fn owner_update_asset_info() {
	new_test_ext_with_balance(STAKING_ASSET_ID, ALICE, INITIAL_BALANCE).execute_with(|| {
		let web3_asset_info = AssetInfo::new(b"WEB3.0".to_vec(), 3, 7);

		// Should succeed and set ALICE as the owner of ASSET_ID
		assert_ok!(GenericAsset::create(
			Origin::root(),
			ALICE,
			asset_options(PermissionLatest::new(ALICE), web3_asset_info.decimal_places()),
			web3_asset_info.clone()
		));

		// Should return the same info as ALICE set for the asset while creating it
		assert_eq!(<AssetMeta<Test>>::get(ASSET_ID), web3_asset_info);

		let web3_asset_info = AssetInfo::new(b"WEB3.1".to_vec(), 5, 11);
		// Should succeed as ALICE is the owner of this asset
		assert_ok!(GenericAsset::update_asset_info(
			Origin::signed(ALICE),
			ASSET_ID,
			web3_asset_info.clone(),
		));

		assert_eq!(<AssetMeta<Test>>::get(ASSET_ID), web3_asset_info);
	});
}

#[test]
fn non_owner_permitted_update_asset_info() {
	new_test_ext_with_balance(STAKING_ASSET_ID, ALICE, INITIAL_BALANCE).execute_with(|| {
		let web3_asset_info = AssetInfo::new(b"WEB3.0".to_vec(), 3, 7);

		// Should succeed and set ALICE as the owner of ASSET_ID
		assert_ok!(GenericAsset::create(
			Origin::root(),
			ALICE,
			asset_options(PermissionLatest::new(ALICE), web3_asset_info.decimal_places()),
			web3_asset_info.clone(),
		));

		// Should succeed as ALICE could update the asset info
		assert_eq!(<AssetMeta<Test>>::get(ASSET_ID), web3_asset_info);

		let web3_asset_info = AssetInfo::new(b"WEB3.1".to_vec(), 5, 11);
		// Should fail as BOB hasn't got the permission
		assert_noop!(
			GenericAsset::update_asset_info(Origin::signed(BOB), ASSET_ID, web3_asset_info.clone()),
			Error::<Test>::NoUpdatePermission
		);

		let bob_update_permission = PermissionLatest {
			update: Owner::Address(BOB),
			mint: Owner::None,
			burn: Owner::None,
		};
		assert_ok!(GenericAsset::update_permission(
			Origin::signed(ALICE),
			ASSET_ID,
			bob_update_permission
		));
		// Should succeed as Bob has now got the update permission
		assert_ok!(GenericAsset::update_asset_info(
			Origin::signed(BOB),
			ASSET_ID,
			web3_asset_info.clone()
		));

		// Should succeed as BOB could update the asset info
		assert_eq!(<AssetMeta<Test>>::get(ASSET_ID), web3_asset_info);
	});
}
