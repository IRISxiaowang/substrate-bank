#![cfg_attr(not(feature = "std"), no_std)]
//!
//! This crate contains traits shared across the codebase.
//!

use sp_runtime::DispatchResult;

use primitives::Role;

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

/// A trait for basic accounting operations like deposit, withdrawal, and transfer.
pub trait BasicAccounting<AccountId, Balance> {
    fn deposit(user: &AccountId, amount: Balance) -> DispatchResult;
    fn withdraw(user: &AccountId, amount: Balance) -> DispatchResult;
    fn transfer(from: &AccountId, to: &AccountId, amount: Balance) -> DispatchResult;
}

/// A trait for stake and redeem funds.
pub trait Stakable<AccountId, Balance> {
    fn stake_funds(user: &AccountId, amount: Balance) -> DispatchResult;
    fn redeem_funds(user: &AccountId, amount: Balance) -> DispatchResult;
}
