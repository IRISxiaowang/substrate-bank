//! Benchmarking setup for pallet-template
#![cfg(feature = "runtime-benchmarks")]
use super::*;

use frame_benchmarking::v2::*;
use frame_support::assert_ok;
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {

	use super::*;
	struct MockUsers<AccountId> {
		pub manager: AccountId,
		pub auditor: AccountId,
		pub customer_1: AccountId,
		pub customer_2: AccountId,
	}

	fn setup<T: Config>() -> MockUsers<T::AccountId> {
		let manager: T::AccountId = whitelisted_caller();
		let auditor: T::AccountId = account("auditor", 0u32, 0u32);
		let customer_1: T::AccountId = account("customer_1", 0u32, 0u32);
		let customer_2: T::AccountId = account("customer_2", 0u32, 0u32);
		assert_ok!(T::RoleManager::register_role(&manager, Role::Manager));
		assert_ok!(T::RoleManager::register_role(&auditor, Role::Auditor));
		assert_ok!(T::RoleManager::register_role(&customer_1, Role::Customer));
		assert_ok!(T::RoleManager::register_role(&customer_2, Role::Customer));
		Accounts::<T>::insert(
			&customer_1,
			AccountData {
				free: 1_000_000u128.into(),
				reserved: 1_000_000u128.into(),
				locked: vec![],
			},
		);
		Accounts::<T>::insert(
			&customer_2,
			AccountData {
				free: 1_000_000u128.into(),
				reserved: 1_000_000u128.into(),
				locked: vec![LockedFund {
					id: 1u64,
					amount: 1_000u128.into(),
					reason: LockReason::Auditor,
				}],
			},
		);

		MockUsers { manager, auditor, customer_1, customer_2 }
	}

	#[benchmark]
	fn deposit() {
		let accounts = setup::<T>();
		let initial_balance = Accounts::<T>::get(&accounts.customer_1).free;
		let amount = 1_000u128.into();

		#[extrinsic_call]
		deposit(RawOrigin::Signed(accounts.manager), accounts.customer_1.clone(), amount);

		// Verify
		assert_eq!(Accounts::<T>::get(accounts.customer_1).free, initial_balance + amount);
	}

	#[benchmark]
	fn withdraw() {
		let accounts = setup::<T>();
		let amount = 1_000u128.into();
		let initial_balance = Accounts::<T>::get(&accounts.customer_1).free;
		#[extrinsic_call]
		withdraw(RawOrigin::Signed(accounts.manager), accounts.customer_1.clone(), amount);

		// Verify
		assert_eq!(Accounts::<T>::get(&accounts.customer_1).free, initial_balance - amount);
	}

	#[benchmark]
	fn transfer() {
		let accounts = setup::<T>();
		let amount = 1_000u128.into();
		let initial_balance_1 = Accounts::<T>::get(&accounts.customer_1).free;
		let initial_balance_2 = Accounts::<T>::get(&accounts.customer_2).free;
		#[extrinsic_call]
		transfer(
			RawOrigin::Signed(accounts.customer_1.clone()),
			accounts.customer_2.clone(),
			amount,
		);

		// Verify
		assert_eq!(Accounts::<T>::get(&accounts.customer_1).free, initial_balance_1 - amount);
		assert_eq!(Accounts::<T>::get(&accounts.customer_2).free, initial_balance_2 + amount);
	}

	#[benchmark]
	fn stake_funds() {
		let accounts = setup::<T>();
		let amount = 1_000u128.into();
		let initial_balance = Accounts::<T>::get(&accounts.customer_1).free;

		#[extrinsic_call]
		stake_funds(RawOrigin::Signed(accounts.customer_1.clone()), amount);

		// Verify
		assert_eq!(Accounts::<T>::get(&accounts.customer_1).free, initial_balance - amount);
		assert_eq!(Accounts::<T>::get(&accounts.customer_1).locked[0].amount, amount);
	}

	#[benchmark]
	fn redeem_funds() {
		let accounts = setup::<T>();
		let amount = 1_000u128.into();
		let reserved_balance = Accounts::<T>::get(&accounts.customer_1).reserved;

		#[extrinsic_call]
		redeem_funds(RawOrigin::Signed(accounts.customer_1.clone()), amount);

		// Verify
		assert_eq!(Accounts::<T>::get(&accounts.customer_1).reserved, reserved_balance - amount);
		assert_eq!(Accounts::<T>::get(&accounts.customer_1).locked[0].amount, amount);
	}

	#[benchmark]
	fn lock_funds_auditor() {
		let accounts = setup::<T>();
		let amount = 1_000u128.into();
		let free_balance = Accounts::<T>::get(&accounts.customer_1).free;
		let reserved_balance = Accounts::<T>::get(&accounts.customer_1).reserved;

		#[extrinsic_call]
		lock_funds_auditor(
			RawOrigin::Signed(accounts.auditor.clone()),
			accounts.customer_1.clone(),
			amount,
			50u32.into(),
		);

		// Verify
		assert_eq!(
			Accounts::<T>::get(&accounts.customer_1),
			AccountData {
				free: free_balance - amount,
				reserved: reserved_balance,
				locked: vec![LockedFund { id: 1u64, amount, reason: LockReason::Auditor },]
			}
		);
	}

	#[benchmark]
	fn unlock_funds_auditor() {
		let accounts = setup::<T>();
		let amount = 1_000u128.into();
		let free_balance = Accounts::<T>::get(&accounts.customer_2).free;

		#[extrinsic_call]
		unlock_funds_auditor(
			RawOrigin::Signed(accounts.auditor.clone()),
			accounts.customer_2.clone(),
			1u64,
		);

		// Verify
		assert_eq!(Accounts::<T>::get(&accounts.customer_2).free, free_balance + amount);
		assert!(Accounts::<T>::get(&accounts.customer_2).locked.is_empty());
	}

	#[benchmark]
	fn set_interest_rate() {
		let accounts = setup::<T>();
		let interest_rate = Perbill::from_percent(5);

		#[extrinsic_call]
		set_interest_rate(RawOrigin::Signed(accounts.manager.clone()), interest_rate * 10_000u32);

		// Verify
		assert_eq!(InterestRate::<T>::get(), interest_rate);
	}

	#[benchmark]
	fn rotate_treasury() {
		let treasury = account("treasury", 0u32, 0u32);
		let new_treasury: T::AccountId = account("new_treasury", 0u32, 0u32);
		TreasuryAccount::<T>::set(Some(treasury));
		#[extrinsic_call]
		rotate_treasury(RawOrigin::Root, new_treasury.clone());

		// Verify
		assert_eq!(TreasuryAccount::<T>::get(), Some(new_treasury));
	}

	impl_benchmark_test_suite!(Pallet, crate::mock::default_test_ext(), crate::mock::Runtime);
}
