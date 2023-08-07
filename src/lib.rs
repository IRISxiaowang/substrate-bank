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

impl<Balance: Saturating + Copy + Ord> AccountData<Balance> {

}

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
pub enum Role {
    Customer,
    Manager,
    Auditor,
}

pub trait ManageRoles<AccountId>{
    fn role(id: &AccountId) -> Option<Role>;
    fn register_role(id: &AccountId, role: Role) -> DispatchResult;
    fn unregister_role(id: &AccountId) -> DispatchResult;
    fn ensure_role(id: &AccountId, role: Role) -> DispatchResult;
}

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
    pub enum Error<T> {
        /// The account role can not changed, you must unregister first.
        AccountAleadyRegistered,
        /// The account hasn't registered a role.
        AccountRoleNotRegistered,
        /// The account role does not equal to the expected role.
        IncorrectRole,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T: Config> {}

    /// The balance of a token type under an account.
    #[pallet::storage]
    #[pallet::getter(fn accounts)]
    pub type Accounts<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, AccountData<T::Balance>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn account_roles)]
    pub type AccountRoles<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, Role>;
    
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

        #[pallet::call_index(1)]
        #[pallet::weight(0)]
        pub fn register(
            origin: OriginFor<T>,
            role: Role,
        ) -> DispatchResult {
            let id = ensure_signed(origin)?;
            Self::register_role(&id, role)
        }

        #[pallet::call_index(2)]
        #[pallet::weight(0)]
        pub fn unregister(
            origin: OriginFor<T>,
        ) -> DispatchResult {
            let id = ensure_signed(origin)?;
            Self::unregister_role(&id)
        }
        
    }

}

impl <T: Config> ManageRoles<T::AccountId> for Pallet<T>{
    fn role(id: &T::AccountId) -> Option<Role> {
        AccountRoles::<T>::get(id)
    }

    fn register_role(id: &T::AccountId, role: Role) -> DispatchResult {
        ensure!(AccountRoles::<T>::get(id).is_none(), Error::<T>::AccountAleadyRegistered);
        AccountRoles::<T>::insert(id, role);
        Ok(())
    }

    fn unregister_role(id: &T::AccountId) -> DispatchResult {
        ensure!(AccountRoles::<T>::get(id).is_some(), Error::<T>::AccountRoleNotRegistered);
        AccountRoles::<T>::remove(id);
        Ok(())
    }

    fn ensure_role(id: &T::AccountId, role: Role) -> DispatchResult {
        match AccountRoles::<T>::get(id) {
            Some(r) => { 
                if r != role {
                   Err(Error::<T>::IncorrectRole.into())
                } else {
                    Ok(())
                }
            },
            None => Err(Error::<T>::AccountRoleNotRegistered.into()),
        }
    }
}

impl<T: Config> Pallet<T> {}
