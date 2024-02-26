//! # Roles Pallet

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{pallet_prelude::*, traits::BuildGenesisConfig};
use frame_system::pallet_prelude::*;
use sp_runtime::DispatchResult;
use sp_std::{prelude::*, vec::Vec};

use primitives::Role;
use traits::ManageRoles;

mod mock;
mod tests;

pub mod weights;
pub use weights::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub use module::*;

#[frame_support::pallet]
pub mod module {

	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Type representing the weight of this pallet
		type WeightInfo: WeightInfo;

		type EnsureGovernance: EnsureOrigin<<Self as frame_system::Config>::RuntimeOrigin>;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The account role cannot be changed, you must unregister first.
		AccountAlreadyRegistered,
		/// The account hasn't registered a role.
		AccountRoleNotRegistered,
		/// The account role does not equal to the expected role.
		IncorrectRole,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A user has registered a role
		RoleRegistered { user: T::AccountId, role: Role },

		/// A user's role is unregistered.
		RoleUnregistered { user: T::AccountId },
	}

	#[pallet::storage]
	#[pallet::getter(fn account_roles)]
	pub type AccountRoles<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Role>;

	#[pallet::genesis_config]
	#[derive(frame_support::DefaultNoBound)]
	pub struct GenesisConfig<T: Config> {
		pub roles: Vec<(T::AccountId, Role)>,
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			self.roles.iter().for_each(|(id, role)| {
				AccountRoles::<T>::insert(id, role);
			});
		}
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Register a customer role for a user.
		///
		/// This function allows a user to be registered with a customer role only.
		/// The user must be signed and authenticated.
		///
		/// Params:
		/// - `role`: The role to assign to the user.
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::register_customer())]
		pub fn register_customer(origin: OriginFor<T>) -> DispatchResult {
			let id = ensure_signed(origin)?;
			Self::register_role(&id, Role::Customer)
		}

		/// Unregister a role for a user.
		///
		/// This function allows a user's role to be unregistered.
		/// The user must be signed and authenticated.
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::unregister())]
		pub fn unregister(origin: OriginFor<T>) -> DispatchResult {
			let id = ensure_signed(origin)?;
			Self::unregister_role(&id)
		}

		/// Register any role for a user through Governance.
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::register_role_governance())]
		pub fn register_role_governance(
			origin: OriginFor<T>,
			id: T::AccountId,
			role: Role,
		) -> DispatchResult {
			// ensure governance
			T::EnsureGovernance::ensure_origin(origin)?;

			Self::register_role(&id, role)
		}
	}
}

impl<T: Config> ManageRoles<T::AccountId> for Pallet<T> {
	/// Get the role of a given user.
	fn role(id: &T::AccountId) -> Option<Role> {
		AccountRoles::<T>::get(id)
	}

	/// Register a role for a user, insert the user's role into storage and emit a role_registered
	/// event.
	fn register_role(id: &T::AccountId, role: Role) -> DispatchResult {
		ensure!(AccountRoles::<T>::get(id).is_none(), Error::<T>::AccountAlreadyRegistered);
		AccountRoles::<T>::insert(id, role);
		Self::deposit_event(Event::<T>::RoleRegistered { user: id.clone(), role });
		Ok(())
	}

	/// Unregister a role for a user, remove the user's role from storage and emit a role
	/// unregistered event.
	fn unregister_role(id: &T::AccountId) -> DispatchResult {
		ensure!(AccountRoles::<T>::get(id).is_some(), Error::<T>::AccountRoleNotRegistered);
		AccountRoles::<T>::remove(id);
		Self::deposit_event(Event::<T>::RoleUnregistered { user: id.clone() });
		Ok(())
	}

	/// Ensure that a user has a specific role.
	fn ensure_role(id: &T::AccountId, role: Role) -> DispatchResult {
		match AccountRoles::<T>::get(id) {
			Some(r) =>
				if r != role {
					Err(Error::<T>::IncorrectRole.into())
				} else {
					Ok(())
				},
			None => Err(Error::<T>::AccountRoleNotRegistered.into()),
		}
	}
}
