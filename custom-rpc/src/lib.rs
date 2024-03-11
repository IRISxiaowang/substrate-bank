use jsonrpsee::{core::RpcResult, proc_macros::rpc, types::error::CallError};
use sc_client_api::HeaderBackend;
use serde::{Deserialize, Serialize};
use sp_runtime::traits::Block as BlockT;

use pallet_bank::{AccountData, LockReason, LockedFund};
use primitives::{AccountId, Balance, BlockNumber, Hash, LockId};
use xy_chain_runtime::runtime_api::{CustomRuntimeApi, DispatchErrorTranslator};

use std::{marker::PhantomData, sync::Arc};

#[derive(Serialize, Deserialize, Clone)]
pub struct RpcLockedFund {
	pub id: LockId,
	pub amount: Balance,
	pub reason: LockReason,
	pub unlock_at: BlockNumber,
}

impl RpcLockedFund {
	fn new(lock: LockedFund<Balance>, unlock_at: BlockNumber) -> Self {
		Self { id: lock.id, amount: lock.amount, reason: lock.reason, unlock_at }
	}
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RpcAccountData {
	pub free: Balance,
	pub reserved: Balance,
	pub locked: Vec<RpcLockedFund>,
}

impl RpcAccountData {
	fn from_account_data(
		account_data: AccountData<Balance>,
		unlock_query: &dyn Fn(LockId) -> BlockNumber,
	) -> Self {
		Self {
			free: account_data.free,
			reserved: account_data.reserved,
			locked: account_data
				.locked
				.into_iter()
				.map(|lock| RpcLockedFund::new(lock, unlock_query(lock.id)))
				.collect::<Vec<_>>(),
		}
	}
}

#[rpc(server, client, namespace = "xyChain")]
/// Custom RPC endpoints for the xy-chain
pub trait CustomRpcApi {
	/// Returns the full Account Data of a user.
	#[method(name = "account_data")]
	fn rpc_account_data(&self, who: AccountId, at: Option<Hash>) -> RpcResult<RpcAccountData>;
	/// Returns the interest earned per annum.
	#[method(name = "interest_pa")]
	fn rpc_interest_pa(&self, who: AccountId, at: Option<Hash>) -> RpcResult<Balance>;
}

pub struct CustomRpc<C, B> {
	pub client: Arc<C>,
	pub _phantom: PhantomData<B>,
}

impl<C, B> CustomRpc<C, B>
where
	B: BlockT<Hash = Hash, Header = xy_chain_runtime::Header>,
	C: sp_api::ProvideRuntimeApi<B> + Send + Sync + 'static + HeaderBackend<B>,
	C::Api: CustomRuntimeApi<B>,
{
	fn unwrap_or_best(&self, from_rpc: Option<<B as BlockT>::Hash>) -> B::Hash {
		from_rpc.unwrap_or_else(|| self.client.info().best_hash)
	}
}

fn to_rpc_error<E: std::error::Error + Send + Sync + 'static>(e: E) -> jsonrpsee::core::Error {
	CallError::from_std_error(e).into()
}

#[allow(dead_code)]
fn map_dispatch_error(e: DispatchErrorTranslator) -> jsonrpsee::core::Error {
	jsonrpsee::core::Error::from(match e {
		DispatchErrorTranslator::Message(message) => match std::str::from_utf8(&message) {
			Ok(message) => anyhow::anyhow!("DispatchError with translated message: \n{message}"),
			Err(error) => anyhow::anyhow!(
				"DispatchError with translated message: Cannot decode message: \n'{error}'"
			),
		},
		DispatchErrorTranslator::Other(e) =>
			anyhow::anyhow!("DispatchError - Other: {}", <&'static str>::from(e)),
	})
}

impl<C, B> CustomRpcApiServer for CustomRpc<C, B>
where
	B: BlockT<Hash = Hash, Header = xy_chain_runtime::Header>,
	C: sp_api::ProvideRuntimeApi<B> + Send + Sync + 'static + HeaderBackend<B>,
	C::Api: CustomRuntimeApi<B>,
{
	fn rpc_account_data(&self, who: AccountId, at: Option<Hash>) -> RpcResult<RpcAccountData> {
		let hash = self.unwrap_or_best(at);
		let account_data = self
			.client
			.runtime_api()
			.account_data(hash, who.clone())
			.map_err(to_rpc_error)?;

		Ok(RpcAccountData::from_account_data(account_data, &|lock_id| {
			self.client
				.runtime_api()
				.fund_unlock_at(hash, who.clone(), lock_id)
				.unwrap_or_default()
		}))
	}

	fn rpc_interest_pa(&self, who: AccountId, at: Option<Hash>) -> RpcResult<Balance> {
		self.client
			.runtime_api()
			.interest_pa(self.unwrap_or_best(at), who)
			.map_err(to_rpc_error)
	}
}
