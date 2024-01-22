//! # Bank Pallet

#![cfg_attr(not(feature = "std"), no_std)]

use codec::MaxEncodedLen;
use frame_support::{pallet_prelude::*, traits::BuildGenesisConfig};
use frame_system::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_arithmetic::traits::Zero;
use sp_runtime::{
    traits::{AtLeast32BitUnsigned, BlockNumberProvider, Saturating},
    DispatchResult, Perbill,
};
use sp_std::{cmp::min, fmt::Debug, prelude::*, vec::Vec};

use primitives::{LockId, Role};
use traits::{BasicAccounting, ManageRoles, Stakable};

mod mock;
mod tests;

mod weights;

pub use weights::WeightInfo;

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
pub enum LockReason {
    Stake,
    Redeem,
    Auditor,
}

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
pub enum UnlockReason {
    Expired,
    Auditor,
}

/// Stores locked funds.
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
pub struct LockedFund<Balance> {
    pub id: LockId,
    pub amount: Balance,
    pub reason: LockReason,
}

/// balance information for an account.
#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, MaxEncodedLen, RuntimeDebug, TypeInfo)]
pub struct AccountData<Balance> {
    pub free: Balance,
    pub reserved: Balance,
    pub locked: Vec<LockedFund<Balance>>,
}

impl<Balance: Saturating + Copy + sp_std::iter::Sum> AccountData<Balance> {
    pub fn total(&self) -> Balance {
        self.free
            .saturating_add(self.reserved)
            .saturating_add(self.locked.iter().map(|l| l.amount).sum())
    }
}

pub use module::*;

#[frame_support::pallet]
pub mod module {
    use sp_runtime::traits::BlockNumberProvider;

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

        type BlockNumberProvider: BlockNumberProvider<BlockNumber = BlockNumberFor<Self>>;

        #[pallet::constant]
        type ExistentialDeposit: Get<Self::Balance>;

        #[pallet::constant]
        type TreasuryAccount: Get<Self::AccountId>;

        #[pallet::constant]
        type MinimumAmount: Get<Self::Balance>;

        #[pallet::constant]
        type RedeemPeriod: Get<BlockNumberFor<Self>>;

        #[pallet::constant]
        type StakePeriod: Get<BlockNumberFor<Self>>;

        #[pallet::constant]
        type InterestPayoutPeriod: Get<BlockNumberFor<Self>>;

        #[pallet::constant]
        type TotalBlocksPerYear: Get<BlockNumberFor<Self>>;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The account's free balance is not sufficient.
        InsufficientBalance,
        /// The amount given is too small.
        AmountTooSmall,
        /// Unlock reason is not compatible with the lock.
        UnauthorisedUnlock,
        /// Interest rate must be between 0 - 10000(0% - 100%).
        InvalidInterestRate,
        /// No lock corresponds to the given lock Id.
        InvalidLockId,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A manager role has minted some funds into an account.
        Deposited {
            user: T::AccountId,
            amount: T::Balance,
        },

        /// A manager role has burned some fund from an account.
        Withdrew {
            user: T::AccountId,
            amount: T::Balance,
        },

        /// Transfered some fund from an account into another account.
        Transferred {
            from: T::AccountId,
            to: T::AccountId,
            amount: T::Balance,
        },

        /// Reaped some fund from an account and removed this account.
        Reaped {
            user: T::AccountId,
            dust: T::Balance,
        },

        /// Auditor or client locked some fund from an account's "free" and "reserved" to "locked".
        Locked {
            user: T::AccountId,
            amount: T::Balance,
            length: BlockNumberFor<T>,
            reason: LockReason,
        },

        /// Auditor or client unlocked some fund from an account's "locked" to "free".
        Unlocked {
            user: T::AccountId,
            amount: T::Balance,
            reason: UnlockReason,
        },

        /// Manager set the interest rate.
        InterestRateSet {
            manager: T::AccountId,
            old_interest_rate: Perbill,
            new_interest_rate: Perbill,
        },

        /// Manager paid total interest.
        InterestPayed {
            interest_rate: Perbill,
            total_interest_payed: T::Balance,
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

    /// Stores the user ID that will have their fund unlocked at a black.
    #[pallet::storage]
    pub type AccountWithUnlockedFund<T: Config> =
        StorageMap<_, Blake2_128Concat, BlockNumberFor<T>, Vec<(T::AccountId, LockId)>, ValueQuery>;

    /// Stores the next locked ID should be.
    #[pallet::storage]
    pub type NextLockId<T: Config> = StorageValue<_, LockId, ValueQuery>;

    /// Stores the interest rate.
    #[pallet::storage]
    pub type InterestRate<T: Config> = StorageValue<_, Perbill, ValueQuery>;

    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T: Config> {
        pub balances: Vec<(T::AccountId, T::Balance)>,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            let total: T::Balance = self
                .balances
                .iter()
                .map(|(account_id, initial_balance)| {
                    assert!(
                    	*initial_balance >= T::ExistentialDeposit::get(),
                    	"the balance of any account should always be more than existential deposit.",
                    );
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
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn integrity_test() {
            // Check if the minimum deposit is greater than or equal to the existential deposit
            assert!(T::MinimumAmount::get() >= T::ExistentialDeposit::get());
            assert!(T::InterestPayoutPeriod::get() <= T::StakePeriod::get());
            assert!(T::InterestPayoutPeriod::get() <= T::RedeemPeriod::get());
            assert!(!T::TotalBlocksPerYear::get().is_zero());
        }

        fn on_finalize(block_number: BlockNumberFor<T>) {
            // Reap accounts below ED
            Self::reap_accounts();

            // Unlock funds that are due.
            AccountWithUnlockedFund::<T>::take(&block_number)
                .into_iter()
                .for_each(|(user, lock_id)| {
                    // Ignore the unlock result - locks can be unlocked early by other means.
                    let _ = Self::unlock(&user, lock_id, UnlockReason::Expired);
                });

            // Pay interest rate.

            // check if we should payout this block
            if (block_number % T::InterestPayoutPeriod::get()).is_zero() {
                // calculate the scaled interest rate
                // scaled_ir = ir_pa / blocks_pa * blocks_per_payout
                let interest_rate = InterestRate::<T>::get();
                let ir_per_payout = interest_rate
                    * Perbill::from_rational(
                        T::InterestPayoutPeriod::get(),
                        T::TotalBlocksPerYear::get(),
                    );
                // Pay out interest for all accounts, and tally the sum
                let total_interest: T::Balance = Accounts::<T>::iter_keys()
                    .map(|account_id| {
                        Accounts::<T>::mutate(account_id, |account_data| {
                            let interest = ir_per_payout * account_data.reserved;
                            account_data.reserved = account_data.reserved.saturating_add(interest);
                            interest
                        })
                    })
                    .sum();
                TotalIssuance::<T>::mutate(|total| {
                    *total = total.saturating_add(total_interest);
                });
                Self::deposit_event(Event::<T>::InterestPayed {
                    interest_rate,
                    total_interest_payed: total_interest,
                });
            }
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

        /// Stake `amount` of fund from the current user's free account to reserved account.
        #[pallet::call_index(3)]
        #[pallet::weight(0)]
        pub fn stake_funds(origin: OriginFor<T>, amount: T::Balance) -> DispatchResult {
            let user = ensure_signed(origin)?;
            T::RoleManager::ensure_role(&user, Role::Customer)?;
            <Self as Stakable<T::AccountId, T::Balance>>::stake_funds(&user, amount)
        }

        /// Redeem `amount` of fund from the current user's reserved account to locked account.
        #[pallet::call_index(4)]
        #[pallet::weight(0)]
        pub fn redeem_funds(origin: OriginFor<T>, amount: T::Balance) -> DispatchResult {
            let user = ensure_signed(origin)?;
            T::RoleManager::ensure_role(&user, Role::Customer)?;
            <Self as Stakable<T::AccountId, T::Balance>>::redeem_funds(&user, amount)
        }
        /// Auditor locked `amount` of fund from the any user's account to locked account for some period.
        /// Funds are taken from "free" first, then from "reserved".
        #[pallet::call_index(5)]
        #[pallet::weight(0)]
        pub fn lock_funds_auditor(
            origin: OriginFor<T>,
            user: T::AccountId,
            amount: T::Balance,
            length: BlockNumberFor<T>,
        ) -> DispatchResult {
            // Ensure the caller is the Auditor
            let id = ensure_signed(origin)?;
            T::RoleManager::ensure_role(&id, Role::Auditor)?;
            // Ensure the user is a customer to be locked
            T::RoleManager::ensure_role(&user, Role::Customer)?;
            // Implement logic to lock funds from free and reserved
            let unlock = T::BlockNumberProvider::current_block_number() + length;

            Accounts::<T>::mutate(&user, |account_data| {
                ensure!(
                    account_data.free + account_data.reserved >= amount,
                    Error::<T>::InsufficientBalance
                );

                let mut remain = amount;
                let free_deduction = min(account_data.free, remain);
                account_data.free -= free_deduction;
                remain -= free_deduction;

                account_data.reserved -= remain;

                let new_locked_fund = LockedFund {
                    id: Self::next_lock_id(),
                    amount,
                    reason: LockReason::Auditor,
                };
                account_data.locked.push(new_locked_fund);

                // Add new unlock user to the AccountWithUnlockedFunds
                AccountWithUnlockedFund::<T>::append(unlock, (user.clone(), new_locked_fund.id));

                Self::deposit_event(Event::<T>::Locked {
                    user: user.clone(),
                    amount,
                    length,
                    reason: LockReason::Auditor,
                });
                Ok(())
            })
        }

        /// Auditor unlocked the LockId which free the `amount` of fund from the user locked by auditor.
        /// Funds are returned from "locked" to "free".
        #[pallet::call_index(6)]
        #[pallet::weight(0)]
        pub fn unlock_funds_auditor(
            origin: OriginFor<T>,
            user: T::AccountId,
            lock_id: LockId,
        ) -> DispatchResult {
            // Ensure the caller is the Auditor
            let id = ensure_signed(origin)?;
            T::RoleManager::ensure_role(&id, Role::Auditor)?;
            // Ensure the user is a customer to be locked
            T::RoleManager::ensure_role(&user, Role::Customer)?;

            Self::unlock(&user, lock_id, UnlockReason::Auditor)
        }

        /// Manager set interest rate in basis point
        ///
        /// Requires Manager.
        #[pallet::call_index(7)]
        #[pallet::weight(0)]
        pub fn set_interest_rate(origin: OriginFor<T>, interest_rate_bps: u32) -> DispatchResult {
            let id = ensure_signed(origin)?;
            T::RoleManager::ensure_role(&id, Role::Manager)?;
            ensure!(
                interest_rate_bps <= 10000u32,
                Error::<T>::InvalidInterestRate
            );
            let old_interest_rate = InterestRate::<T>::get();
            InterestRate::<T>::set(Perbill::from_rational(interest_rate_bps, 10000u32));

            Self::deposit_event(Event::<T>::InterestRateSet {
                manager: id,
                old_interest_rate,
                new_interest_rate: Perbill::from_rational(interest_rate_bps, 10000u32),
            });
            Ok(())
        }
    }
}

impl<T: Config> BasicAccounting<T::AccountId, T::Balance> for Pallet<T> {
    fn deposit(user: &T::AccountId, amount: T::Balance) -> DispatchResult {
        if amount < T::MinimumAmount::get() {
            return Err(Error::<T>::AmountTooSmall.into());
        }
        Self::mint(&user, amount)?;
        Self::deposit_event(Event::<T>::Deposited {
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
        Self::deposit_event(Event::<T>::Withdrew {
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
        Self::deposit_event(Event::Transferred {
            from: from.clone(),
            to: to.clone(),
            amount,
        });
        Ok(())
    }
}

impl<T: Config> Stakable<T::AccountId, T::Balance> for Pallet<T> {
    /// Stake funds from free to reserved
    fn stake_funds(user: &T::AccountId, amount: T::Balance) -> DispatchResult {
        T::RoleManager::ensure_role(&user, Role::Customer)?;
        ensure!(
            amount >= T::MinimumAmount::get(),
            Error::<T>::AmountTooSmall
        );
        Accounts::<T>::mutate(&user, |account| -> DispatchResult {
            ensure!(account.free >= amount, Error::<T>::InsufficientBalance);
            account.free -= amount;
            let new_locked_fund = LockedFund {
                id: Self::next_lock_id(),
                amount,
                reason: LockReason::Stake,
            };
            account.locked.push(new_locked_fund);

            let unlock = T::BlockNumberProvider::current_block_number() + T::StakePeriod::get();
            AccountWithUnlockedFund::<T>::append(unlock, (user.clone(), new_locked_fund.id));

            Ok(())
        })?;
        Self::deposit_event(Event::<T>::Locked {
            user: user.clone(),
            amount,
            length: T::StakePeriod::get(),
            reason: LockReason::Stake,
        });

        Ok(())
    }

    /// Redeem funds from reserved to free after a certain time
    fn redeem_funds(user: &T::AccountId, amount: T::Balance) -> DispatchResult {
        T::RoleManager::ensure_role(&user, Role::Customer)?;
        ensure!(
            amount >= T::MinimumAmount::get(),
            Error::<T>::AmountTooSmall
        );

        // get unlock BlockNumber
        let unlock = T::BlockNumberProvider::current_block_number() + T::RedeemPeriod::get();

        // Add new locked funds to user's Account Data
        Accounts::<T>::mutate(&user, |account| -> DispatchResult {
            ensure!(account.reserved >= amount, Error::<T>::InsufficientBalance);
            account.reserved -= amount;
            let new_locked_fund = LockedFund {
                id: Self::next_lock_id(),
                amount,
                reason: LockReason::Redeem,
            };
            account.locked.push(new_locked_fund);

            // Add new unlock user to the AccountWithUnlockedFunds
            AccountWithUnlockedFund::<T>::append(unlock, (user.clone(), new_locked_fund.id));
            Ok(())
        })?;

        Self::deposit_event(Event::<T>::Locked {
            user: user.clone(),
            amount,
            length: T::RedeemPeriod::get(),
            reason: LockReason::Redeem,
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
                .map(|(_, account)| account.total())
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
                treasury_account.free = treasury_account.free.saturating_add(total_reaped_amount);
            });
        }
    }

    /// Get the lock id to store into the LockedFund.
    fn next_lock_id() -> LockId {
        NextLockId::<T>::mutate(|id| {
            *id = id.saturating_add(1);
            *id
        })
    }

    ///Transfer locked funds to free funds
    fn unlock(
        account_id: &T::AccountId,
        locked_id: LockId,
        reason: UnlockReason,
    ) -> DispatchResult {
        Accounts::<T>::try_mutate(account_id, |account_data| {
            if let Some(index) = account_data
                .locked
                .iter()
                .position(|item| item.id == locked_id)
            {
                ensure!(
                    reason != UnlockReason::Auditor
                        || account_data.locked[index].reason == LockReason::Auditor,
                    Error::<T>::UnauthorisedUnlock
                );
                let unlocked_amount = account_data.locked[index].amount;

                if account_data.locked[index].reason == LockReason::Stake {
                    account_data.reserved = account_data.reserved.saturating_add(unlocked_amount);
                } else {
                    account_data.free = account_data.free.saturating_add(unlocked_amount);
                }

                // Remove the unlocked locked fund from the vector
                account_data.locked.remove(index);
                Self::deposit_event(Event::Unlocked {
                    user: account_id.clone(),
                    amount: unlocked_amount,
                    reason,
                });
                Ok(())
            } else {
                Err(Error::<T>::InvalidLockId.into())
            }
        })
    }
}
