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
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, Default, MaxEncodedLen, RuntimeDebug, TypeInfo)]
pub struct AccountData<Balance> {
    pub free: Balance,
    pub reserved: Balance,
}

impl<Balance: Saturating + Copy + Ord> AccountData<Balance> {}

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
pub trait ManageRoles<AccountId> {
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
            + From<u128>
            + sp_std::iter::Sum;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The account role cannot be changed, you must unregister first.
        AccountAleadyRegistered,
        /// The account hasn't registered a role.
        AccountRoleNotRegistered,
        /// The account role does not equal to the expected role.
        IncorrectRole,
        /// The account role does not have the correct permission.
        UnAuthorised,
        /// The account's free balance is not sufficient.
        InsufficientBalance,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A user has registered a role
        RoleRegistered { user: T::AccountId, role: Role },

        /// A user's role is unregistered.
        RoleUnregistered { user: T::AccountId },

        /// A manager role mint some fund and deposit to an account.
        Deposit { user: T::AccountId, amount: T::Balance },

        /// A manager role burn some fund and withdraw from an account.
        Withdraw { user: T::AccountId, amount: T::Balance },

        /// Burn some fund from an account.
        Burn { user: T::AccountId, amount: T::Balance },

        /// Mint some fund into an account.
        Mint { user: T::AccountId, amount: T::Balance },

        /// Mint some fund into an account.
        Transfer { from: T::AccountId, to: T::AccountId, amount: T::Balance },
    }

    /// The balance of a token type under an account.
    #[pallet::storage]
    #[pallet::getter(fn accounts)]
    pub type Accounts<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, AccountData<T::Balance>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn account_roles)]
    pub type AccountRoles<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Role>;

    #[pallet::storage]
    #[pallet::getter(fn total_issuance)]
    /// Storage item to track the total issuance of the token.
    pub type TotalIssuance<T: Config> = StorageValue<_, T::Balance, ValueQuery>;

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
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        /// Integrity check: Ensure that the sum of all funds in balances matches total_issuance.
        fn integrity_test() {
            let total_issuance = TotalIssuance::<T>::get();

            let actual_total = Accounts::<T>::iter().map(|(_, account)| account.free + account.reserved).sum();

            assert!(
                total_issuance == actual_total,
                "Integrity check failed: total_issuance does not match sum of balances."
            );
    }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Mint some fund and deposit into user's account.
        ///
        /// Requires Manager.
        #[pallet::call_index(0)]
        #[pallet::weight(0)]
        pub fn deposit(
            origin: OriginFor<T>,
            user: T::AccountId, 
            #[pallet::compact] amount: T::Balance,
        ) -> DispatchResult {
            let id = ensure_signed(origin)?;
            Self::ensure_role(&id, Role::Manager)?;

            Self::mint(&user, amount)?;
            Self::deposit_event(Event::<T>::Deposit { user: user.clone(), amount,} );
            Ok(())
        }

        /// Withdraw from user's account and the withdrew funds are burned.
        ///
        /// Requires Manager.
        #[pallet::call_index(1)]
        #[pallet::weight(0)]
        pub fn withdraw(
            origin: OriginFor<T>,
            user: T::AccountId,
            #[pallet::compact] amount: T::Balance,
        ) -> DispatchResult {
            let id = ensure_signed(origin)?;
            Self::ensure_role(&id, Role::Manager)?;
            
            Self::burn(&user, amount)?;
            Self::deposit_event(Event::<T>::Withdraw { user: user.clone(), amount,} );
            Ok(())
        }

        /// Transfer `amount` of fund from the current user to another user.
        #[pallet::call_index(2)]
        #[pallet::weight(0)]
        pub fn transfer(
            origin: OriginFor<T>,
            to_user: T::AccountId,
            #[pallet::compact] amount: T::Balance,
        ) -> DispatchResult{
            let id = ensure_signed(origin)?;
            Self::ensure_role(&id, Role::Customer)?;
            Self::ensure_role(&to_user, Role::Customer)?;

            Accounts::<T>::mutate(&id, |balance| -> DispatchResult{
                if balance.free >= amount {
                    balance.free -= amount;
                    Ok(())
                }else{
                    Err(Error::<T>::InsufficientBalance.into())
                }
            })?;
            Accounts::<T>::mutate(&to_user, |balance|{
                balance.free = balance.free.saturating_add(amount);
            });
            Self::deposit_event(Event::Transfer { from: id.clone(), to: to_user.clone(), amount });
            Ok(())
        }

        /// Register a role for a user.
        ///
        /// This function allows a user to be registered with a specific role.
        /// The user must be signed and authenticated.
        ///
        /// Params:
        /// - `role`: The role to assign to the user.
        #[pallet::call_index(3)]
        #[pallet::weight(0)]
        pub fn register(origin: OriginFor<T>, role: Role) -> DispatchResult {
            let id = ensure_signed(origin)?;
            Self::register_role(&id, role)
        }

        /// Unregister a role for a user.
        ///
        /// This function allows a user's role to be unregistered.
        /// The user must be signed and authenticated.
        #[pallet::call_index(4)]
        #[pallet::weight(0)]
        pub fn unregister(origin: OriginFor<T>) -> DispatchResult {
            let id = ensure_signed(origin)?;
            Self::unregister_role(&id)
        }
    }
}

impl<T: Config> ManageRoles<T::AccountId> for Pallet<T> {
    /// Get the role of a given user.
    fn role(id: &T::AccountId) -> Option<Role> {
        AccountRoles::<T>::get(id)
    }

    /// Register a role for a user, insert the user's role into storage and emit a role_registered event.
    fn register_role(id: &T::AccountId, role: Role) -> DispatchResult {
        ensure!(
            AccountRoles::<T>::get(id).is_none(),
            Error::<T>::AccountAleadyRegistered
        );
        AccountRoles::<T>::insert(id, role);
        Self::deposit_event(Event::<T>::RoleRegistered {
            user: id.clone(),
            role,
        });
        Ok(())
    }

    /// Unregister a role for a user, remove the user's role from storage and emit a role unregistered event.
    fn unregister_role(id: &T::AccountId) -> DispatchResult {
        ensure!(
            AccountRoles::<T>::get(id).is_some(),
            Error::<T>::AccountRoleNotRegistered
        );
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
            }
            None => Err(Error::<T>::AccountRoleNotRegistered.into()),
        }
    }
}

impl<T: Config> Pallet<T> {
    /// Burn some fund from a user's account.
    fn burn(user: &T::AccountId, amount: T::Balance) -> DispatchResult{
        Self::ensure_role(&user, Role::Customer)?;
        Accounts::<T>::mutate(&user, |balance| -> DispatchResult {
            if balance.free >= amount{
                balance.free -= amount;
                Ok(())
            }else{
                Err(Error::<T>::InsufficientBalance.into())
            }
        })?;
        TotalIssuance::<T>::mutate(|total|{
            *total = total.saturating_sub(amount);
        });
        Self::deposit_event(Event::Burn { user: user.clone(), amount });
        Ok(())
    }

    /// Mint some fund into a user's account.
    fn mint(user: &T::AccountId, amount: T::Balance) -> DispatchResult{
        Self::ensure_role(&user, Role::Customer)?;

        Accounts::<T>::mutate(&user, |balance| {
            balance.free = balance.free.saturating_add(amount);
        });
        TotalIssuance::<T>::mutate(|total|{
            *total = total.saturating_add(amount);
        });
        Self::deposit_event(Event::Mint { user: user.clone(), amount });
        Ok(())
    }

}
