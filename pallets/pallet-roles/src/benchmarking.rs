//! Benchmarking setup for pallet-template
#![cfg(feature = "runtime-benchmarks")]
use super::*;

use frame_benchmarking::v2::*;
use frame_support::assert_ok;
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn register() {
		let caller: T::AccountId = whitelisted_caller();

		#[extrinsic_call]
		register(RawOrigin::Signed(caller.clone()), Role::Customer);

		// Verify
		assert_eq!(Pallet::<T>::role(&caller), Some(Role::Customer));
	}
	#[benchmark]
	fn unregister() {
		let caller: T::AccountId = whitelisted_caller();
		assert_ok!(
			Pallet::<T>::register(RawOrigin::Signed(caller.clone()).into(), Role::Customer,)
		);

		#[extrinsic_call]
		unregister(RawOrigin::Signed(caller.clone()));

		// Verify
		assert_eq!(Pallet::<T>::role(&caller), None);
	}

	impl_benchmark_test_suite!(Pallet, crate::mock::default_test_ext(), crate::mock::Runtime);
}
