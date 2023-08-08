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

/// Enum representing the different roles that a user can have.
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
pub enum Role {
    /// Represents a regular customer role.
    Customer,
    /// Represents a manager role with higher privileges.
    Manager,
    /// Represents an auditor role responsible for auditing.
    Auditor,
}

/// Trait for managing user roles.
pub trait ManageRoles<AccountId>{
    /// Get the role of a given user.
    fn role(id: &AccountId) -> Option<Role>;
    /// Register a role for a user.
    fn register_role(id: &AccountId, role: Role) -> DispatchResult;
    /// Unregister a role for a user.
    fn unregister_role(id: &AccountId) -> DispatchResult;
    /// Ensure that a user has a specific role.
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
    pub enum Event<T: Config> {
        /// Event emitted when a user's role is registered.
        RoleRegistered{user: T::AccountId, role: Role},

        /// Event emitted when a user's role is unregistered.
        RoleUnregistered{user: T::AccountId},
    }

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

        /// Register a role for a user.
        ///
        /// This function allows a user to be registered with a specific role.
        /// The user must be signed and authenticated.
        ///
        /// - `origin`: The origin of the transaction (signed account).
        /// - `role`: The role to assign to the user.
        ///
        /// Returns `Ok(())` if the user is successfully registered with the role,
        /// or an error if registration fails.
        #[pallet::call_index(1)]
        #[pallet::weight(0)]
        pub fn register(
            origin: OriginFor<T>,
            role: Role,
        ) -> DispatchResult {
            let id = ensure_signed(origin)?;
            Self::register_role(&id, role)
        }

        /// Unregister a role for a user.
        ///
        /// This function allows a user's role to be unregistered.
        /// The user must be signed and authenticated.
        ///
        /// - `origin`: The origin of the transaction (signed account).
        ///
        /// Returns `Ok(())` if the user's role is successfully unregistered,
        /// or an error if unregistration fails.
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
    /// Get the role of a given user.
    fn role(id: &T::AccountId) -> Option<Role> {
        AccountRoles::<T>::get(id)
    }

    /// Register a role for a user, insert the user's role into storage and emit a role_registered event.
    fn register_role(id: &T::AccountId, role: Role) -> DispatchResult {
        ensure!(AccountRoles::<T>::get(id).is_none(), Error::<T>::AccountAleadyRegistered);
        AccountRoles::<T>::insert(id, role);
        Self::deposit_event(Event::<T>::RoleRegistered { user: id.clone(), role });
        Ok(())
    }

    /// Unregister a role for a user, remove the user's role from storage and emit a role unregistered event.
    fn unregister_role(id: &T::AccountId) -> DispatchResult {
        ensure!(AccountRoles::<T>::get(id).is_some(), Error::<T>::AccountRoleNotRegistered);
        AccountRoles::<T>::remove(id);
        Self::deposit_event(Event::<T>::RoleUnregistered { user: id.clone() });
        Ok(())
    }

    /// Ensure that a user has a specific role.
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
