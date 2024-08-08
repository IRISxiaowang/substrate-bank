//! # NFT Pallet
//!
//! This pallet provides functionality for creating, transferring, and burning non-fungible tokens
//! (NFTs). It supports role-based access control and integrates with a role management system.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;

use sp_runtime::{
	traits::{AtLeast32BitUnsigned, BlockNumberProvider, Saturating},
	DispatchResult,
};
use sp_std::{fmt::Debug, prelude::*, vec::Vec};

use primitives::{NftId, NftState, PodId, Response, Role, FILENAME_MAXSIZE};
use traits::{BasicAccounting, GetTreasury, ManageNfts, ManageRoles};

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
	pub state: NftState,
}

/// Represents Pod information including its nft id, price and the target user.
#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct PodInfo<T: Config> {
	pub nft_id: NftId,
	pub to_user: T::AccountId,
	pub price: T::Balance,
	pub expiry_block: BlockNumberFor<T>,
}

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
pub enum CancelReason {
	Expired,
	Canceled,
}

pub use module::*;

#[frame_support::pallet]
pub mod module {

	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type WeightInfo: WeightInfo;

		/// The balance type
		type Balance: Member
			+ Parameter
			+ MaxEncodedLen
			+ AtLeast32BitUnsigned
			+ Default
			+ Copy
			+ MaybeSerializeDeserialize
			+ Debug
			+ From<u128>
			+ sp_std::iter::Sum;

		type RoleManager: ManageRoles<Self::AccountId>;

		type Bank: BasicAccounting<Self::AccountId, Self::Balance> + GetTreasury<Self::AccountId>;

		type EnsureGovernance: EnsureOrigin<<Self as frame_system::Config>::RuntimeOrigin>;

		#[pallet::constant]
		type MaxSize: Get<u32>;

		#[pallet::constant]
		type PodFee: Get<Self::Balance>;

		#[pallet::constant]
		type NftLockedPeriod: Get<BlockNumberFor<Self>>;
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
		/// The specified NFT is not in the POD (Paid on Delivery) state.
		NftNotForPod,
		/// The receiver is not compatible with POD receiver.
		IncorrectReceiver,
		/// The nft state is not the required state.
		NftStateNotMatch,
		/// The nft state is not Free.
		NftStateNotFree,
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
		/// Emitted when an NFT creation is rejected by the auditor rejected the nft.
		NftRejected { nft_id: NftId },
		/// Indicates that an NFT has been listed for sale.
		NftPodCreated { from: T::AccountId, to: T::AccountId, nft_id: NftId, price: T::Balance },

		/// Indicates that someone received an Nft with a certain price.
		NftDelivered {
			seller: T::AccountId,
			buyer: T::AccountId,
			nft_id: NftId,
			price: T::Balance,
			tips: T::Balance,
		},
		/// Emitted when the receiver rejected the nft.
		NftPodRejected { nft_id: NftId },
		/// Indicates that the creator canceled the nft on POD.
		NftPodCanceled { nft_id: NftId, reason: CancelReason },
	}

	#[pallet::storage]
	#[pallet::getter(fn next_nft)]
	pub type NextNftId<T: Config> = StorageValue<_, NftId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn next_pod)]
	pub type NextPodId<T: Config> = StorageValue<_, PodId, ValueQuery>;

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

	/// Stores the pricing information for NFTs awaiting trade.
	#[pallet::storage]
	#[pallet::getter(fn pending_trade_nfts)]
	pub type PendingPodNfts<T: Config> = StorageMap<_, Blake2_128Concat, PodId, PodInfo<T>>;

	/// Stores the users' nft ids that expiry for POD pending delivery at a block.
	#[pallet::storage]
	pub type PodExpiry<T: Config> =
		StorageMap<_, Blake2_128Concat, BlockNumberFor<T>, Vec<(PodId, NftId)>, ValueQuery>;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(block_number: BlockNumberFor<T>) {
			// Expire nfts that are due.
			PodExpiry::<T>::take(block_number).into_iter().for_each(|(pod_id, nft_id)| {
				// Ignore the expired result.
				let _ = Self::cancel_nft_pod(pod_id, nft_id, CancelReason::Expired);
			});
		}
	}

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
				(NftData { data, file_name: file_name.clone(), state: NftState::Free }, id.clone()),
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
			Self::ensure_nft_owner(&id, nft_id)?;

			// Ensure the nft is enable to burn.
			Self::ensure_nft_state(nft_id, NftState::Free)?;

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
			Self::ensure_nft_owner(&id, nft_id)?;

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
		pub fn approve_nft(
			origin: OriginFor<T>,
			nft_id: NftId,
			response: Response,
		) -> DispatchResult {
			// Get the account id
			let id = ensure_signed(origin)?;

			// Ensure auditor role.
			T::RoleManager::ensure_role(&id, Role::Auditor)?;

			PendingNft::<T>::take(nft_id)
				.map(|(nft_data, user)| {
					if response == Response::Accept {
						Owners::<T>::insert(nft_id, user.clone());
						Nfts::<T>::insert(nft_id, nft_data);
						Self::deposit_event(Event::<T>::NftMinted { owner: user, nft_id });
					} else {
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
			if let Some(nft_data) = Nfts::<T>::take(nft_id) {
				if let NftState::POD(pod_id) = nft_data.state {
					PendingPodNfts::<T>::remove(pod_id);
				}
				// todo
				//if let NftState::POD(auction_id) = nft_data.state {
				// create a trait manageAuctioin, with function remove auction
				//	Auctions::<T>::remove(auction_id);
				// transfer the money back to the bider }
			}
			Owners::<T>::remove(nft_id);

			Self::deposit_event(Event::<T>::NftBurned { nft_id });

			Ok(())
		}

		#[pallet::call_index(5)]
		#[pallet::weight(T::WeightInfo::create_pod())]
		pub fn create_pod(
			origin: OriginFor<T>,
			to_user: T::AccountId,
			nft_id: NftId,
			amount: T::Balance,
		) -> DispatchResult {
			// Get the account id
			let id = ensure_signed(origin)?;

			// Ensure role is not auditor.
			T::RoleManager::ensure_not_role(&id, Role::Auditor)?;
			T::RoleManager::ensure_role(&to_user, Role::Customer)?;

			// Ensure the nft is belong to the correct owner.
			Self::ensure_nft_owner(&id, nft_id)?;
			// Change nft state to POD.
			let pod_id = Self::next_pod_id();
			// Ensure the nft state is free, then changed to new state.
			Self::change_nft_state(nft_id, NftState::POD(pod_id))?;

			// Get treasury account.
			let treasury = T::Bank::treasury()?;

			// Added the block number that the nft processing will be expired.
			let expired_at =
				frame_system::Pallet::<T>::current_block_number() + T::NftLockedPeriod::get();
			PodExpiry::<T>::append(expired_at, (pod_id, nft_id));

			// Add the price and target user to the storage.
			PendingPodNfts::<T>::insert(
				pod_id,
				PodInfo {
					nft_id,
					to_user: to_user.clone(),
					price: amount,
					expiry_block: expired_at,
				},
			);

			// Managers do not pay fee.
			if T::RoleManager::role(&id) == Some(Role::Customer) {
				T::Bank::transfer(&id, &treasury, T::PodFee::get())?;
			}

			Self::deposit_event(Event::<T>::NftPodCreated {
				from: id,
				to: to_user,
				nft_id,
				price: amount,
			});

			Ok(())
		}

		#[pallet::call_index(6)]
		#[pallet::weight(T::WeightInfo::receive_pod())]
		pub fn receive_pod(
			origin: OriginFor<T>,
			pod_id: PodId,
			response: Response,
			tips: Option<T::Balance>,
		) -> DispatchResult {
			// Get the account id
			let buyer = ensure_signed(origin)?;

			// Ensure buyer role is customer.
			T::RoleManager::ensure_role(&buyer, Role::Customer)?;

			// Ensure the caller is the intended receiver
			let pod_info = PendingPodNfts::<T>::take(pod_id).ok_or(Error::<T>::NftNotForPod)?;
			ensure!(pod_info.to_user == buyer, Error::<T>::IncorrectReceiver);

			if response == Response::Accept {
				let final_amount = pod_info.price.saturating_add(tips.unwrap_or_default());

				// Transfer fund to the seller and ownership to the buyer
				let seller = Self::nft_transfer(pod_info.nft_id, &buyer)?;

				// Pay fee to Treasury account.
				let final_seller = match T::RoleManager::role(&seller) {
					Some(Role::Manager) => T::Bank::treasury()?,
					_ => seller,
				};

				T::Bank::transfer(&buyer, &final_seller, final_amount)?;

				Self::deposit_event(Event::<T>::NftDelivered {
					seller: final_seller,
					buyer,
					nft_id: pod_info.nft_id,
					price: pod_info.price,
					tips: tips.unwrap_or_default(),
				});

				Ok(())
			} else {
				// POD is rejected. Cleanup storage and free up the NFT
				// Change nft state to Free.
				Self::change_nft_state(pod_info.nft_id, NftState::Free)?;

				Self::deposit_event(Event::<T>::NftPodRejected { nft_id: pod_info.nft_id });
				Ok(())
			}
		}

		#[pallet::call_index(7)]
		#[pallet::weight(T::WeightInfo::cancel_pod())]
		pub fn cancel_pod(origin: OriginFor<T>, pod_id: PodId) -> DispatchResult {
			// Get the account id
			let id = ensure_signed(origin)?;

			// Ensure role is not auditor.
			T::RoleManager::ensure_not_role(&id, Role::Auditor)?;

			let pod_info = PendingPodNfts::<T>::get(pod_id).ok_or(Error::<T>::NftNotForPod)?;

			// Ensure the nft is belong to the correct owner.
			Self::ensure_nft_owner(&id, pod_info.nft_id)?;
			// Change nft state to POD.
			Self::cancel_nft_pod(pod_id, pod_info.nft_id, CancelReason::Canceled)?;

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

		/// Get the trade id to store into the PendingTradeNft.
		pub(crate) fn next_pod_id() -> PodId {
			NextPodId::<T>::mutate(|id| {
				*id = id.wrapping_add(1);
				*id
			})
		}

		fn cancel_nft_pod(pod_id: PodId, nft_id: NftId, reason: CancelReason) -> DispatchResult {
			PendingPodNfts::<T>::remove(pod_id);
			Self::change_nft_state(nft_id, NftState::Free)?;
			Self::deposit_event(Event::<T>::NftPodCanceled { nft_id, reason });

			Ok(())
		}
	}

	impl<T: Config> ManageNfts<T::AccountId> for Pallet<T> {
		fn nft_transfer(
			nft_id: NftId,
			to_user: &T::AccountId,
		) -> Result<T::AccountId, DispatchError> {
			// Ensure role is not auditor.
			T::RoleManager::ensure_not_role(to_user, Role::Auditor)?;

			let owner = Owners::<T>::get(nft_id).ok_or(Error::<T>::InvalidNftId)?;
			// Valid nft and owner
			Self::ensure_nft_owner(&owner, nft_id)?;

			// Transfer Nft ownership to new user.
			Owners::<T>::mutate(nft_id, |user| {
				*user = Some(to_user.clone());
			});

			Ok(owner)
		}

		fn ensure_nft_owner(id: &T::AccountId, nft_id: NftId) -> DispatchResult {
			// Ensure the NftId is valid.
			ensure!(Nfts::<T>::get(nft_id) != None, Error::<T>::InvalidNftId);

			// Ensure the Nft belong to the correct owner.
			ensure!(Owners::<T>::get(nft_id) == Some(id.clone()), Error::<T>::Unauthorised);

			Ok(())
		}

		fn ensure_nft_state(nft_id: NftId, state: NftState) -> DispatchResult {
			match Nfts::<T>::get(nft_id).map(|nft_data| nft_data.state == state) {
				Some(true) => Ok(()),
				Some(false) => Err(Error::<T>::NftStateNotMatch.into()),
				None => Err(Error::<T>::InvalidNftId.into()),
			}
		}

		fn change_nft_state(nft_id: NftId, new_state: NftState) -> DispatchResult {
			Nfts::<T>::mutate_exists(nft_id, |maybe_nft_data| {
				if let Some(nft_data) = maybe_nft_data {
					if new_state != NftState::Free {
						ensure!(nft_data.state == NftState::Free, Error::<T>::NftStateNotFree);
					}
					nft_data.state = new_state;
					Ok(())
				} else {
					Err(Error::<T>::InvalidNftId.into())
				}
			})
		}
	}
}
