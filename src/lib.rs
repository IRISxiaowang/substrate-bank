//! # Bank Pallet

#![cfg_attr(not(feature = "std"), no_std)]

use codec::MaxEncodedLen;
use frame_support::{pallet_prelude::*, traits::BuildGenesisConfig};
use frame_system::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_runtime::{
    traits::{AtLeast32BitUnsigned, Saturating},
    DispatchResult,
};
use sp_std::{fmt::Debug, prelude::*, vec::Vec};

mod mock;
mod tests;

mod weights;

pub use weights::WeightInfo;

/// balance information for an account.
#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, MaxEncodedLen, RuntimeDebug, TypeInfo)]
pub struct AccountData<Balance> {
    pub free: Balance,
    pub reserved: Balance,
}

impl<Balance: Saturating + Copy + Ord> AccountData<Balance> {}

pub use module::*;

#[frame_support::pallet]
pub mod module {
    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The balance type
        type Balance: Member
            + Parameter
            + MaxEncodedLen
            + AtLeast32BitUnsigned
            + Default
            + Copy
            + MaybeSerializeDeserialize
            + Debug
            + From<u128>;
    }

    #[pallet::error]
    pub enum Error<T> {}

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T: Config> {}

    /// The balance of a token type under an account.
    #[pallet::storage]
    #[pallet::getter(fn accounts)]
    pub type Accounts<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, AccountData<T::Balance>, ValueQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub balances: Vec<(T::AccountId, T::Balance)>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            GenesisConfig { balances: vec![] }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            self.balances
                .iter()
                .for_each(|(account_id, initial_balance)| {
                    // assert!(
                    // 	*initial_balance >= T::ExistentialDeposits::get(),
                    // 	"the balance of any account should always be more than existential deposit.",
                    // );
                    Accounts::<T>::mutate(account_id, |account_data| {
                        account_data.free = *initial_balance
                    });
                });
        }
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Mint some fund and deposit into user's account.
        ///
        /// Requires Root.
        #[pallet::call_index(0)]
        #[pallet::weight(0)]
        pub fn deposit(
            _origin: OriginFor<T>,
            #[pallet::compact] amount: T::Balance,
        ) -> DispatchResult {
            // Ensure root

            Ok(())
        }
    }
}

impl<T: Config> Pallet<T> {}
