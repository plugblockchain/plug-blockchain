//! Additional traits to srml original traits. These traits are generally used
//! to decouple srml modules from crml modules.

use sr_std::marker::PhantomData;

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
pub struct DummyChargeFee<T, U>(PhantomData<T>, PhantomData<U>);
impl<T, U> ChargeFee<T> for DummyChargeFee<T, U> {
	type Amount = U;

	fn charge_fee(_: &T, _: Self::Amount) -> Result<(), &'static str> { Ok(()) }
	fn refund_fee(_: &T, _: Self::Amount) -> Result<(), &'static str> { Ok(()) }
}

