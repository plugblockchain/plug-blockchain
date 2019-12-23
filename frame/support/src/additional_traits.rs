//! Additional traits to srml original traits. These traits are generally used
//! to decouple `srml` modules from `prml` modules.

use sp_runtime::traits::DoughnutApi;
use rstd::marker::PhantomData;

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
pub trait DelegatedDispatchVerifier<Doughnut> {
	type AccountId;

	/// The doughnut permission domain it verifies
	const DOMAIN: &'static str;
	/// Check the doughnut authorizes a dispatched call to `module` and `method` for this domain
	fn verify_dispatch(
		doughnut: &Doughnut,
		module: &str,
		method: &str,
	) -> Result<(), &'static str>;

	/// Check the doughnut authorizes a dispatched call from runtime to the specified contract address for this domain.
	fn verify_runtime_to_contract_dispatch(caller: &Self::AccountId, doughnut: &Doughnut, contract_addr: &Self::AccountId) -> Result<(), &'static str> {
		Err("Doughnut runtime to contract dispatch verification is not implemented for this domain")
	}
	
	/// Check the doughnut authorizes a dispatched call from a contract to another contract with the specified addresses for this domain.
	fn verify_contract_to_contract_dispatch(caller: &Self::AccountId, doughnut: &Doughnut, contract_addr: &Self::AccountId) -> Result<(), &'static str> {
		Err("Doughnut contract to contract dispatch verification is not implemented for this domain")
	}
}

/// A dummy implementation for when dispatch verifiaction is not needed
impl<Doughnut> DelegatedDispatchVerifier<Doughnut> for () {
	type AccountId = u64;
	const DOMAIN: &'static str = "";
	fn verify_dispatch(_: &Doughnut, _: &str, _: &str) -> Result<(), &'static str> {
		Ok(())
	}
}

/// Something which may have doughnut. Returns a ref to the doughnut, if any.
/// It's main purpose is to allow checking if an `OuterOrigin` contains a doughnut (i.e. it is delegated).
pub trait MaybeDoughnutRef {
	/// The doughnut type
	type Doughnut: DoughnutApi;
	/// Return a `&Doughnut`, if any
	fn doughnut(&self) -> Option<&Self::Doughnut>;
}

impl MaybeDoughnutRef for () {
	type Doughnut = ();
	fn doughnut(&self) -> Option<&Self::Doughnut> { None }
}
