// Copyright 2019-2020
//     by  Centrality Investments Ltd.
//     and Parity Technologies (UK) Ltd.
// This file is part of Plug-blockchain.

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

//! Runtime API definition required by Generic Asset RPC extensions.
//!
//! This API should be imported and implemented by the runtime,
//! of a node that wants to use the custom RPC extension
//! adding System access methods.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use pallet_generic_asset::AssetInfo;
use sp_std::vec::Vec;

sp_api::decl_runtime_apis! {
	/// The API to query asset meta information.
	pub trait AssetMetaApi<AssetId: Decode+Encode>
	{
		/// Get all assets data paired with their ids.
		fn asset_meta() -> Vec<(AssetId, AssetInfo)>;
	}
}
