//! # Auction Pallet
//!
//! This pallet provides functionality for creating, canceling and biding auction for non-fungible
//! tokens (NFTs). It supports role-based access control and integrates with a role management
//! system.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;

use sp_runtime::{
	traits::{AtLeast32BitUnsigned, BlockNumberProvider, Saturating},
	DispatchResult, Percent,
};
use sp_std::{fmt::Debug, prelude::*, vec::Vec};

use primitives::{AuctionId, NftId, NftState, Role};
use traits::{BasicAccounting, GetTreasury, ManageAuctions, ManageNfts, ManageRoles};

//mod mock;
//mod test

pub mod weights;
pub use module::*;
pub use weights::*;

//#[cfg(feature = "runtime-benchmarks")]
//mod benchmarking;

/// Represents Auction data including its reserve price, expiry block, current price and bider.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct AuctionData<T: Config> {
	pub nft_id: NftId,
	pub owner: T::AccountId,
	pub start: Option<T::Balance>,
	pub reserve: Option<T::Balance>,
	pub buy_now: Option<T::Balance>,
	pub expiry_block: BlockNumberFor<T>,
	pub current_bid: Option<(T::AccountId, T::Balance)>,
}

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

		type NftManager: ManageNfts<Self::AccountId>;

		#[pallet::constant]
		type BidsPoolAccount: Get<Self::AccountId>;

		#[pallet::constant]
		type AuctionSuccessFeePercentage: Get<Percent>;

		#[pallet::constant]
		type AuctionStartFee: Get<Self::Balance>;

		#[pallet::constant]
		type MinimumIncrease: Get<Self::Balance>;

		#[pallet::constant]
		type AuctionLength: Get<BlockNumberFor<Self>>;

		#[pallet::constant]
		type ExtendedLength: Get<BlockNumberFor<Self>>;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The specified Auction ID does not exist.
		InvalidAuctionId,
		/// The bid price is smaller than the current bid price.
		BidPriceTooLow,
		/// When the current auction price exceeds the reserve price, the auction can not be
		/// canceled.
		AuctionCancelFailed,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Emitted when an auction has been created and is open for bidding.
		AuctionCreated { who: T::AccountId, auction_id: AuctionId },

		/// Emitted when an auction has been canceled by its owner.
		AuctionCanceled { auction_id: AuctionId },

		/// Emitted when an auction has expired without meeting the reserve price.
		AuctionExpired { auction_id: AuctionId },

		/// Emitted when an auction has concluded successfully with a winning bid.
		AuctionSucceeded {
			auction_id: AuctionId,
			to: T::AccountId,
			asset: NftId,
			price: T::Balance,
		},

		/// Emitted when a new bid is placed on an auction.
		Bid { who: T::AccountId, auction_id: AuctionId, price: T::Balance },
	}

	#[pallet::storage]
	#[pallet::getter(fn next_auction)]
	pub type NextAuctionId<T: Config> = StorageValue<_, AuctionId, ValueQuery>;

	/// Stores the data associated with each auction.
	#[pallet::storage]
	#[pallet::getter(fn auctions)]
	pub type Auctions<T: Config> = StorageMap<_, Blake2_128Concat, AuctionId, AuctionData<T>>;

	/// Stores the auction IDs that are set to expire at a specific block.
	#[pallet::storage]
	pub type AuctionsExpiryBlock<T: Config> =
		StorageMap<_, Blake2_128Concat, BlockNumberFor<T>, Vec<AuctionId>, ValueQuery>;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(block_number: BlockNumberFor<T>) {
			// Expire auctions that are due.
			AuctionsExpiryBlock::<T>::take(block_number).into_iter().for_each(|auction_id| {
				if let Some(auction_data) = Auctions::<T>::take(auction_id) {
					Self::resolve_auction(auction_id, auction_data);
				}
			});
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Creates a new auction for a specified NFT.
		/// This function can only be called by an account with the Customer role.
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::create_auction())]
		pub fn create_auction(
			origin: OriginFor<T>,
			nft_id: NftId,
			start: Option<T::Balance>,
			reserve: Option<T::Balance>,
			buy_now: Option<T::Balance>,
		) -> DispatchResult {
			// Get the account id
			let id = ensure_signed(origin)?;

			// Ensure role is customer.
			T::RoleManager::ensure_role(&id, Role::Customer)?;

			// check the NftIdState is Free
			T::NftManager::ensure_nft_state(nft_id, NftState::Free)?;

			// Check the Nft is belong to the origin
			T::NftManager::ensure_nft_is_valid(&id, nft_id)?;

			// Set the Auctions storage
			let auction_id = Self::next_auction_id();
			let expiry_block =
				T::AuctionLength::get() + frame_system::Pallet::<T>::current_block_number();

			Auctions::<T>::insert(
				auction_id,
				AuctionData {
					nft_id,
					owner: id.clone(),
					start,
					reserve,
					buy_now,
					expiry_block,
					current_bid: None,
				},
			);

			// Append the auction id to the AuctionsExpiryBlock storage.
			AuctionsExpiryBlock::<T>::append(expiry_block, auction_id);

			// Change Nft state to auction.
			T::NftManager::change_nft_state(nft_id, NftState::Auction(auction_id))?;

			// Pay fee.
			T::Bank::transfer(&id, &T::Bank::treasury()?, T::AuctionStartFee::get())?;

			Self::deposit_event(Event::<T>::AuctionCreated { who: id, auction_id });
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::bid())]
		pub fn bid(
			origin: OriginFor<T>,
			auction_id: AuctionId,
			price: T::Balance,
		) -> DispatchResult {
			// Get the account id
			let id = ensure_signed(origin)?;

			// Ensure role is customer.
			T::RoleManager::ensure_role(&id, Role::Customer)?;

			// update storage auctions
			Auctions::<T>::try_mutate(auction_id, |maybe_auction_data| {
				if let Some(auction_data) = maybe_auction_data.as_mut() {
					let current_price = sp_std::cmp::max(
						if let Some((last_bidder, last_price)) = &auction_data.current_bid {
							// Refund the previous bidder's bid
							T::Bank::transfer(
								&T::BidsPoolAccount::get(),
								last_bidder,
								*last_price,
							)?;
							Ok::<T::Balance, DispatchError>(*last_price)
						} else {
							Ok(Default::default())
						}?,
						auction_data.start.unwrap_or_default(),
					);

					// Ensure bid is valid.
					ensure!(
						price >= current_price + T::MinimumIncrease::get(),
						Error::<T>::BidPriceTooLow
					);

					// Transfer bid to Bids Pool's account.
					T::Bank::transfer(&id, &T::BidsPoolAccount::get(), price)?;

					// Update current bid's storage.
					auction_data.current_bid = Some((id.clone(), price));

					// Calculate how many blocks the auction will over, if it shorter than specific
					// length, then extend to specific length.
					if T::ExtendedLength::get() >
						auction_data.expiry_block -
							frame_system::Pallet::<T>::current_block_number()
					{
						// Remove the old auction id
						AuctionsExpiryBlock::<T>::mutate(auction_data.expiry_block, |auctions| {
							auctions.retain(|id| *id != auction_id);
						});

						let new_expiry = frame_system::Pallet::<T>::current_block_number()
							.saturating_add(T::ExtendedLength::get());
						auction_data.expiry_block = new_expiry;

						// Add the new block number to the storage.
						AuctionsExpiryBlock::<T>::append(new_expiry, auction_id);
					}

					Self::deposit_event(Event::<T>::Bid { who: id, auction_id, price });

					Ok(())
				} else {
					Err(Error::<T>::InvalidAuctionId.into())
				}
			})
		}

		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::cancel_auction())]
		pub fn cancel_auction(origin: OriginFor<T>, auction_id: AuctionId) -> DispatchResult {
			// Get the account id
			let id = ensure_signed(origin)?;

			// Ensure role is customer.
			T::RoleManager::ensure_role(&id, Role::Customer)?;

			// todo read the storage auctions
			let auction_data =
				Auctions::<T>::get(auction_id).ok_or(Error::<T>::InvalidAuctionId)?;

			// Ensure the nft is belong to the correct owner.
			T::NftManager::ensure_nft_is_valid(&id, auction_data.nft_id)?;

			// Compare whether current price is upon the reserve price.
			if let Some((bider, price)) = auction_data.current_bid {
				if price >= auction_data.reserve.unwrap_or_default() {
					return Err(Error::<T>::AuctionCancelFailed.into());
				} else {
					// Transfer back the money to the bider.
					T::Bank::transfer(&T::BidsPoolAccount::get(), &bider, price)?;
				}
			}
			// Changer the Nft state to free.
			T::NftManager::change_nft_state(auction_data.nft_id, NftState::Free)?;
			// Remove the auction from storage Auctions.
			Auctions::<T>::remove(auction_id);

			Self::deposit_event(Event::<T>::AuctionCanceled { auction_id });

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Get the auction id to store into the NextAuctionId.
		pub(crate) fn next_auction_id() -> AuctionId {
			NextAuctionId::<T>::mutate(|id| {
				*id = id.wrapping_add(1);
				*id
			})
		}

		/// Get the auction done.
		fn resolve_auction(auction_id: AuctionId, auction_data: AuctionData<T>) {
			if let Some((bider, price)) = auction_data.current_bid {
				let bid_pool = T::BidsPoolAccount::get();

				if price >= auction_data.reserve.unwrap_or_default() {
					// transfer money
					let tax = T::AuctionSuccessFeePercentage::get() * price;
					let rest = price.saturating_sub(tax);
					let _ = T::Bank::transfer(&bid_pool, &auction_data.owner, rest);
					if let Ok(treasury) = T::Bank::treasury() {
						let _ = T::Bank::transfer(&bid_pool, &treasury, tax);
					}
					// nft change owner
					let _ = T::NftManager::nft_transfer(auction_data.nft_id, &bider);
					Self::deposit_event(Event::<T>::AuctionSucceeded {
						auction_id,
						to: bider,
						asset: auction_data.nft_id,
						price,
					});
				} else {
					// return money to last bider
					let _ = T::Bank::transfer(&bid_pool, &bider, price);
					Self::deposit_event(Event::<T>::AuctionExpired { auction_id });
				}
			} else {
				Self::deposit_event(Event::<T>::AuctionExpired { auction_id });
			}

			// nft change state
			let _ = T::NftManager::change_nft_state(auction_data.nft_id, NftState::Free);
		}
	}

	impl<T: Config> ManageAuctions<T::AccountId> for Pallet<T> {
		fn force_cancel(auction_id: AuctionId) -> DispatchResult {
			if let Some(auction_data) = Auctions::<T>::take(auction_id) {
				if let Some((bider, price)) = auction_data.current_bid {
					T::Bank::transfer(&T::BidsPoolAccount::get(), &bider, price)?;
				}
			}
			Ok(())
		}
	}
}
