#![cfg_attr(not(feature = "std"), no_std)]
//!
//! This crate contains basic primitive types used within the blockchain codebase.
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