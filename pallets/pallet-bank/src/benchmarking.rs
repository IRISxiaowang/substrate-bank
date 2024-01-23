//! Benchmarking setup for pallet-template
#![cfg(feature = "runtime-benchmarks")]
use super::*;

use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use frame_support::assert_ok;

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn deposit() {
		let manager: T::AccountId = whitelisted_caller();
		let customer: T::AccountId = account("customer", 0u32, 0u32);
		let initial_balance = Accounts::<T>::get(&customer).free;

		let amount = 1_000u128.into();
		
		assert_ok!(T::RoleManager::register_role(&manager, Role::Manager));
		assert_ok!(T::RoleManager::register_role(&customer, Role::Customer));

		#[extrinsic_call]
		deposit(
            RawOrigin::Signed(manager),
            customer.clone(),
            amount
		);

		// Verify 
		assert_eq!(Accounts::<T>::get(customer).free, initial_balance + amount);
	}

	impl_benchmark_test_suite!(Pallet, crate::mock::default_test_ext(), crate::mock::Runtime);
}
