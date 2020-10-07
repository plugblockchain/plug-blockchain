// Copyright 2019 Plug New Zealand Ltd.
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
// along with Plug.  If not, see <http://www.gnu.org/licenses/>.

//! Extra trait implementations for the `GenericAsset` module

use crate::{Error, Module, NegativeImbalance, PositiveImbalance, SpendingAssetIdAuthority, Trait};
use codec::Codec;
use frame_support::traits::{ExistenceRequirement, Imbalance, SignedImbalance, WithdrawReasons};
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, CheckedSub, MaybeSerializeDeserialize, Zero},
	DispatchError, DispatchResult,
};
use sp_std::{fmt::Debug, result};

// Note: in the following traits the terms:
// - 'token' / 'asset' / 'currency' and
// - 'balance' / 'value' / 'amount'
// are used interchangeably as they make more sense in certain contexts.

/// Something which provides an ID with authority from chain storage
pub trait AssetIdAuthority {
	/// The asset ID type e.g a `u32`
	type AssetId;
	/// Return the authoritative asset ID (no `&self`)
	fn asset_id() -> Self::AssetId;
}

/// An abstraction over the accounting behaviour of a fungible, multi-currency system
/// Currencies in the system are identifiable by a unique `CurrencyId`
pub trait MultiCurrencyAccounting {
	/// The ID type for an account in the system
	type AccountId: Codec + Debug + Default;
	/// The balance of an account for a particular currency
	type Balance: AtLeast32BitUnsigned + Codec + Copy + MaybeSerializeDeserialize + Debug + Default;
	/// The ID type of a currency in the system
	type CurrencyId: Codec + Debug + Default;
	/// A type the is aware of the default network currency ID
	/// When the currency ID is not specified for a `MultiCurrencyAccounting` method, it will be used
	/// by default
	type DefaultCurrencyId: AssetIdAuthority<AssetId = Self::CurrencyId>;
	/// The opaque token type for an imbalance of a particular currency. This is returned by unbalanced operations
	/// and must be dealt with. It may be dropped but cannot be cloned.
	type NegativeImbalance: Imbalance<Self::Balance, Opposite = Self::PositiveImbalance>;
	/// The opaque token type for an imbalance of a particular currency. This is returned by unbalanced operations
	/// and must be dealt with. It may be dropped but cannot be cloned.
	type PositiveImbalance: Imbalance<Self::Balance, Opposite = Self::NegativeImbalance>;

	// PUBLIC IMMUTABLES

	/// The minimum balance any single account may have. This is equivalent to the `Balances` module's
	/// `ExistentialDeposit`.
	fn minimum_balance() -> Self::Balance {
		Zero::zero()
	}

	/// The combined balance (free + reserved) of `who` for the given `currency`.
	fn total_balance(who: &Self::AccountId, currency: Option<Self::CurrencyId>) -> Self::Balance;

	/// The 'free' balance of a given account.
	///
	/// This is the only balance that matters in terms of most operations on tokens. It alone
	/// is used to determine the balance when in the contract execution environment. When this
	/// balance falls below the value of `ExistentialDeposit`, then the 'current account' is
	/// deleted: specifically `FreeBalance`. Further, the `OnFreeBalanceZero` callback
	/// is invoked, giving a chance to external modules to clean up data associated with
	/// the deleted account.
	///
	/// `system::AccountNonce` is also deleted if `ReservedBalance` is also zero (it also gets
	/// collapsed to zero if it ever becomes less than `ExistentialDeposit`.
	fn free_balance(who: &Self::AccountId, currency: Option<Self::CurrencyId>) -> Self::Balance;

	/// Returns `Ok` iff the account is able to make a withdrawal of the given amount
	/// for the given reason. Basically, it's just a dry-run of `withdraw`.
	///
	/// `Err(...)` with the reason why not otherwise.
	fn ensure_can_withdraw(
		who: &Self::AccountId,
		currency: Option<Self::CurrencyId>,
		_amount: Self::Balance,
		reasons: WithdrawReasons,
		new_balance: Self::Balance,
	) -> DispatchResult;

	// PUBLIC MUTABLES (DANGEROUS)

	/// Adds up to `value` to the free balance of `who`. If `who` doesn't exist, it is created.
	///
	/// Infallible.
	fn deposit_creating(
		who: &Self::AccountId,
		currency: Option<Self::CurrencyId>,
		value: Self::Balance,
	) -> Self::PositiveImbalance;

	/// Mints `value` to the free balance of `who`.
	///
	/// If `who` doesn't exist, nothing is done and an Err returned.
	fn deposit_into_existing(
		who: &Self::AccountId,
		currency: Option<Self::CurrencyId>,
		value: Self::Balance,
	) -> result::Result<Self::PositiveImbalance, DispatchError>;

	/// Ensure an account's free balance equals some value; this will create the account
	/// if needed.
	///
	/// Returns a signed imbalance and status to indicate if the account was successfully updated or update
	/// has led to killing of the account.
	fn make_free_balance_be(
		who: &Self::AccountId,
		currency: Option<Self::CurrencyId>,
		balance: Self::Balance,
	) -> SignedImbalance<Self::Balance, Self::PositiveImbalance>;

	/// Transfer some liquid free balance to another staker.
	///
	/// This is a very high-level function. It will ensure all appropriate fees are paid
	/// and no imbalance in the system remains.
	fn transfer(
		source: &Self::AccountId,
		dest: &Self::AccountId,
		currency: Option<Self::CurrencyId>,
		value: Self::Balance,
		existence_requirement: ExistenceRequirement,
	) -> DispatchResult;

	/// Removes some free balance from `who` account for `reason` if possible. If `liveness` is
	/// `KeepAlive`, then no less than `ExistentialDeposit` must be left remaining.
	///
	/// This checks any locks, vesting, and liquidity requirements. If the removal is not possible,
	/// then it returns `Err`.
	///
	/// If the operation is successful, this will return `Ok` with a `NegativeImbalance` whose value
	/// is `value`.
	fn withdraw(
		who: &Self::AccountId,
		currency: Option<Self::CurrencyId>,
		value: Self::Balance,
		reasons: WithdrawReasons,
		liveness: ExistenceRequirement,
	) -> result::Result<Self::NegativeImbalance, DispatchError>;
}

impl<T: Trait> MultiCurrencyAccounting for Module<T> {
	type AccountId = T::AccountId;
	type CurrencyId = T::AssetId;
	type Balance = T::Balance;
	type DefaultCurrencyId = SpendingAssetIdAuthority<T>;
	type PositiveImbalance = PositiveImbalance<T>;
	type NegativeImbalance = NegativeImbalance<T>;

	fn total_balance(who: &T::AccountId, currency: Option<T::AssetId>) -> Self::Balance {
		<Module<T>>::total_balance(currency.unwrap_or_else(|| Self::DefaultCurrencyId::asset_id()), who)
	}

	fn free_balance(who: &T::AccountId, currency: Option<T::AssetId>) -> Self::Balance {
		<Module<T>>::free_balance(currency.unwrap_or_else(|| Self::DefaultCurrencyId::asset_id()), who)
	}

	fn deposit_creating(
		who: &T::AccountId,
		currency: Option<T::AssetId>,
		value: Self::Balance,
	) -> Self::PositiveImbalance {
		if value.is_zero() {
			return Self::PositiveImbalance::zero();
		}

		let asset_id = currency.unwrap_or_else(|| Self::DefaultCurrencyId::asset_id());
		let imbalance = Self::make_free_balance_be(who, currency, <Module<T>>::free_balance(asset_id, who) + value);
		if let SignedImbalance::Positive(p) = imbalance {
			p
		} else {
			// Impossible, but be defensive.
			Self::PositiveImbalance::zero()
		}
	}

	fn deposit_into_existing(
		who: &T::AccountId,
		currency: Option<T::AssetId>,
		value: Self::Balance,
	) -> result::Result<Self::PositiveImbalance, DispatchError> {
		// No existential deposit rule and creation fee in GA. `deposit_into_existing` is same with `deposit_creating`.
		Ok(Self::deposit_creating(who, currency, value))
	}

	fn ensure_can_withdraw(
		who: &T::AccountId,
		currency: Option<T::AssetId>,
		amount: Self::Balance,
		reasons: WithdrawReasons,
		new_balance: Self::Balance,
	) -> DispatchResult {
		<Module<T>>::ensure_can_withdraw(
			currency.unwrap_or_else(|| Self::DefaultCurrencyId::asset_id()),
			who,
			amount,
			reasons,
			new_balance,
		)
	}

	fn make_free_balance_be(
		who: &T::AccountId,
		currency: Option<T::AssetId>,
		balance: Self::Balance,
	) -> SignedImbalance<Self::Balance, Self::PositiveImbalance> {
		let asset_id = currency.unwrap_or_else(|| Self::DefaultCurrencyId::asset_id());
		let original = <Module<T>>::free_balance(asset_id, who);
		let imbalance = if original <= balance {
			SignedImbalance::Positive(Self::PositiveImbalance::new(balance - original, asset_id))
		} else {
			SignedImbalance::Negative(Self::NegativeImbalance::new(original - balance, asset_id))
		};
		<Module<T>>::set_free_balance(asset_id, who, balance);
		imbalance
	}

	fn transfer(
		transactor: &T::AccountId,
		dest: &T::AccountId,
		currency: Option<T::AssetId>,
		value: Self::Balance,
		_ex: ExistenceRequirement, // no existential deposit policy for generic asset
	) -> DispatchResult {
		if value.is_zero() {
			return Ok(());
		}
		<Module<T>>::make_transfer(
			currency.unwrap_or_else(|| Self::DefaultCurrencyId::asset_id()),
			transactor,
			dest,
			value,
		)
	}

	fn withdraw(
		who: &T::AccountId,
		currency: Option<T::AssetId>,
		value: Self::Balance,
		reasons: WithdrawReasons,
		_ex: ExistenceRequirement, // no existential deposit policy for generic asset
	) -> result::Result<Self::NegativeImbalance, DispatchError> {
		if value.is_zero() {
			return Ok(Self::NegativeImbalance::zero());
		}

		let asset_id = currency.unwrap_or_else(|| Self::DefaultCurrencyId::asset_id());
		let new_balance = <Module<T>>::free_balance(asset_id, who)
			.checked_sub(&value)
			.ok_or(Error::<T>::InsufficientBalance)?;

		<Module<T>>::ensure_can_withdraw(asset_id, who, value, reasons, new_balance)?;
		<Module<T>>::set_free_balance(asset_id, who, new_balance);

		Ok(Self::NegativeImbalance::new(value, asset_id))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::mock::{ExtBuilder, GenericAsset, Test};
	use frame_support::assert_noop;
	use sp_runtime::traits::Zero;

	#[test]
	fn multi_accounting_minimum_balance() {
		ExtBuilder::default().build().execute_with(|| {
			assert_eq!(
				<GenericAsset as MultiCurrencyAccounting>::minimum_balance(),
				Zero::zero()
			);
		});
	}

	#[test]
	fn multi_accounting_total_balance() {
		let (alice, asset_id, amount) = (&1, 16000, 100);
		ExtBuilder::default()
			.free_balance((asset_id, *alice, amount))
			.build()
			.execute_with(|| {
				assert_eq!(
					<GenericAsset as MultiCurrencyAccounting>::total_balance(alice, Some(asset_id)),
					amount
				);

				GenericAsset::reserve(asset_id, alice, amount / 2).ok();
				// total balance should include reserved balance
				assert_eq!(
					<GenericAsset as MultiCurrencyAccounting>::total_balance(alice, Some(asset_id)),
					amount
				);
			});
	}

	#[test]
	fn multi_accounting_free_balance() {
		let (alice, asset_id, amount) = (&1, 16000, 100);
		ExtBuilder::default()
			.free_balance((asset_id, *alice, amount))
			.build()
			.execute_with(|| {
				assert_eq!(
					<GenericAsset as MultiCurrencyAccounting>::free_balance(alice, Some(asset_id)),
					amount
				);

				GenericAsset::reserve(asset_id, alice, amount / 2).ok();
				// free balance should not include reserved balance
				assert_eq!(
					<GenericAsset as MultiCurrencyAccounting>::free_balance(alice, Some(asset_id)),
					amount / 2
				);
			});
	}

	#[test]
	fn multi_accounting_deposit_creating() {
		let (alice, asset_id, amount) = (&1, 16000, 100);
		ExtBuilder::default().build().execute_with(|| {
			let imbalance = <GenericAsset as MultiCurrencyAccounting>::deposit_creating(alice, Some(asset_id), amount);
			// Check a positive imbalance of `amount` was created
			assert_eq!(imbalance.peek(), amount);
			// check free balance of asset has increased with `make_free_balance_be
			assert_eq!(GenericAsset::free_balance(asset_id, &alice), amount);
			// explitically drop `imbalance` so issuance is managed
			drop(imbalance);
			// check issuance of asset has increased with `make_free_balance_be`
			assert_eq!(GenericAsset::total_issuance(asset_id), amount);
		});
	}

	#[test]
	fn multi_accounting_deposit_into_existing() {
		let (alice, asset_id, amount) = (&1, 16000, 100);
		ExtBuilder::default().build().execute_with(|| {
			let result =
				<GenericAsset as MultiCurrencyAccounting>::deposit_into_existing(alice, Some(asset_id), amount);
			// Check a positive imbalance of `amount` was created
			assert_eq!(result.unwrap().peek(), amount);
			// check free balance of asset has increased with `make_free_balance_be
			assert_eq!(GenericAsset::free_balance(asset_id, &alice), amount);
			// check issuance of asset has increased with `make_free_balance_be`
			assert_eq!(GenericAsset::total_issuance(asset_id), amount);
		});
	}

	#[test]
	fn multi_accounting_ensure_can_withdraw() {
		let (alice, asset_id, amount) = (1, 16000, 100);
		ExtBuilder::default()
			.free_balance((asset_id, alice, amount))
			.build()
			.execute_with(|| {
				assert_eq!(
					<GenericAsset as MultiCurrencyAccounting>::ensure_can_withdraw(
						&alice,
						Some(asset_id),
						amount / 2,
						WithdrawReasons::none(),
						amount / 2,
					),
					Ok(())
				);

				// check free balance has not decreased
				assert_eq!(GenericAsset::free_balance(asset_id, &alice), amount);
				// check issuance has not decreased
				assert_eq!(GenericAsset::total_issuance(asset_id), amount);
			});
	}

	#[test]
	fn multi_accounting_make_free_balance_be() {
		let (alice, asset_id, amount) = (1, 16000, 100);
		ExtBuilder::default().build().execute_with(|| {
			// Issuance should be `0` initially
			assert_eq!(GenericAsset::total_issuance(asset_id), 0);

			let result =
				<GenericAsset as MultiCurrencyAccounting>::make_free_balance_be(&alice, Some(asset_id), amount);
			// Check a positive imbalance of `amount` was created
			if let SignedImbalance::Positive(imb) = result.0 {
				assert_eq!(imb.peek(), amount);
			} else {
				assert!(false);
			}
			// check free balance of asset has increased with `make_free_balance_be
			assert_eq!(GenericAsset::free_balance(asset_id, &alice), amount);
			// check issuance of asset has increased with `make_free_balance_be`
			assert_eq!(GenericAsset::total_issuance(asset_id), amount);
		});
	}

	#[test]
	fn multi_accounting_transfer() {
		let (alice, dest_id, asset_id, amount) = (1, 2, 16000, 100);

		ExtBuilder::default()
			.free_balance((asset_id, alice, amount))
			.build()
			.execute_with(|| {
				assert_eq!(
					<GenericAsset as MultiCurrencyAccounting>::transfer(
						&alice,
						&dest_id,
						Some(asset_id),
						amount,
						ExistenceRequirement::KeepAlive
					),
					Ok(())
				);
				assert_eq!(GenericAsset::free_balance(asset_id, &dest_id), amount);
			});
	}

	#[test]
	fn multi_accounting_withdraw() {
		let (alice, asset_id, amount) = (1, 16000, 100);
		ExtBuilder::default()
			.free_balance((asset_id, alice, amount))
			.build()
			.execute_with(|| {
				assert_eq!(GenericAsset::total_issuance(asset_id), amount);
				let result = <GenericAsset as MultiCurrencyAccounting>::withdraw(
					&alice,
					Some(asset_id),
					amount / 2,
					WithdrawReasons::none(),
					ExistenceRequirement::KeepAlive,
				);
				assert_eq!(result.unwrap().peek(), amount / 2);

				// check free balance of asset has decreased for the account
				assert_eq!(GenericAsset::free_balance(asset_id, &alice), amount / 2);
				// check global issuance has decreased for the asset
				assert_eq!(GenericAsset::total_issuance(asset_id), amount / 2);
			});
	}

	#[test]
	fn it_uses_default_currency_when_unspecified() {
		// Run through all the `MultiAccounting` functions checking that the default currency is
		// used when the Asset ID is left unspecified (`None`)
		let (alice, bob, amount) = (&1, &2, 100);
		ExtBuilder::default()
			.free_balance((16001, *alice, amount)) // `160001` is the spending asset id from genesis config
			.build()
			.execute_with(|| {
				assert_eq!(
					<GenericAsset as MultiCurrencyAccounting>::total_balance(alice, None),
					amount
				);

				assert_eq!(
					<GenericAsset as MultiCurrencyAccounting>::free_balance(alice, None),
					amount
				);

				// Mint `amount` of default currency into `alice`s account
				let _ = <GenericAsset as MultiCurrencyAccounting>::deposit_creating(alice, None, amount);
				// Check balance updated
				assert_eq!(
					<GenericAsset as MultiCurrencyAccounting>::total_balance(alice, None),
					amount + amount
				);
				assert_eq!(GenericAsset::total_issuance(16001), amount + amount);

				// Make free balance be equal to `amount` again
				let _ = <GenericAsset as MultiCurrencyAccounting>::make_free_balance_be(alice, None, amount);
				assert_eq!(
					<GenericAsset as MultiCurrencyAccounting>::free_balance(alice, None),
					amount
				);
				assert_eq!(GenericAsset::total_issuance(16001), amount);

				// Mint `amount` of `asset_id` into `alice`s account. Similar to `deposit_creating` above
				let _ = <GenericAsset as MultiCurrencyAccounting>::deposit_into_existing(alice, None, amount);
				// Check balance updated
				assert_eq!(
					<GenericAsset as MultiCurrencyAccounting>::total_balance(alice, None),
					amount + amount
				);
				assert_eq!(GenericAsset::total_issuance(16001), amount + amount);

				// transfer
				let _ = <GenericAsset as MultiCurrencyAccounting>::transfer(
					alice,
					bob,
					None,
					amount,
					ExistenceRequirement::KeepAlive,
				);
				assert_eq!(
					<GenericAsset as MultiCurrencyAccounting>::free_balance(alice, None),
					amount
				);
				assert_eq!(
					<GenericAsset as MultiCurrencyAccounting>::free_balance(bob, None),
					amount
				);
				assert_eq!(GenericAsset::total_issuance(16001), amount + amount);

				// ensure can withdraw
				assert!(<GenericAsset as MultiCurrencyAccounting>::ensure_can_withdraw(
					alice,
					None,
					amount,
					WithdrawReasons::none(),
					amount,
				)
				.is_ok());

				// withdraw
				let _ = <GenericAsset as MultiCurrencyAccounting>::withdraw(
					alice,
					None,
					amount / 2,
					WithdrawReasons::none(),
					ExistenceRequirement::KeepAlive,
				);
				assert_eq!(
					<GenericAsset as MultiCurrencyAccounting>::free_balance(alice, None),
					amount / 2
				);
			});
	}
	#[test]
	fn multi_accounting_transfer_more_than_free_balance_should_fail() {
		let (alice, dest_id, asset_id, amount) = (1, 2, 16000, 100);

		ExtBuilder::default()
			.free_balance((asset_id, alice, amount))
			.build()
			.execute_with(|| {
				assert_noop!(
					<GenericAsset as MultiCurrencyAccounting>::transfer(
						&alice,
						&dest_id,
						Some(asset_id),
						amount * 2,
						ExistenceRequirement::KeepAlive
					),
					Error::<Test>::InsufficientBalance,
				);
			});
	}

	#[test]
	fn multi_accounting_transfer_locked_funds_should_fail() {
		let (alice, dest_id, asset_id, amount) = (1, 2, 16000, 100);
		ExtBuilder::default()
			.free_balance((asset_id, alice, amount))
			.build()
			.execute_with(|| {
				// Lock alice's funds
				GenericAsset::set_lock(1u64.to_be_bytes(), &alice, amount, WithdrawReasons::all());

				assert_noop!(
					<GenericAsset as MultiCurrencyAccounting>::transfer(
						&alice,
						&dest_id,
						Some(asset_id),
						amount,
						ExistenceRequirement::KeepAlive
					),
					Error::<Test>::LiquidityRestrictions,
				);
			});
	}

	#[test]
	fn multi_accounting_transfer_reserved_funds_should_fail() {
		let (alice, dest_id, asset_id, amount) = (1, 2, 16000, 100);
		ExtBuilder::default()
			.free_balance((asset_id, alice, amount))
			.build()
			.execute_with(|| {
				GenericAsset::reserve(asset_id, &alice, amount).ok();
				assert_noop!(
					<GenericAsset as MultiCurrencyAccounting>::transfer(
						&alice,
						&dest_id,
						Some(asset_id),
						amount,
						ExistenceRequirement::KeepAlive
					),
					Error::<Test>::InsufficientBalance,
				);
			});
	}

	#[test]
	fn multi_accounting_withdraw_more_than_free_balance_should_fail() {
		let (alice, asset_id, amount) = (1, 16000, 100);
		ExtBuilder::default()
			.free_balance((asset_id, alice, amount))
			.build()
			.execute_with(|| {
				assert_noop!(
					<GenericAsset as MultiCurrencyAccounting>::withdraw(
						&alice,
						Some(asset_id),
						amount * 2,
						WithdrawReasons::none(),
						ExistenceRequirement::KeepAlive
					),
					Error::<Test>::InsufficientBalance,
				);
			});
	}

	#[test]
	fn multi_accounting_withdraw_locked_funds_should_fail() {
		let (alice, asset_id, amount) = (1, 16000, 100);
		ExtBuilder::default()
			.free_balance((asset_id, alice, amount))
			.build()
			.execute_with(|| {
				// Lock alice's funds
				GenericAsset::set_lock(1u64.to_be_bytes(), &alice, amount, WithdrawReasons::all());

				assert_noop!(
					<GenericAsset as MultiCurrencyAccounting>::withdraw(
						&alice,
						Some(asset_id),
						amount,
						WithdrawReasons::all(),
						ExistenceRequirement::KeepAlive
					),
					Error::<Test>::LiquidityRestrictions,
				);
			});
	}

	#[test]
	fn multi_accounting_withdraw_reserved_funds_should_fail() {
		let (alice, asset_id, amount) = (1, 16000, 100);
		ExtBuilder::default()
			.free_balance((asset_id, alice, amount))
			.build()
			.execute_with(|| {
				// Reserve alice's funds
				GenericAsset::reserve(asset_id, &alice, amount).ok();

				assert_noop!(
					<GenericAsset as MultiCurrencyAccounting>::withdraw(
						&alice,
						Some(asset_id),
						amount,
						WithdrawReasons::all(),
						ExistenceRequirement::KeepAlive
					),
					Error::<Test>::InsufficientBalance,
				);
			});
	}

	#[test]
	fn multi_accounting_make_free_balance_edge_cases() {
		let (alice, asset_id) = (&1, 16000);
		ExtBuilder::default().build().execute_with(|| {
			let max_value = u64::max_value();
			let min_value = Zero::zero();

			let _ = <GenericAsset as MultiCurrencyAccounting>::make_free_balance_be(alice, Some(asset_id), max_value);
			// Check balance updated
			assert_eq!(GenericAsset::total_issuance(asset_id), max_value);
			assert_eq!(
				<GenericAsset as MultiCurrencyAccounting>::free_balance(alice, Some(asset_id)),
				max_value
			);

			let _ = <GenericAsset as MultiCurrencyAccounting>::make_free_balance_be(alice, Some(asset_id), min_value);
			// Check balance updated
			assert_eq!(GenericAsset::total_issuance(asset_id), min_value);
			assert_eq!(
				<GenericAsset as MultiCurrencyAccounting>::free_balance(alice, Some(asset_id)),
				min_value
			);
		})
	}
}
