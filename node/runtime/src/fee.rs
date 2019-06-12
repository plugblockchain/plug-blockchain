//!
//! Runtime extrinsic fee logic
//!
use crate::{Call, Balances, Runtime, AccountId, Header};
use runtime_primitives::traits::{Applyable, Block as BlockT, Checkable};
use support::{
	dispatch::Result,
	traits::MakePayment,
	additional_traits::ChargeExtrinsicFee,
};
use node_primitives::{Hash, Index};

/// A type that does fee calculation and payment for extrinsics
pub struct ExtrinsicFeeCharger<Block, Context, T>(rstd::marker::PhantomData<(Block, Context, T)>);

impl<Block, Context> ChargeExtrinsicFee<AccountId, <Block::Extrinsic as Checkable<Context>>::Checked> for ExtrinsicFeeCharger<Block, Context, Runtime>
where
	Context: Default,
	Block: BlockT<Header = Header, Hash = Hash>,
	Block::Extrinsic: Checkable<Context>,
	<Block::Extrinsic as Checkable<Context>>::Checked: Applyable<Index=Index, AccountId=AccountId, Call=Call>,
{
	fn charge_extrinsic_fee(transactor: &AccountId, encoded_len: usize, extrinsic: &<Block::Extrinsic as Checkable<Context>>::Checked) -> Result {
		<Balances as MakePayment<AccountId, <Block::Extrinsic as Checkable<Context>>::Checked>>::make_payment(transactor, encoded_len, extrinsic)
	}
}
