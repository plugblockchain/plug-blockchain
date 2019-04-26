//! Additional traits to srml original traits. These traits are generally used
//! to decouple `srml` modules from `prml` modules.

use sr_std::marker::PhantomData;

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
