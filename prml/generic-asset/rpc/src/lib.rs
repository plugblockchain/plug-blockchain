// Copyright 2019-2020 Parity Technologies (UK) Ltd.
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

//! RPC interface for the generic asset module.

pub use self::gen_client::Client as GenericAssetClient;
use codec::{Decode, Encode};
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use prml_generic_asset::AssetInfo;
pub use prml_generic_asset_rpc_runtime_api::AssetMetaApi;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;



#[rpc]
pub trait GenericAssetApi<BlockHash, ResponseType> {
	/// Get all assets data paired with their ids.
	#[rpc(name = "genericAsset_registeredAssets")]
	fn asset_meta(&self, at: Option<BlockHash>) -> Result<ResponseType>;
}

/// A struct that implements the [`GenericAssetApi`].
pub struct GenericAsset<C, P> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<P>,
}

impl<C, P> GenericAsset<C, P> {
	/// Create new `GenericAsset` with the given reference to the client.
	pub fn new(client: Arc<C>) -> Self {
		GenericAsset {
			client,
			_marker: Default::default(),
		}
	}
}

/// Error type of this RPC api.
pub enum Error {
	/// The call to runtime failed.
	RuntimeError,
}

impl<C, Block, AssetId> GenericAssetApi<<Block as BlockT>::Hash, Vec<(AssetId, AssetInfo)>>
	for GenericAsset<C, (Block, AssetId)>
where
	Block: BlockT,
	C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	C::Api: AssetMetaApi<Block, AssetId>,
	AssetId: Decode + Encode + Send + Sync + 'static,
{
	fn asset_meta(&self, at: Option<<Block as BlockT>::Hash>) -> Result<Vec<(AssetId, AssetInfo)>> {
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		self.client.runtime_api().asset_meta(&at).map_err(|e| RpcError {
			code: ErrorCode::ServerError(Error::RuntimeError as i64),
			message: "Unable to query asset meta data.".into(),
			data: Some(format!("{:?}", e).into()),
		})
	}
}
#[cfg(test)]
mod test {
	use substrate_test_runtime_client::{
		runtime::Block,
		Backend,
		DefaultTestClientBuilderExt,
		TestClient,
		TestClientBuilderExt,
		TestClientBuilder,
	};
	use sp_application_crypto::AppPair;
	use sp_keyring::Sr25519Keyring;
	use sp_core::{crypto::key_types::BABE};
	use sp_keystore::{SyncCryptoStorePtr, SyncCryptoStore};
	use sc_keystore::LocalKeystore;
	use sc_rpc_api::DenyUnsafe;

	use std::sync::Arc;
	use sc_consensus_babe::{Config, block_import, AuthorityPair};
	use sc_consensus_babe_rpc::BabeRpcHandler;
	use jsonrpc_core::IoHandler;
	use tempfile::tempfile;

	fn create_temp_keystore_ga<P: AppPair>(
		authority: Sr25519Keyring,
	) -> (SyncCryptoStorePtr, tempfile::TempDir) {
		let keystore_path = tempfile::tempdir().expect("Creates keystore path");
		let keystore = Arc::new(LocalKeystore::open(keystore_path.path(), None)
			.expect("Creates keystore"));
		SyncCryptoStore::sr25519_generate_new(&*keystore, BABE, Some(&authority.to_seed()))
			.expect("Creates authority key");

		(keystore, keystore_path)
	}

	fn test_ga_rpc_handler(
		deny_unsafe: DenyUnsafe
	) -> BabeRpcHandler<Block, TestClient, sc_consensus::LongestChain<Backend, Block>> {
		let builder = TestClientBuilder::new();
		let (client, longest_chain) = builder.build_with_longest_chain();
		let client = Arc::new(client);
		let config = Config::get_or_compute(&*client).expect("config available");
		let (_, link) = block_import(
			config.clone(),
			client.clone(),
			client.clone(),
		).expect("can initialize block-import");

		let epoch_changes = link.epoch_changes().clone();
		let keystore = create_temp_keystore_ga::<AuthorityPair>(Sr25519Keyring::Alice).0;

		BabeRpcHandler::new(
			client.clone(),
			epoch_changes,
			keystore,
			config,
			longest_chain,
			deny_unsafe,
		)
	}

	#[test]
	fn working_registered_assets_rpc() {

		//let handler = test_ga_rpc_handler(DenyUnsafe::No);

		let mut io = IoHandler::new();
		//io.extend_with(GenericAssetApi::<sp_core::H256, _>::to_delegate(handler::<TestClient>));

		let request = r#"{
			"id":"1", "jsonrpc":"2.0",
			"method": "genericAsset_registeredAssets",
			"params":[]
			}"#;

		let response = "{\"jsonrpc\":\"2.0\",\"result\":[[16001,{\
			\"decimal_places\":4,\
			\"existential_deposit\":1,\
			\"symbol\":[]}],\
			[16000,{\"decimal_places\":4,\
			\"existential_deposit\":1,\
			\"symbol\":[]}]],\
			\"id\":\"1\"}";

		assert_eq!(Some(response.into()), io.handle_request_sync(request));
	}
}
