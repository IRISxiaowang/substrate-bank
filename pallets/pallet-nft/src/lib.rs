//! # NFT Pallet
//!
//! This pallet provides functionality for creating, transferring, and burning non-fungible tokens
//! (NFTs). It supports role-based access control and integrates with a role management system.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
use sp_runtime::DispatchResult;
use sp_std::{prelude::*, vec::Vec};

use primitives::{NftId, Role, FILENAME_MAXSIZE};
use traits::{ManageNfts, ManageRoles};

mod mock;
mod tests;

pub mod weights;
pub use weights::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

/// Represents NFT data including its raw data and file name.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct NftData {
	pub data: Vec<u8>,
	pub file_name: Vec<u8>,
}

pub use module::*;

#[frame_support::pallet]
pub mod module {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type WeightInfo: WeightInfo;

		type RoleManager: ManageRoles<Self::AccountId>;

		type EnsureGovernance: EnsureOrigin<<Self as frame_system::Config>::RuntimeOrigin>;

		#[pallet::constant]
		type MaxSize: Get<u32>;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The caller is not authorized to perform this operation on the NFT.
		/// For example, trying to sell or change an NFT that belongs to someone else.
		Unauthorised,
		/// The specified NFT ID does not exist.
		InvalidNftId,
		/// The provided data exceeds the allowed size limit.
		DataTooLarge,
		/// The provided file name exceeds the maximum permitted length.
		FileNameTooLarge,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Emitted when an NFT is approved by the auditor and minted.
		NftMinted { owner: T::AccountId, nft_id: NftId },
		/// Emitted when an NFT is burned.
		NftBurned { nft_id: NftId },
		/// Emitted when an NFT is transferred from one owner to another.
		NftTransferred { from: T::AccountId, to: T::AccountId, nft_id: NftId },
		/// Emitted when a new NFT is requested to be minted, pending audit.
		NFTPending { nft_id: NftId, file_name: Vec<u8> },
		/// Emitted when an NFT creation is rejected by the auditor.
		NftRejected { nft_id: NftId },
	}

	#[pallet::storage]
	#[pallet::getter(fn next_nft)]
	pub type NextNftId<T: Config> = StorageValue<_, NftId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn pending_nft)]
	pub type PendingNft<T: Config> =
		StorageMap<_, Blake2_128Concat, NftId, (NftData, T::AccountId)>;
	#[pallet::storage]
	#[pallet::getter(fn owners)]
	pub type Owners<T: Config> = StorageMap<_, Blake2_128Concat, NftId, T::AccountId>;

	#[pallet::storage]
	#[pallet::getter(fn nfts)]
	pub type Nfts<T: Config> = StorageMap<_, Blake2_128Concat, NftId, NftData>;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Handles creating an NFT, marking it as pending audit.
		/// Validates file name and data size, ensuring they comply with pallet constraints.
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::request_mint())]
		pub fn request_mint(
			origin: OriginFor<T>,
			file_name: Vec<u8>,
			data: Vec<u8>,
		) -> DispatchResult {
			// Get the account id
			let id = ensure_signed(origin)?;

			// Ensure role is not auditor.
			T::RoleManager::ensure_not_role(&id, Role::Auditor)?;

			// Checks if the size of the NFT data is within the allowed maximum limit.
			ensure!(data.len() as u32 <= T::MaxSize::get(), Error::<T>::DataTooLarge);
			ensure!(file_name.len() as u32 <= FILENAME_MAXSIZE, Error::<T>::FileNameTooLarge);

			// Set the Nfts storage
			let nft_id = Self::next_nft_id();

			PendingNft::<T>::insert(
				nft_id,
				(NftData { data, file_name: file_name.clone() }, id.clone()),
			);

			Self::deposit_event(Event::<T>::NFTPending { nft_id, file_name });
			Ok(())
		}

		/// Burns an NFT, removing it from storage.
		/// Verifies that the caller is the NFT's owner or has appropriate permissions.
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::burned())]
		pub fn burned(origin: OriginFor<T>, nft_id: NftId) -> DispatchResult {
			// Get the account id
			let id = ensure_signed(origin)?;

			// Valid nft and owner
			Self::ensure_nft_is_valid(&id, nft_id)?;

			// Remove storage
			Nfts::<T>::remove(nft_id);
			Owners::<T>::remove(nft_id);

			Self::deposit_event(Event::<T>::NftBurned { nft_id });
			Ok(())
		}

		/// Transfers an NFT from one account to another.
		/// Checks for proper ownership and role compliance before proceeding.
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::transfer())]
		pub fn transfer(
			origin: OriginFor<T>,
			to_user: T::AccountId,
			nft_id: NftId,
		) -> DispatchResult {
			// Get the account id
			let id = ensure_signed(origin)?;

			// Ensure role is not auditor.
			T::RoleManager::ensure_not_role(&id, Role::Auditor)?;

			// Valid nft and owner
			Self::ensure_nft_is_valid(&id, nft_id)?;

			// Transfer Nft ownership to new user.
			Owners::<T>::mutate(nft_id, |user| {
				*user = Some(to_user.clone());
			});

			Self::deposit_event(Event::<T>::NftTransferred { from: id, to: to_user, nft_id });
			Ok(())
		}

		/// Audits and approves or rejects a pending NFT.
		/// Require an Auditor role.
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::approve_nft())]
		pub fn approve_nft(origin: OriginFor<T>, nft_id: NftId, approve: bool) -> DispatchResult {
			// Get the account id
			let id = ensure_signed(origin)?;

			// Ensure auditor role.
			T::RoleManager::ensure_role(&id, Role::Auditor)?;

			PendingNft::<T>::take(nft_id)
				.map(|(nft_data, user)| {
					if approve {
						Owners::<T>::insert(nft_id, user.clone());
						Nfts::<T>::insert(nft_id, nft_data);
						Self::deposit_event(Event::<T>::NftMinted { owner: user, nft_id });
					} else {
						PendingNft::<T>::remove(nft_id);
						Self::deposit_event(Event::<T>::NftRejected { nft_id });
					}
				})
				.ok_or(Error::<T>::InvalidNftId)?;

			Ok(())
		}

		/// Forces the burning of an NFT, bypassing usual ownership checks.
		/// Intended for governance use in exceptional situations.
		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::force_burn())]
		pub fn force_burn(origin: OriginFor<T>, nft_id: NftId) -> DispatchResult {
			// ensure governance
			T::EnsureGovernance::ensure_origin(origin)?;

			// Remove storage
			Nfts::<T>::remove(nft_id);
			Owners::<T>::remove(nft_id);

			Self::deposit_event(Event::<T>::NftBurned { nft_id });

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Get the nft id to store into the Nfts.
		pub(crate) fn next_nft_id() -> NftId {
			NextNftId::<T>::mutate(|id| {
				*id = id.wrapping_add(1);
				*id
			})
		}
	}

	impl<T: Config> ManageNfts<T::AccountId> for Pallet<T> {
		fn nft_transfer(
			from_user: &T::AccountId,
			to_user: &T::AccountId,
			nft_id: NftId,
		) -> DispatchResult {
			// Ensure role is not auditor.
			T::RoleManager::ensure_not_role(to_user, Role::Auditor)?;

			// Valid nft and owner
			Self::ensure_nft_is_valid(from_user, nft_id)?;

			// Transfer Nft ownership to new user.
			Owners::<T>::mutate(nft_id, |user| {
				*user = Some(to_user.clone());
			});

			Ok(())
		}

		fn ensure_nft_is_valid(id: &T::AccountId, nft_id: NftId) -> DispatchResult {
			// Ensure the NftId is valid.
			ensure!(Nfts::<T>::get(nft_id) != None, Error::<T>::InvalidNftId);

			// Ensure the Nft belong to the correct owner.
			ensure!(Owners::<T>::get(nft_id) == Some(id.clone()), Error::<T>::Unauthorised);

			Ok(())
		}

		fn owner(nft_id: NftId) -> Option<T::AccountId> {
			Owners::<T>::get(nft_id)
		}
	}
}
