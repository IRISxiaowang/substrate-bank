//! # NFT Pallet

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{pallet_prelude::*, traits::BuildGenesisConfig};
use frame_system::pallet_prelude::*;
use sp_runtime::DispatchResult;
use sp_std::{marker::PhantomData, prelude::*, vec::Vec};

use primitives::{NftId, Role};
use traits::ManageRoles;

// mod mock;
// mod tests;

pub mod weights;
pub use weights::*;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

/// Stores Nft data
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct NftData<NftId> {
	pub nft_id: NftId,
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

		/// Type representing the weight of this pallet
		type WeightInfo: WeightInfo;

		type RoleManager: ManageRoles<Self::AccountId>;

		type EnsureGovernance: EnsureOrigin<<Self as frame_system::Config>::RuntimeOrigin>;

		#[pallet::constant]
		type MaxSize: Get<u32>;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The Nft is not belong the user who want to deal with it.
		Unauthorised,
		/// The account role does not equal to the expected role.
		IncorrectRole,
		/// The Nft Id should exist.
		InvalidNftId,
		/// Tax rate must be between 0% - 100%.
		InvalidTaxRate,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Created an Nft.
		NftMinted { who: T::AccountId, nft_id: NftId },
		/// Burnt an Nft.
		NftBurned { who: T::AccountId, nft_id: NftId },
		/// Transferred an Nft.
		NftTransferred { from: T::AccountId, to: T::AccountId, nft_id: NftId },
		/// An Nft created.
		NFTPending { nft_id: NftId, file_name: Vec<u8> },
		/// Auditor audited an nft pass or fail.
		NftAudited { nft_id: NftId, result: bool },
	}

	#[pallet::storage]
	#[pallet::getter(fn next_nft)]
	pub type NextNftId<T: Config> = StorageValue<_, NftId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn pending_nft)]
	pub type PendingNft<T: Config> =
		StorageMap<_, Blake2_128Concat, NftId, (NftData<NftId>, T::AccountId)>;
	#[pallet::storage]
	#[pallet::getter(fn owners)]
	pub type Owners<T: Config> = StorageMap<_, Blake2_128Concat, NftId, T::AccountId>;

	#[pallet::storage]
	#[pallet::getter(fn nfts)]
	pub type Nfts<T: Config> = StorageMap<_, Blake2_128Concat, NftId, NftData<NftId>>;

	#[pallet::genesis_config]
	#[derive(frame_support::DefaultNoBound)]
	pub struct GenesisConfig<T: Config> {
		phantom: PhantomData<T>,
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {}
	}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create an Nft which pending for auditor checking.
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
			ensure!(T::RoleManager::role(&id) != Some(Role::Auditor), Error::<T>::IncorrectRole);

			// Set the Nfts storage
			let nft_id = Self::next_nft_id();

			PendingNft::<T>::insert(
				nft_id,
				(NftData { nft_id, data, file_name: file_name.clone() }, id.clone()),
			);

			Self::deposit_event(Event::<T>::NFTPending { nft_id, file_name });
			Ok(())
		}

		/// Burn an Nft.
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::burned())]
		pub fn burned(origin: OriginFor<T>, nft_id: NftId) -> DispatchResult {
			// Get the account id
			let id = ensure_signed(origin)?;

			// Valid nft and owner
			Self::ensure_nft_is_valid(id.clone(), nft_id)?;

			// Remove storage
			Nfts::<T>::remove(nft_id);
			Owners::<T>::remove(nft_id);

			Self::deposit_event(Event::<T>::NftBurned { who: id, nft_id });
			Ok(())
		}

		/// Transfer an Nft.
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::transfer())]
		pub fn transfer(
			origin: OriginFor<T>,
			to_user: T::AccountId,
			nft_id: NftId,
		) -> DispatchResult {
			// Get the account id
			let id = ensure_signed(origin)?;

			// Valid nft and owner
			Self::ensure_nft_is_valid(id.clone(), nft_id)?;

			// Ensure role is not auditor.
			ensure!(
				T::RoleManager::role(&to_user) != Some(Role::Auditor),
				Error::<T>::IncorrectRole
			);

			// Transfer Nft ownership to new user.
			Owners::<T>::mutate(nft_id, |user| {
				*user = Some(to_user.clone());
			});

			Self::deposit_event(Event::<T>::NftTransferred { from: id, to: to_user, nft_id });
			Ok(())
		}

		/// Audit an Nft pass or fail.
		///
		/// Required Auditor.
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::approve_nft())]
		pub fn approve_nft(origin: OriginFor<T>, nft_id: NftId, result: bool) -> DispatchResult {
			// Get the account id
			let id = ensure_signed(origin)?;

			// Ensure auditor role.
			T::RoleManager::ensure_role(&id, Role::Auditor)?;

			if result {
				if let Some((nft_data, user)) = PendingNft::<T>::take(nft_id) {
					Owners::<T>::insert(nft_id, user.clone());
					Nfts::<T>::insert(nft_id, nft_data);
					Self::deposit_event(Event::<T>::NftMinted { who: user, nft_id });
				}
			} else {
				PendingNft::<T>::remove(nft_id);
			}

			Self::deposit_event(Event::<T>::NftAudited { nft_id, result });
			Ok(())
		}

		/// Force burn an Nft.
		///
		/// Required governance.
		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::force_burn())]
		pub fn force_burn(origin: OriginFor<T>, nft_id: NftId) -> DispatchResult {
			// ensure governance
			T::EnsureGovernance::ensure_origin(origin)?;

			// Remove storage
			Nfts::<T>::remove(nft_id);
			if let Some(user) = Owners::<T>::take(nft_id) {
				Self::deposit_event(Event::<T>::NftBurned { who: user, nft_id });
			};

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Get the nft id to store into the Nfts.
		fn next_nft_id() -> NftId {
			NextNftId::<T>::mutate(|id| {
				*id = id.wrapping_add(1);
				*id
			})
		}

		fn ensure_nft_is_valid(id: T::AccountId, nft_id: NftId) -> DispatchResult {
			// Ensure the NftId is valid.
			ensure!(Nfts::<T>::get(nft_id) != None, Error::<T>::InvalidNftId);

			// Ensure the Nft belong to the correct owner.
			ensure!(Owners::<T>::get(nft_id) == Some(id), Error::<T>::Unauthorised);

			Ok(())
		}
	}
}
