#![cfg_attr(not(feature = "std"), no_std)]
//!
//! This crate contains traits shared across the codebase.

use sp_runtime::{DispatchError, DispatchResult};

use primitives::{AuctionId, NftId, NftState, Role};

use sp_std::marker::PhantomData;

use sp_std::vec::Vec;

/// Trait for managing user roles.
pub trait ManageRoles<AccountId> {
	/// Get the role of a given user.
	fn role(id: &AccountId) -> Option<Role>;
	/// Register a role for a user.
	fn register_role(id: &AccountId, role: Role) -> DispatchResult;
	/// Unregister a role for a user.
	fn unregister_role(id: &AccountId) -> DispatchResult;
	/// Ensure that a user has a specific role.
	fn ensure_role(id: &AccountId, role: Role) -> DispatchResult;
	/// Ensure that a user has not a specific role.
	fn ensure_not_role(id: &AccountId, role: Role) -> DispatchResult;
}

/// A trait for basic accounting operations like deposit, withdrawal, and transfer.
pub trait BasicAccounting<AccountId, Balance> {
	fn deposit(user: &AccountId, amount: Balance) -> DispatchResult;
	fn withdraw(user: &AccountId, amount: Balance) -> DispatchResult;
	fn transfer(from: &AccountId, to: &AccountId, amount: Balance) -> DispatchResult;
	fn free_balance(user: &AccountId) -> Balance;
}

/// A trait for stake and redeem funds.
pub trait Stakable<AccountId, Balance> {
	fn stake_funds(user: &AccountId, amount: Balance) -> DispatchResult;
	fn redeem_funds(user: &AccountId, amount: Balance) -> DispatchResult;
	fn staked(user: &AccountId) -> Balance;
}

/// A trait for Nft operations like request mint, burn, transfer and approve.
pub trait ManageNfts<AccountId> {
	fn nft_transfer(nft_id: NftId, to_user: &AccountId) -> Result<AccountId, DispatchError>;
	fn ensure_nft_owner(id: &AccountId, nft_id: NftId) -> DispatchResult;
	fn ensure_nft_state(nft_id: NftId, state: NftState) -> DispatchResult;
	fn change_nft_state(nft_id: NftId, state: NftState) -> DispatchResult;

	#[cfg(feature = "runtime-benchmarks")]
	fn insert_nft(nft_id: NftId, owner: AccountId, file_name: Vec<u8>, data: Vec<u8>);
}

/// A trait for Auction operations like force cancel.
pub trait ManageAuctions<AccountId> {
	fn force_cancel(auction_id: AuctionId) -> DispatchResult;
}

/// A trait for getting the treasury account.
pub trait GetTreasury<AccountId> {
	fn treasury() -> Result<AccountId, DispatchError>;
}

pub struct SuccessOrigin<T>(PhantomData<T>);

impl<T: frame_system::Config> frame_support::traits::EnsureOrigin<T::RuntimeOrigin>
	for SuccessOrigin<T>
{
	type Success = ();

	fn try_origin(_o: T::RuntimeOrigin) -> Result<Self::Success, T::RuntimeOrigin> {
		Ok(())
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn try_successful_origin() -> Result<T::RuntimeOrigin, ()> {
		Ok(frame_system::RawOrigin::Root.into())
	}
}
