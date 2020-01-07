//! Additional traits to srml original traits. These traits are generally used
//! to decouple `srml` modules from `prml` modules.

use crate::dispatch::Parameter;
use sp_runtime::traits::PlugDoughnutApi;
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
pub trait DelegatedDispatchVerifier {
    type Doughnut: DoughnutApi;
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
impl<D: DoughnutApi, A: Parameter> DelegatedDispatchVerifier for DummyDispatchVerifier<D, A> {
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
