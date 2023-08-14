//! # AccountRole Pallet

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{pallet_prelude::*, traits::BuildGenesisConfig};
use frame_system::pallet_prelude::*;
use sp_runtime::DispatchResult;
use sp_std::prelude::*;

use primitives::Role;

mod mock;
mod tests;

mod weights;

pub use weights::WeightInfo;

pub use module::*;

#[frame_support::pallet]
pub mod module {

    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type WeightInfo: WeightInfo;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Example of error.
        TemplateError,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Example Event
        TemplateEvent { user: T::AccountId },
    }

    #[pallet::storage]
    #[pallet::getter(fn example)]
    pub type ExampleStorage<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Role>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        users: Vec<T::AccountId>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self { users: vec![] }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {}
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Register a role for a user.
        ///
        /// This function allows a user to be registered with a specific role.
        /// The user must be signed and authenticated.
        ///
        /// Params:
        /// - `role`: The role to assign to the user.
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::example_extrinsic())]
        pub fn example_extrinsic(origin: OriginFor<T>) -> DispatchResult {
            let id = ensure_signed(origin)?;
            Self::deposit_event(Event::<T>::TemplateEvent { user: id });
            Ok(())
        }
    }
}
