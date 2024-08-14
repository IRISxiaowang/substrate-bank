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

use primitives::{AuctionId, NftId, NftState};
use traits::{BasicAccounting, GetTreasury, ManageAuctions, ManageNfts, ManageRoles};

mod mock;
mod tests;

pub mod weights;
pub use module::*;
pub use weights::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

/// Represents Auction data including its reserve price, expiry block, current price and bider.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct AuctionData<T: Config> {
	pub nft_id: NftId,
	pub start: Option<T::Balance>,
	pub reserve: Option<T::Balance>,
	pub buy_now: Option<T::Balance>,
	pub expiry_block: BlockNumberFor<T>,
	pub current_bid: Option<(T::AccountId, T::Balance)>,
}

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
enum CancelOption<T: Config> {
	Force,
	ByUser(T::AccountId),
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
		CannotCancelAuction,
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
		BidRegistered { new_bidder: T::AccountId, auction_id: AuctionId, new_price: T::Balance },
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
					if auction_data.expiry_block == block_number {
						Self::resolve_auction(auction_id, auction_data);
					}
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

			// Check the Nft is belong to the origin
			T::NftManager::ensure_nft_owner(&id, nft_id)?;

			// Set the Auctions storage
			let auction_id = Self::next_auction_id();
			let expiry_block =
				T::AuctionLength::get() + frame_system::Pallet::<T>::current_block_number();

			Auctions::<T>::insert(
				auction_id,
				AuctionData { nft_id, start, reserve, buy_now, expiry_block, current_bid: None },
			);

			// Append the auction id to the AuctionsExpiryBlock storage.
			AuctionsExpiryBlock::<T>::append(expiry_block, auction_id);

			// Ensure the nft state is free, then change Nft state to auction.
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
			new_price: T::Balance,
		) -> DispatchResult {
			// Get the new bidder account id
			let new_bidder = ensure_signed(origin)?;

			// update storage auctions
			Auctions::<T>::try_mutate_exists(auction_id, |maybe_auction_data| {
				let auction_data =
					maybe_auction_data.as_mut().ok_or(Error::<T>::InvalidAuctionId)?;
				// Refound money to the last bidder, the bid pool has enough money to transfer back,
				// therefore the transfer will succeed.
				auction_data.current_bid.as_ref().map(|(last_bidder, last_price)| {
					T::Bank::transfer(&T::BidsPoolAccount::get(), last_bidder, *last_price)
				});
				// When the bid price is greater than Buy now, the auction is end.
				if new_price >= auction_data.buy_now.unwrap_or_default() {
					Self::complete_auction(new_bidder, new_price, auction_id, auction_data.nft_id);
					// nft change state
					T::NftManager::change_nft_state(auction_data.nft_id, NftState::Free)?;
					*maybe_auction_data = None;
					Ok(())
				} else {
					let current_price = sp_std::cmp::max(
						if let Some((_, last_price)) = &auction_data.current_bid {
							Ok::<T::Balance, DispatchError>(*last_price)
						} else {
							Ok(Default::default())
						}?,
						auction_data.start.unwrap_or_default(),
					);

					// Ensure bid is valid.
					ensure!(
						new_price >= current_price + T::MinimumIncrease::get(),
						Error::<T>::BidPriceTooLow
					);

					// Transfer bid to Bids Pool's account.
					T::Bank::transfer(&new_bidder, &T::BidsPoolAccount::get(), new_price)?;

					// Update current bid's storage.
					auction_data.current_bid = Some((new_bidder.clone(), new_price));

					// Calculate how many blocks the auction will over, if it shorter than specific
					// length, then extend to specific length.
					let new_expiry = frame_system::Pallet::<T>::current_block_number()
						.saturating_add(T::ExtendedLength::get());

					if new_expiry > auction_data.expiry_block {
						// The old auction id will be handled at on-finalize.
						auction_data.expiry_block = new_expiry;

						// Add the new block number to the storage.
						AuctionsExpiryBlock::<T>::append(new_expiry, auction_id);
					}

					Self::deposit_event(Event::<T>::BidRegistered {
						new_bidder,
						auction_id,
						new_price,
					});

					Ok(())
				}
			})
		}

		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::cancel_auction())]
		pub fn cancel_auction(origin: OriginFor<T>, auction_id: AuctionId) -> DispatchResult {
			// Get the account id
			let id = ensure_signed(origin)?;

			Self::do_cancel_auction(auction_id, CancelOption::ByUser(id))
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
					Self::complete_auction(bider, price, auction_id, auction_data.nft_id);
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

		fn complete_auction(
			bider: T::AccountId,
			price: T::Balance,
			auction_id: AuctionId,
			nft_id: NftId,
		) {
			// transfer money
			let bid_pool = T::BidsPoolAccount::get();
			let tax = T::AuctionSuccessFeePercentage::get() * price;
			let rest = price.saturating_sub(tax);
			if let Ok(treasury) = T::Bank::treasury() {
				let _ = T::Bank::transfer(&bid_pool, &treasury, tax);
			}
			// nft change owner, every Nft must have an owner, so that it must be ok.
			if let Ok(owner) = T::NftManager::nft_transfer(nft_id, &bider) {
				let _ = T::Bank::transfer(&bid_pool, &owner, rest);
			}
			Self::deposit_event(Event::<T>::AuctionSucceeded {
				auction_id,
				to: bider,
				asset: nft_id,
				price,
			});
		}

		fn do_cancel_auction(auction_id: AuctionId, cancel: CancelOption<T>) -> DispatchResult {
			// Read the storage auctions, remove the auction id and return money to bidder.

			Auctions::<T>::take(auction_id)
				.map(|auction_data| {
					// If force cancel the id is none, only normal cancel will check owner of nft.
					if let CancelOption::ByUser(owner) = cancel.clone() {
						T::NftManager::ensure_nft_owner(&owner, auction_data.nft_id)?;
					}

					// Only normal cancel will compare whether current price is upon the reserve
					// price.
					if let Some((bider, price)) = auction_data.current_bid {
						(cancel == CancelOption::Force ||
							price < auction_data.reserve.unwrap_or_default())
						.then(|| T::Bank::transfer(&T::BidsPoolAccount::get(), &bider, price))
						.ok_or(Error::<T>::CannotCancelAuction)??;
						// Transfer back the money to the bider.
					}

					Self::deposit_event(Event::<T>::AuctionCanceled { auction_id });

					// Changer the Nft state to free.
					T::NftManager::change_nft_state(auction_data.nft_id, NftState::Free)
				}) // Result here is Some(Ok())
				.ok_or(Error::<T>::InvalidAuctionId)? // Convert to Ok(Ok()) then use ? to become Ok()
		}
	}

	impl<T: Config> ManageAuctions<T::AccountId> for Pallet<T> {
		fn force_cancel(auction_id: AuctionId) -> DispatchResult {
			Self::do_cancel_auction(auction_id, CancelOption::Force)
		}
	}
}
