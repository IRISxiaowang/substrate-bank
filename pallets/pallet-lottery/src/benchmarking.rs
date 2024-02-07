//! Benchmarking setup for pallet-template
#![cfg(feature = "runtime-benchmarks")]
use super::*;

use frame_benchmarking::v2::*;
use frame_support::assert_ok;
use frame_system::RawOrigin;
use primitives::DOLLAR;
use sp_runtime::Percent;

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn set_prize_split() {
		let split =
			vec![Percent::from_percent(50), Percent::from_percent(30), Percent::from_percent(20)];

		#[extrinsic_call]
		set_prize_split(RawOrigin::Root, split.clone());

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

	impl_benchmark_test_suite!(Pallet, crate::mock::default_test_ext(), crate::mock::Runtime);
}
