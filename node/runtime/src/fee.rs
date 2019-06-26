// Copyright (C) 2019 Centrality Investments Limited
// This file is part of PLUG.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

//!
//! Runtime extrinsic fee logic
//!
use crate::{AccountId, Balances, Header, Runtime};
use runtime_primitives::traits::{Block as BlockT, Checkable};
use support::{
	dispatch::Result,
	traits::MakePayment,
	additional_traits::ChargeExtrinsicFee,
};
use node_primitives::{Hash};

/// A type that does fee calculation and payment for extrinsics
pub struct ExtrinsicFeeCharger<Block, Context, T>(rstd::marker::PhantomData<(Block, Context, T)>);

impl<Block, Context> ChargeExtrinsicFee<AccountId, <Block::Extrinsic as Checkable<Context>>::Checked> for ExtrinsicFeeCharger<Block, Context, Runtime>
where
	Context: Default,
	Block: BlockT<Header = Header, Hash = Hash>,
	Block::Extrinsic: Checkable<Context>,
{
	fn charge_extrinsic_fee(transactor: &AccountId, encoded_len: usize, extrinsic: &<Block::Extrinsic as Checkable<Context>>::Checked) -> Result {
		<Balances as MakePayment<AccountId, <Block::Extrinsic as Checkable<Context>>::Checked>>::make_payment(transactor, encoded_len, extrinsic)
	}
}
