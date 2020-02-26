//! Additional traits to srml original traits. These traits are generally used
//! to decouple `srml` modules from `prml` modules.

use crate::dispatch::{Parameter, DispatchError, DispatchResult};

use crate::traits::{
	ExistenceRequirement, Imbalance, SignedImbalance, WithdrawReasons,
};
use codec::FullCodec;
use sp_std::{fmt::Debug, marker::PhantomData, result};
use sp_runtime::traits::{
	PlugDoughnutApi, MaybeSerializeDeserialize, AtLeast32Bit, Zero,
};

/// Perform fee payment for an extrinsic
pub trait ChargeExtrinsicFee<AccountId, Extrinsic> {
	/// Calculate and charge a fee for the given `extrinsic`
	/// How the fee is calculated is an implementation detail.
	fn charge_extrinsic_fee<'a>(
		transactor: &AccountId,
		encoded_len: usize,
		extrinsic: &'a Extrinsic,
	) -> Result<(), &'static str>;
}

/// Charge fee trait
pub trait ChargeFee<AccountId> {
	/// The type of fee amount.
	type Amount;

	/// Charge `amount` of fees from `transactor`. Return Ok iff the payment was successful.
	fn charge_fee(transactor: &AccountId, amount: Self::Amount) -> Result<(), &'static str>;

	/// Refund `amount` of previous charged fees from `transactor`. Return Ok iff the refund was successful.
	fn refund_fee(transactor: &AccountId, amount: Self::Amount) -> Result<(), &'static str>;
}

/// Dummy `ChargeFee` implementation, mainly for testing purpose.
pub struct DummyChargeFee<T, U>(PhantomData<(T, U)>);

impl<T, U> ChargeExtrinsicFee<T, U> for DummyChargeFee<T, U> {
	fn charge_extrinsic_fee<'a>(
		_: &T,
		_: usize,
		_: &'a U,
	) -> Result<(), &'static str> {
		Ok(())
	}
}

impl<T, U> ChargeFee<T> for DummyChargeFee<T, U> {
	type Amount = U;

	fn charge_fee(_: &T, _: Self::Amount) -> Result<(), &'static str> { Ok(()) }
	fn refund_fee(_: &T, _: Self::Amount) -> Result<(), &'static str> { Ok(()) }
}

/// A type which can verify a doughnut delegation proof in order to dispatch a module/method call
/// into the runtime
/// The `verify()` hook is injected into every module/method on the runtime.
/// When a doughnut proof is included along with a transaction, `verify` will be invoked just before executing method logic.
pub trait DelegatedDispatchVerifier {
    type Doughnut: PlugDoughnutApi;
    type AccountId: Parameter;

    /// The doughnut permission domain it verifies
    const DOMAIN: &'static str;

	/// Check the doughnut authorizes a dispatched call to `module` and `method` for this domain
    fn verify_dispatch(
        _doughnut: &Self::Doughnut,
        _module: &str,
        _method: &str,
    ) -> Result<(), &'static str> {
		Err("Doughnut call to module and method verification not implemented for this domain")
    }

    /// Check the doughnut authorizes a dispatched call from runtime to the specified contract address for this domain.
    fn verify_runtime_to_contract_call(
        _caller: &Self::AccountId,
        _doughnut: &Self::Doughnut,
        _contract_addr: &Self::AccountId,
    ) -> Result<(), &'static str> {
        Err("Doughnut runtime to contract call verification is not implemented for this domain")
    }

    /// Check the doughnut authorizes a dispatched call from a contract to another contract with the specified addresses for this domain.
    fn verify_contract_to_contract_call(
        _caller: &Self::AccountId,
        _doughnut: &Self::Doughnut,
        _contract_addr: &Self::AccountId,
    ) -> Result<(), &'static str> {
        Err("Doughnut contract to contract call verification is not implemented for this domain")
    }
}

pub struct DummyDispatchVerifier<D, A>(PhantomData<(D, A)>);

/// A dummy implementation for when dispatch verifiaction is not needed
impl<D: PlugDoughnutApi, A: Parameter> DelegatedDispatchVerifier for DummyDispatchVerifier<D, A> {
    type Doughnut = D;
    type AccountId = A;
    const DOMAIN: &'static str = "";
    fn verify_dispatch(_: &Self::Doughnut, _: &str, _: &str) -> Result<(), &'static str> {
        Ok(())
    }
    fn verify_runtime_to_contract_call(
        _caller: &Self::AccountId,
        _doughnut: &Self::Doughnut,
        _contract_addr: &Self::AccountId,
    ) -> Result<(), &'static str> {
        Ok(())
    }

    fn verify_contract_to_contract_call(
        _caller: &Self::AccountId,
        _doughnut: &Self::Doughnut,
        _contract_addr: &Self::AccountId,
    ) -> Result<(), &'static str> {
        Ok(())
    }
}

impl DelegatedDispatchVerifier for () {
    type Doughnut = ();
    type AccountId = u64;
    const DOMAIN: &'static str = "";
    fn verify_dispatch(
        doughnut: &Self::Doughnut,
        module: &str,
        method: &str,
    ) -> Result<(), &'static str> {
        DummyDispatchVerifier::<Self::Doughnut, Self::AccountId>::verify_dispatch(doughnut, module, method)
    }
    fn verify_runtime_to_contract_call(
        caller: &Self::AccountId,
        doughnut: &Self::Doughnut,
        addr: &Self::AccountId,
    ) -> Result<(), &'static str> {
        DummyDispatchVerifier::<Self::Doughnut, Self::AccountId>::verify_runtime_to_contract_call(caller, doughnut, addr)
    }

    fn verify_contract_to_contract_call(
        caller: &Self::AccountId,
        doughnut: &Self::Doughnut,
        addr: &Self::AccountId,
    ) -> Result<(), &'static str> {
        DummyDispatchVerifier::<Self::Doughnut, Self::AccountId>::verify_contract_to_contract_call(caller, doughnut, addr)
    }
}

/// Something which may have doughnut. Returns a ref to the doughnut, if any.
/// It's main purpose is to allow checking if an `OuterOrigin` contains a doughnut (i.e. it is delegated).
pub trait MaybeDoughnutRef {
	/// The doughnut type
	type Doughnut: PlugDoughnutApi;
	/// Return a `&Doughnut`, if any
	fn doughnut(&self) -> Option<&Self::Doughnut>;
}

impl MaybeDoughnutRef for () {
	type Doughnut = ();
	fn doughnut(&self) -> Option<&Self::Doughnut> { None }
}

// Note: in the following traits the terms:
// - 'token' / 'asset' / 'currency' and
// - 'balance' / 'value' / 'amount'
// are used interchangeably as they make more sense in certain contexts.

/// An abstraction over the accounting behaviour of a fungible, multi-currency system
/// Currencies in the system are identifiable by a unique `CurrencyId`
pub trait MultiCurrencyAccounting {
	/// The ID type for an account in the system
	type AccountId: FullCodec + Debug + Default;
	/// The balance of an account for a particular currency
	type Balance: AtLeast32Bit + FullCodec + Copy + MaybeSerializeDeserialize + Debug + Default;
	/// The ID type of a currency in the system
	type CurrencyId: FullCodec + Debug + Default;
	/// A type the is aware of the default network currency ID
	/// When the currency ID is not specified for a `MultiCurrencyAccounting` method, it will be used
	/// by default
	type DefaultCurrencyId: AssetIdAuthority<AssetId=Self::CurrencyId>;
	/// The opaque token type for an imbalance of a particular currency. This is returned by unbalanced operations
	/// and must be dealt with. It may be dropped but cannot be cloned.
	type NegativeImbalance: Imbalance<Self::Balance, Opposite=Self::PositiveImbalance>;
	/// The opaque token type for an imbalance of a particular currency. This is returned by unbalanced operations
	/// and must be dealt with. It may be dropped but cannot be cloned.
	type PositiveImbalance: Imbalance<Self::Balance, Opposite=Self::NegativeImbalance>;

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
		value: Self::Balance
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

/// A type which provides an ID with authority from chain storage
pub trait AssetIdAuthority {
	/// The asset ID type e.g a `u32`
	type AssetId;
	/// Return the authoritative asset ID
	fn asset_id() -> Self::AssetId;
}

/// A type which can provide it's inherent asset ID
/// It is useful in the context of an asset/currency aware balance type
/// It differs from `AssetIdAuthority` in that it is not statically defined
pub trait InherentAssetIdProvider {
	/// The asset ID type e.g. a `u32`
	type AssetId;
	/// Return the inherent asset ID
	fn asset_id(&self) -> Self::AssetId;
}
