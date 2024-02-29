#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use scale_info::TypeInfo;

use sp_api::decl_runtime_apis;
use sp_runtime::DispatchError;
use sp_std::vec::Vec;

use pallet_bank::AccountData;
use primitives::{AccountId, Balance, BlockNumber, LockId};

/// Custom tool for translating Dispatch error to a human readable format.
#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum DispatchErrorTranslator {
	Message(Vec<u8>),
	Other(DispatchError),
}
impl From<DispatchError> for DispatchErrorTranslator {
	fn from(err: DispatchError) -> Self {
		match err {
			DispatchError::Module(sp_runtime::ModuleError { message: Some(message), .. }) =>
				DispatchErrorTranslator::Message(message.as_bytes().to_vec()),
			DispatchError::Other(str) => DispatchErrorTranslator::Message(str.as_bytes().to_vec()),
			err => DispatchErrorTranslator::Other(err),
		}
	}
}

decl_runtime_apis!(
	/// Custom Runtime API for the xy-chain
	pub trait CustomRuntimeApi {
		/// Returns account Data for a user
		fn account_data(who: AccountId) -> AccountData<Balance>;
		/// Calculate and returns the actual APY for an account in BPS format
		/// (with accumulated compounding interest)
		fn apy_in_bps(who: AccountId) -> u32;
		/// Returns when a locked fund is released.
		fn fund_unlock_at(who: AccountId, lock_id: LockId) -> BlockNumber;
	}
);
