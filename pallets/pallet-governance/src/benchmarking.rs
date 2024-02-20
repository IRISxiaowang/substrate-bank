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
	fn initiate_proposal() {
		let caller: T::AccountId = whitelisted_caller();
		let new_council: T::AccountId = account("council", 0u32, 0u32);

		let call = Box::new(
			crate::Call::<T>::council_rotate_authorities { new_members: vec![new_council] }.into(),
		);

		let first_proposal = 1u32;
		CurrentAuthorities::<T>::set(BTreeSet::from([caller.clone()]));

		#[extrinsic_call]
		initiate_proposal(RawOrigin::Signed(caller), call);

		// Verify
		assert!(Proposals::<T>::contains_key(first_proposal));
	}

	#[benchmark]
	fn vote() {
		let caller: T::AccountId = whitelisted_caller();
		let caller_2: T::AccountId = account("caller_2", 0u32, 0u32);

		let call = Box::new(frame_system::Call::<T>::remark { remark: vec![] }.into());
		let first_proposal = 1u32;

		CurrentAuthorities::<T>::set(BTreeSet::from([caller.clone(), caller_2.clone()]));
		assert_ok!(Pallet::<T>::initiate_proposal(RawOrigin::Signed(caller.clone()).into(), call));

		#[extrinsic_call]
		vote(RawOrigin::Signed(caller_2.clone()), first_proposal, true);

		// Verify
		assert_eq!(
			Votes::<T>::get(first_proposal),
			CastedVotes { yays: BTreeSet::from([caller, caller_2]), nays: Default::default() }
		);
	}

	#[benchmark]
	fn council_rotate_authorities() {
		let new_council: T::AccountId = account("council", 0u32, 0u32);
		let call = Call::<T>::council_rotate_authorities { new_members: vec![new_council.clone()] };
		let origin = T::EnsureGovernance::try_successful_origin().unwrap();

		#[block]
		{
			assert_ok!(call.dispatch_bypass_filter(origin));
		}

		// Verify
		assert_eq!(CurrentAuthorities::<T>::get(), BTreeSet::from([new_council]));
	}

	#[benchmark]
	fn force_rotate_authorities() {
		let new_council: T::AccountId = account("council", 0u32, 0u32);

		#[extrinsic_call]
		force_rotate_authorities(RawOrigin::Root, vec![new_council.clone()]);

		// Verify
		assert_eq!(CurrentAuthorities::<T>::get(), BTreeSet::from([new_council]));
	}

	impl_benchmark_test_suite!(Pallet, crate::mock::default_test_ext(), crate::mock::Runtime);
}
