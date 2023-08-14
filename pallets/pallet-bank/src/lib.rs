//! # Bank Pallet

#![cfg_attr(not(feature = "std"), no_std)]

use codec::MaxEncodedLen;
use frame_support::{pallet_prelude::*, traits::BuildGenesisConfig};
use frame_system::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_arithmetic::traits::Zero;
use sp_runtime::{
    traits::{AtLeast32BitUnsigned, Saturating},
    DispatchResult,
};
use sp_std::{fmt::Debug, prelude::*, vec::Vec};

use primitives::Role;
use traits::{BasicAccounting, ManageRoles};

mod mock;
mod tests;

mod weights;

pub use weights::WeightInfo;

/// balance information for an account.
#[derive(
    Encode, Decode, Copy, Clone, PartialEq, Eq, Default, MaxEncodedLen, RuntimeDebug, TypeInfo,
)]
pub struct AccountData<Balance> {
    pub free: Balance,
    pub reserved: Balance,
}

impl<Balance: Saturating + Copy> AccountData<Balance> {
    pub fn total(&self) -> Balance {
        self.free.saturating_add(self.reserved)
    }
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

        type RoleManager: ManageRoles<Self::AccountId>;

        #[pallet::constant]
        type ExistentialDeposit: Get<Self::Balance>;

        #[pallet::constant]
        type TreasuryAccount: Get<Self::AccountId>;

        #[pallet::constant]
        type MinimumAmount: Get<Self::Balance>;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The account's free balance is not sufficient.
        InsufficientBalance,
        /// The amount given is too small.
        AmountTooSmall,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A manager role has minted some funds into an account.
        Deposit {
            user: T::AccountId,
            amount: T::Balance,
        },

        /// A manager role has burned some fund from an account.
        Withdraw {
            user: T::AccountId,
            amount: T::Balance,
        },

        /// Transfered some fund from an account into another account.
        Transfer {
            from: T::AccountId,
            to: T::AccountId,
            amount: T::Balance,
        },

        /// Reaped some fund from an account and removed this account.
        Reaped {
            user: T::AccountId,
            dust: T::Balance,
        },
    }

    /// The balance of a token type under an account.
    #[pallet::storage]
    #[pallet::getter(fn accounts)]
    pub type Accounts<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, AccountData<T::Balance>, ValueQuery>;

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
            let total: T::Balance = self
                .balances
                .iter()
                .map(|(account_id, initial_balance)| {
                    // assert!(
                    // 	*initial_balance >= T::ExistentialDeposits::get(),
                    // 	"the balance of any account should always be more than existential deposit.",
                    // );
                    let _ = T::RoleManager::register_role(&account_id, Role::Customer);
                    Accounts::<T>::mutate(account_id, |account_data| {
                        account_data.free = *initial_balance
                    });
                    *initial_balance
                })
                .sum();
            TotalIssuance::<T>::set(total);
        }
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn integrity_test() {
            // Check if the minimum deposit is greater than or equal to the existential deposit
            assert!(T::MinimumAmount::get() >= T::ExistentialDeposit::get());
        }

        fn on_finalize(_block_number: BlockNumberFor<T>) {
            Self::reap_accounts();
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
            T::RoleManager::ensure_role(&id, Role::Manager)?;
            <Self as BasicAccounting<T::AccountId, T::Balance>>::deposit(&user, amount)
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
            T::RoleManager::ensure_role(&id, Role::Manager)?;
            <Self as BasicAccounting<T::AccountId, T::Balance>>::withdraw(&user, amount)
        }

        /// Transfer `amount` of fund from the current user to another user.
        #[pallet::call_index(2)]
        #[pallet::weight(0)]
        pub fn transfer(
            origin: OriginFor<T>,
            to_user: T::AccountId,
            #[pallet::compact] amount: T::Balance,
        ) -> DispatchResult {
            let id = ensure_signed(origin)?;
            T::RoleManager::ensure_role(&id, Role::Customer)?;
            T::RoleManager::ensure_role(&to_user, Role::Customer)?;
            <Self as BasicAccounting<T::AccountId, T::Balance>>::transfer(&id, &to_user, amount)
        }
    }
}

impl<T: Config> BasicAccounting<T::AccountId, T::Balance> for Pallet<T> {
    fn deposit(user: &T::AccountId, amount: T::Balance) -> DispatchResult {
        if amount < T::MinimumAmount::get() {
            return Err(Error::<T>::AmountTooSmall.into());
        }
        Self::mint(&user, amount)?;
        Self::deposit_event(Event::<T>::Deposit {
            user: user.clone(),
            amount,
        });
        Ok(())
    }

    fn withdraw(user: &T::AccountId, amount: T::Balance) -> DispatchResult {
        if amount < T::MinimumAmount::get() {
            return Err(Error::<T>::AmountTooSmall.into());
        }
        Self::burn(&user, amount)?;
        Self::deposit_event(Event::<T>::Withdraw {
            user: user.clone(),
            amount,
        });
        Ok(())
    }

    fn transfer(from: &T::AccountId, to: &T::AccountId, amount: T::Balance) -> DispatchResult {
        if amount < T::MinimumAmount::get() {
            return Err(Error::<T>::AmountTooSmall.into());
        }
        Accounts::<T>::mutate(&from, |balance| -> DispatchResult {
            if balance.free >= amount {
                balance.free -= amount;
                Ok(())
            } else {
                Err(Error::<T>::InsufficientBalance.into())
            }
        })?;
        Accounts::<T>::mutate(&to, |balance| {
            balance.free = balance.free.saturating_add(amount);
        });
        Self::deposit_event(Event::Transfer {
            from: from.clone(),
            to: to.clone(),
            amount,
        });
        Ok(())
    }
}

impl<T: Config> Pallet<T> {
    /// Burn some fund from a user's account.
    fn burn(user: &T::AccountId, amount: T::Balance) -> DispatchResult {
        T::RoleManager::ensure_role(&user, Role::Customer)?;
        Accounts::<T>::mutate(&user, |balance| -> DispatchResult {
            if balance.free >= amount {
                balance.free -= amount;
                Ok(())
            } else {
                Err(Error::<T>::InsufficientBalance.into())
            }
        })?;
        TotalIssuance::<T>::mutate(|total| {
            *total = total.saturating_sub(amount);
        });
        Ok(())
    }

    /// Mint some fund into a user's account.
    fn mint(user: &T::AccountId, amount: T::Balance) -> DispatchResult {
        T::RoleManager::ensure_role(&user, Role::Customer)?;

        Accounts::<T>::mutate(&user, |balance| {
            balance.free = balance.free.saturating_add(amount);
        });
        TotalIssuance::<T>::mutate(|total| {
            *total = total.saturating_add(amount);
        });
        Ok(())
    }

    #[cfg(test)]
    /// Integrity check: Ensure that the sum of all funds in balances matches total_issuance.
    fn check_total_issuance() -> bool {
        TotalIssuance::<T>::get()
            == Accounts::<T>::iter()
                .map(|(_, account)| account.free + account.reserved)
                .sum()
    }

    /// Reaps funds from accounts that have balances below the Existential Deposit (ED).
    /// Reaped funds are transferred to the Treasury account.
    fn reap_accounts() {
        let total_reaped_amount = Accounts::<T>::iter()
            .filter(|(_id, balance)| balance.total() < T::ExistentialDeposit::get())
            .map(|(id, balance)| {
                Self::deposit_event(Event::Reaped {
                    user: id.clone(),
                    dust: balance.total(),
                });
                Accounts::<T>::remove(&id);
                balance.total()
            })
            .sum();

        if total_reaped_amount > Zero::zero() {
            Accounts::<T>::mutate(&T::TreasuryAccount::get(), |treasury_account| {
                treasury_account.free += total_reaped_amount;
            });
        }
    }
}
