//! Benchmarking setup for pallet-template
#![cfg(feature = "runtime-benchmarks")]
use super::*;

use frame_benchmarking::v2::*;
use frame_support::{assert_ok, traits::UnfilteredDispatchable};
use frame_system::RawOrigin;
use primitives::DOLLAR;
use sp_runtime::{traits::BlockNumberProvider, Percent};

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn set_prize_split() {
		let split =
			vec![Percent::from_percent(50), Percent::from_percent(30), Percent::from_percent(20)];
		let call = Call::<T>::set_prize_split { prize_split: split.clone() };
		let origin = T::EnsureGovernance::try_successful_origin().unwrap();

		#[block]
		{
			assert_ok!(call.dispatch_bypass_filter(origin));
		}

		// Verify
		assert_eq!(PrizeSplit::<T>::get(), split);
	}

	#[benchmark]
	fn update_ticket_price() {
		let caller: T::AccountId = whitelisted_caller();
		assert_ok!(T::RoleManager::register_role(&caller, Role::Manager));

		#[extrinsic_call]
		update_ticket_price(RawOrigin::Signed(caller.clone()), 1u128.into());

		// Verify
		assert_eq!(TicketPrice::<T>::get(), 1u128.into());
	}

	#[benchmark]
	fn buy_ticket() {
		let caller: T::AccountId = whitelisted_caller();
		let amount = (DOLLAR * 1_000).into();
		assert_ok!(T::RoleManager::register_role(&caller, Role::Customer));

		assert_ok!(T::Bank::deposit(&caller.clone(), amount));
		TicketPrice::<T>::set((DOLLAR * 2).into());

		#[extrinsic_call]
		buy_ticket(RawOrigin::Signed(caller.clone()), 5);

		// Verify
		assert_eq!(T::Bank::free_balance(&caller), (DOLLAR * 990).into());
		assert_eq!(T::Bank::free_balance(&T::PrizePoolAccount::get()), (DOLLAR * 10).into());
	}

	#[benchmark]
	fn force_draw() {
		let call = Call::<T>::force_draw {};
		let origin = T::EnsureGovernance::try_successful_origin().unwrap();

		#[block]
		{
			assert_ok!(call.dispatch_bypass_filter(origin));
		}

		// Verify
		assert_eq!(StartBlock::<T>::get(), frame_system::Pallet::<T>::current_block_number());
	}
	impl_benchmark_test_suite!(Pallet, crate::mock::default_test_ext(), crate::mock::Runtime);
}
