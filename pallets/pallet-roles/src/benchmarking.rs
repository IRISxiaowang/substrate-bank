//! Benchmarking setup for pallet-template
#![cfg(feature = "runtime-benchmarks")]
use super::*;

use frame_benchmarking::v2::*;
use frame_support::{assert_ok, traits::UnfilteredDispatchable};
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn register() {
		let caller: T::AccountId = whitelisted_caller();

		#[extrinsic_call]
		register(RawOrigin::Signed(caller.clone()));

		// Verify
		assert_eq!(Pallet::<T>::role(&caller), Some(Role::Customer));
	}
	#[benchmark]
	fn unregister() {
		let caller: T::AccountId = whitelisted_caller();
		assert_ok!(Pallet::<T>::register(RawOrigin::Signed(caller.clone()).into()));

		#[extrinsic_call]
		unregister(RawOrigin::Signed(caller.clone()));

		// Verify
		assert_eq!(Pallet::<T>::role(&caller), None);
	}

	#[benchmark]
	fn register_role_governance() {
		let manager: T::AccountId = account("manager", 0u32, 0u32);
		let call = Call::<T>::register_role_governance { id: manager.clone(), role: Role::Manager };
		let origin = T::EnsureGovernance::try_successful_origin().unwrap();

		#[block]
		{
			assert_ok!(call.dispatch_bypass_filter(origin));
		}

		// Verify
		assert_eq!(Pallet::<T>::role(&manager), Some(Role::Manager));
	}

	impl_benchmark_test_suite!(Pallet, crate::mock::default_test_ext(), crate::mock::Runtime);
}
