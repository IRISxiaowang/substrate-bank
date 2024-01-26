//! Unit tests for the tokens module.

#![cfg(test)]

use crate::{
	mock::{
		default_test_ext, AccountId, Balance, Bank, MockGenesisConfig, Roles, Runtime,
		RuntimeEvent, RuntimeOrigin, StakePeriod, System, ALICE, BOB, INTEREST_PAYOUT_PERIOD,
		REDEEM_PERIOD, STAKE_PERIOD, TREASURY,
	},
	*,
};
use frame_support::{assert_err, assert_noop, assert_ok};
use frame_system::RawOrigin;

fn stake(user: AccountId, amount: Balance) {
	let reserved = Bank::accounts(user).reserved;
	let _ = Bank::stake_funds(RuntimeOrigin::signed(user), amount);
	Bank::on_finalize(System::block_number() + StakePeriod::get());
	assert_eq!(Bank::accounts(user).reserved, reserved.saturating_add(amount));
	System::assert_has_event(RuntimeEvent::Bank(Event::<Runtime>::Locked {
		user,
		amount,
		length: STAKE_PERIOD,
		reason: LockReason::Stake,
	}));
	System::assert_last_event(RuntimeEvent::Bank(Event::<Runtime>::Unlocked {
		user,
		amount,
		reason: UnlockReason::Expired,
	}));
}

#[test]
fn can_deposit() {
	default_test_ext().execute_with(|| {
		assert_eq!(Bank::accounts(ALICE), AccountData::default());
		assert_eq!(Bank::accounts(BOB), AccountData::default());
		assert_ok!(Roles::register_role(&ALICE, Role::Manager));
		assert_ok!(Roles::register_role(&BOB, Role::Customer));
		System::reset_events();
		let sender = RuntimeOrigin::signed(ALICE);
		assert_ok!(Bank::deposit(sender, BOB, 1_000));
		// Check that the event was emitted
		System::assert_last_event(RuntimeEvent::Bank(Event::<Runtime>::Deposited {
			user: BOB,
			amount: 1_000,
		}));
		assert_eq!(Accounts::<Runtime>::get(BOB).free, 1_000);
		assert_noop!(
			Bank::deposit(RuntimeOrigin::signed(ALICE), ALICE, 500),
			pallet_roles::Error::<Runtime>::IncorrectRole
		);
		assert!(Bank::check_total_issuance());
	});

	MockGenesisConfig::with_balances(vec![(ALICE, 1_000_000), (BOB, 50)])
		.build()
		.execute_with(|| {
			assert_eq!(Accounts::<Runtime>::get(ALICE).free, 1_000_000);
			assert_eq!(Accounts::<Runtime>::get(BOB).free, 50);
		});
}

#[test]
fn can_withdraw() {
	MockGenesisConfig::with_balances(vec![(BOB, 500)]).build().execute_with(|| {
		assert_ok!(Roles::register_role(&ALICE, Role::Manager));
		assert_eq!(Accounts::<Runtime>::get(BOB).free, 500);
		System::reset_events();
		assert_ok!(Bank::withdraw(RuntimeOrigin::signed(ALICE), BOB, 100));
		System::assert_last_event(RuntimeEvent::Bank(Event::<Runtime>::Withdrew {
			user: BOB,
			amount: 100,
		}));
		assert_eq!(Accounts::<Runtime>::get(BOB).free, 400);
		assert_noop!(
			Bank::withdraw(RuntimeOrigin::signed(ALICE), BOB, 500),
			Error::<Runtime>::InsufficientBalance
		);
		assert_noop!(
			Bank::withdraw(RuntimeOrigin::signed(ALICE), ALICE, 500),
			pallet_roles::Error::<Runtime>::IncorrectRole
		);
		assert!(Bank::check_total_issuance());
	});
}

#[test]
fn can_transfer() {
	MockGenesisConfig::with_balances(vec![(ALICE, 1_000), (BOB, 500)])
		.build()
		.execute_with(|| {
			System::reset_events();
			assert_ok!(Bank::transfer(RuntimeOrigin::signed(ALICE), BOB, 100));
			// Check that the event was emitted
			System::assert_last_event(RuntimeEvent::Bank(Event::<Runtime>::Transferred {
				from: ALICE,
				to: BOB,
				amount: 100,
			}));

			assert_eq!(Accounts::<Runtime>::get(BOB).free, 600);
			assert_eq!(Accounts::<Runtime>::get(ALICE).free, 900);
			assert_noop!(
				Bank::transfer(RuntimeOrigin::signed(ALICE), BOB, 1_000),
				Error::<Runtime>::InsufficientBalance
			);
			assert_eq!(Bank::accounts(3), AccountData::default());
			assert_ok!(Roles::register_role(&3, Role::Manager));
			assert_noop!(
				Bank::transfer(RuntimeOrigin::signed(3), BOB, 100),
				pallet_roles::Error::<Runtime>::IncorrectRole
			);
			assert!(Bank::check_total_issuance());
		});
}

#[test]
fn cannot_dealwith_smaller_than_min() {
	default_test_ext().execute_with(|| {
		let charlie: AccountId = 3u32;
		assert_eq!(Bank::accounts(ALICE), AccountData::default());
		assert_eq!(Bank::accounts(BOB), AccountData::default());
		assert_eq!(Bank::accounts(charlie), AccountData::default());
		assert_ok!(Roles::register_role(&ALICE, Role::Customer));
		assert_ok!(Roles::register_role(&BOB, Role::Customer));
		assert_ok!(Roles::register_role(&charlie, Role::Manager));

		assert_noop!(
			Bank::deposit(RuntimeOrigin::signed(charlie), BOB, 4),
			Error::<Runtime>::AmountTooSmall
		);
		assert_noop!(
			Bank::withdraw(RuntimeOrigin::signed(charlie), BOB, 4),
			Error::<Runtime>::AmountTooSmall
		);
		assert_noop!(
			Bank::transfer(RuntimeOrigin::signed(ALICE), BOB, 4),
			Error::<Runtime>::AmountTooSmall
		);

		assert_noop!(
			Bank::stake_funds(RuntimeOrigin::signed(ALICE), 4),
			Error::<Runtime>::AmountTooSmall
		);
		assert_noop!(
			Bank::redeem_funds(RuntimeOrigin::signed(ALICE), 4),
			Error::<Runtime>::AmountTooSmall
		);
	});
}

#[test]
fn can_reap_accounts() {
	default_test_ext().execute_with(|| {
		TreasuryAccount::<Runtime>::set(Some(TREASURY));
		let charlie: AccountId = 3u32;
		assert_eq!(Bank::accounts(ALICE), AccountData::default());
		assert_eq!(Bank::accounts(BOB), AccountData::default());
		assert_eq!(Bank::accounts(charlie), AccountData::default());
		assert_ok!(Roles::register_role(&ALICE, Role::Customer));
		assert_ok!(Roles::register_role(&BOB, Role::Customer));
		assert_ok!(Roles::register_role(&charlie, Role::Manager));

		assert_ok!(Bank::deposit(RuntimeOrigin::signed(charlie), BOB, 100));
		assert_eq!(Accounts::<Runtime>::get(BOB).free, 100);
		assert_ok!(Bank::transfer(RuntimeOrigin::signed(BOB), ALICE, 98));
		assert_eq!(Accounts::<Runtime>::get(BOB).free, 2);
		assert_eq!(Accounts::<Runtime>::get(ALICE).free, 98);

		System::reset_events();
		Bank::on_finalize(1);
		System::assert_last_event(RuntimeEvent::Bank(Event::<Runtime>::Reaped {
			user: BOB,
			dust: 2,
		}));
		assert_eq!(Accounts::<Runtime>::get(BOB), Default::default());
		let treasury = Bank::treasury().expect("Treasury account must be set.");

		assert_eq!(Accounts::<Runtime>::get(treasury).free, 1_000_002);
		assert!(Bank::check_total_issuance());
	});
}

#[test]
fn can_stake_funds() {
	MockGenesisConfig::with_balances(vec![(ALICE, 1_000)]).build().execute_with(|| {
		System::reset_events();
		assert_ok!(Bank::stake_funds(RuntimeOrigin::signed(ALICE), 200));
		System::assert_last_event(RuntimeEvent::Bank(Event::<Runtime>::Locked {
			user: ALICE,
			amount: 200,
			length: StakePeriod::get(),
			reason: LockReason::Stake,
		}));

		assert!(Bank::check_total_issuance());

		assert_noop!(
			Bank::stake_funds(RuntimeOrigin::signed(ALICE), 801),
			Error::<Runtime>::InsufficientBalance
		);
	});
}

#[test]
fn can_redeem_funds() {
	MockGenesisConfig::with_balances(vec![(ALICE, 1_000)]).build().execute_with(|| {
		stake(ALICE, 1_000);
		System::reset_events();
		assert_ok!(Bank::redeem_funds(RuntimeOrigin::signed(ALICE), 200));
		System::assert_last_event(RuntimeEvent::Bank(Event::<Runtime>::Locked {
			user: ALICE,
			amount: 200,
			length: REDEEM_PERIOD,
			reason: LockReason::Redeem,
		}));
		assert_eq!(
			Accounts::<Runtime>::get(ALICE),
			AccountData {
				free: 0,
				reserved: 800,
				locked: vec![LockedFund { id: 2, amount: 200, reason: LockReason::Redeem }]
			}
		);

		assert!(Bank::check_total_issuance());

		assert_noop!(
			Bank::redeem_funds(RuntimeOrigin::signed(ALICE), 801),
			Error::<Runtime>::InsufficientBalance
		);
	});
}

#[test]
fn auditor_can_lock_funds() {
	MockGenesisConfig::with_balances(vec![(ALICE, 1_000)]).build().execute_with(|| {
		let charlie: AccountId = 3u32;
		assert_eq!(Bank::accounts(charlie), AccountData::default());
		assert_ok!(Roles::register_role(&charlie, Role::Auditor));
		stake(ALICE, 900);
		System::reset_events();
		assert_ok!(Bank::lock_funds_auditor(RuntimeOrigin::signed(charlie), ALICE, 200, 20));
		System::assert_last_event(RuntimeEvent::Bank(Event::<Runtime>::Locked {
			user: ALICE,
			amount: 200,
			length: 20,
			reason: LockReason::Auditor,
		}));
		assert_eq!(
			Accounts::<Runtime>::get(ALICE),
			AccountData {
				free: 0,
				reserved: 800,
				locked: vec![LockedFund { id: 2, amount: 200, reason: LockReason::Auditor }]
			}
		);

		assert!(Bank::check_total_issuance());

		assert_noop!(
			Bank::lock_funds_auditor(RuntimeOrigin::signed(charlie), ALICE, 801, 20),
			Error::<Runtime>::InsufficientBalance
		);
	});
}

#[test]
fn auditor_can_unlock_funds() {
	MockGenesisConfig::with_balances(vec![(ALICE, 1_000)]).build().execute_with(|| {
		let charlie: AccountId = 3u32;
		assert_eq!(Bank::accounts(charlie), AccountData::default());
		assert_ok!(Roles::register_role(&charlie, Role::Auditor));
		stake(ALICE, 900);
		assert_ok!(Bank::lock_funds_auditor(RuntimeOrigin::signed(charlie), ALICE, 200, 20));

		assert_ok!(Bank::redeem_funds(RuntimeOrigin::signed(ALICE), 500));

		assert_eq!(
			Accounts::<Runtime>::get(ALICE),
			AccountData {
				free: 0,
				reserved: 300,
				locked: vec![
					LockedFund { id: 2, amount: 200, reason: LockReason::Auditor },
					LockedFund { id: 3, amount: 500, reason: LockReason::Redeem }
				]
			}
		);

		assert_noop!(
			Bank::unlock_funds_auditor(RuntimeOrigin::signed(charlie), ALICE, 3),
			Error::<Runtime>::UnauthorisedUnlock
		);

		System::reset_events();
		assert_ok!(Bank::unlock_funds_auditor(RuntimeOrigin::signed(charlie), ALICE, 2));
		System::assert_last_event(RuntimeEvent::Bank(Event::<Runtime>::Unlocked {
			user: ALICE,
			amount: 200,
			reason: UnlockReason::Auditor,
		}));

		assert_eq!(
			Accounts::<Runtime>::get(ALICE),
			AccountData {
				free: 200,
				reserved: 300,
				locked: vec![LockedFund { id: 3, amount: 500, reason: LockReason::Redeem }]
			}
		);
		assert!(Bank::check_total_issuance());
	});
}

#[test]
fn incorrect_role_cannot_call_auditor_function() {
	MockGenesisConfig::with_balances(vec![(ALICE, 1_000)]).build().execute_with(|| {
		let charlie: AccountId = 3u32;
		assert_eq!(Bank::accounts(charlie), AccountData::default());
		assert_ok!(Roles::register_role(&charlie, Role::Auditor));
		assert_eq!(Bank::accounts(BOB), AccountData::default());
		assert_ok!(Roles::register_role(&BOB, Role::Manager));
		stake(ALICE, 900);
		// Auditor can lock and unlock
		assert_ok!(Bank::lock_funds_auditor(RuntimeOrigin::signed(charlie), ALICE, 200, 20));

		// Customer cannot lock/unlock
		assert_noop!(
			Bank::lock_funds_auditor(RuntimeOrigin::signed(ALICE), ALICE, 100, 20),
			pallet_roles::Error::<Runtime>::IncorrectRole
		);
		assert_noop!(
			Bank::unlock_funds_auditor(RuntimeOrigin::signed(ALICE), ALICE, 1),
			pallet_roles::Error::<Runtime>::IncorrectRole
		);

		// Manager cannot lock/unlock
		assert_noop!(
			Bank::unlock_funds_auditor(RuntimeOrigin::signed(BOB), ALICE, 1),
			pallet_roles::Error::<Runtime>::IncorrectRole
		);
		assert_noop!(
			Bank::lock_funds_auditor(RuntimeOrigin::signed(BOB), ALICE, 100, 20),
			pallet_roles::Error::<Runtime>::IncorrectRole
		);

		assert_eq!(
			Accounts::<Runtime>::get(ALICE),
			AccountData {
				free: 0,
				reserved: 800,
				locked: vec![LockedFund { id: 2, amount: 200, reason: LockReason::Auditor },]
			}
		);
		assert!(Bank::check_total_issuance());
	});
}

#[test]
fn auditor_cannot_unlock_invalid_id() {
	MockGenesisConfig::with_balances(vec![(ALICE, 1_000)]).build().execute_with(|| {
		let charlie: AccountId = 3u32;
		assert_ok!(Roles::register_role(&charlie, Role::Auditor));
		assert_err!(
			Bank::unlock_funds_auditor(RuntimeOrigin::signed(charlie), ALICE, 200),
			crate::Error::<Runtime>::InvalidLockId
		);
	});
}

#[test]
fn manager_can_set_interest_rate() {
	MockGenesisConfig::with_balances(vec![(ALICE, 1_000)]).build().execute_with(|| {
		let charlie: AccountId = 3u32;
		assert_ok!(Roles::register_role(&charlie, Role::Manager));
		assert_ok!(Bank::set_interest_rate(RuntimeOrigin::signed(charlie), 500));
		assert_eq!(InterestRate::<Runtime>::get(), Perbill::from_percent(5));
		System::assert_last_event(RuntimeEvent::Bank(Event::<Runtime>::InterestRateSet {
			manager: charlie,
			old_interest_rate: Perbill::from_percent(0),
			new_interest_rate: Perbill::from_percent(5),
		}));
	});
}

#[test]
fn incorrect_role_cannot_set_interest_rate() {
	MockGenesisConfig::with_balances(vec![(ALICE, 1_000)]).build().execute_with(|| {
		let charlie: AccountId = 3u32;
		assert_ok!(Roles::register_role(&charlie, Role::Auditor));
		assert_err!(
			Bank::set_interest_rate(RuntimeOrigin::signed(charlie), 500),
			pallet_roles::Error::<Runtime>::IncorrectRole
		);
		assert_err!(
			Bank::set_interest_rate(RuntimeOrigin::signed(ALICE), 500),
			pallet_roles::Error::<Runtime>::IncorrectRole
		);
		assert!(InterestRate::<Runtime>::get().is_zero());
	});
}

#[test]
fn pay_interest() {
	MockGenesisConfig::with_balances(vec![(ALICE, 1_000_000_000)])
		.build()
		.execute_with(|| {
			let charlie: AccountId = 3u32;
			assert_ok!(Roles::register_role(&charlie, Role::Manager));
			assert_ok!(Bank::set_interest_rate(RuntimeOrigin::signed(charlie), 500));
			stake(ALICE, 1_000_000_000);
			Bank::on_finalize(INTEREST_PAYOUT_PERIOD);
			assert_eq!(
				Accounts::<Runtime>::get(ALICE),
				AccountData { free: 0, reserved: 1_000_000_951, locked: vec![] }
			);
			System::assert_last_event(RuntimeEvent::Bank(Event::<Runtime>::InterestPayed {
				interest_rate: Perbill::from_percent(5),
				total_interest_payed: 951,
			}));
			assert!(Bank::check_total_issuance());
		});
}

#[test]
fn can_rotate_treasury() {
	MockGenesisConfig::with_balances(vec![(ALICE, 100)]).build().execute_with(|| {
		// Setup old treasury account and data
		let new_treasury = 10u32;
		TreasuryAccount::<Runtime>::set(Some(TREASURY));
		let account_data = AccountData {
			free: 1_000_000_000,
			reserved: 500,
			locked: vec![
				LockedFund { id: 1, amount: 1_000, reason: LockReason::Auditor },
				LockedFund { id: 2, amount: 2_000, reason: LockReason::Redeem },
			],
		};
		Accounts::<Runtime>::insert(TREASURY, account_data.clone());
		assert_eq!(Bank::treasury(), Ok(TREASURY));

		// Rotate treasury account
		assert_ok!(Bank::rotate_treasury(RawOrigin::Root.into(), new_treasury));

		// Verify all data are migrated
		assert_eq!(Bank::treasury(), Ok(new_treasury));
		assert_eq!(TreasuryAccount::<Runtime>::get(), Some(new_treasury));
		assert_eq!(Accounts::<Runtime>::get(new_treasury), account_data);
	});
}

#[test]
fn incorrect_role_cannot_rotate_treasury() {
	MockGenesisConfig::with_balances(vec![(ALICE, 100)]).build().execute_with(|| {
		TreasuryAccount::<Runtime>::set(Some(TREASURY));
		let new_treasury = 10u32;
		let charlie: AccountId = 3u32;
		assert_ok!(Roles::register_role(&charlie, Role::Auditor));
		assert_ok!(Roles::register_role(&BOB, Role::Manager));

		assert_err!(
			Bank::rotate_treasury(RuntimeOrigin::signed(charlie), new_treasury),
			sp_runtime::DispatchError::BadOrigin
		);
		assert_err!(
			Bank::rotate_treasury(RuntimeOrigin::signed(BOB), new_treasury),
			sp_runtime::DispatchError::BadOrigin
		);
		assert_err!(
			Bank::rotate_treasury(RuntimeOrigin::signed(ALICE), new_treasury),
			sp_runtime::DispatchError::BadOrigin
		);

		assert_eq!(TreasuryAccount::<Runtime>::get(), Some(TREASURY));
		assert_eq!(Accounts::<Runtime>::get(TREASURY).free, 1_000_000);
		assert!(Bank::check_total_issuance());
	});
}

#[test]
fn test_error_can_display_in_treasury() {
	MockGenesisConfig::with_balances(vec![(ALICE, 100)]).build().execute_with(|| {
		let new_treasury = 10u32;
		let collision_account: u32 = ALICE;
		assert_err!(Bank::treasury(), Error::<Runtime>::TreasuryAccountNotSet);
		TreasuryAccount::<Runtime>::set(Some(TREASURY));

		assert_err!(
			Bank::rotate_treasury(RawOrigin::Root.into(), collision_account),
			Error::<Runtime>::AccountIdAlreadyTaken
		);
		assert_ok!(Bank::rotate_treasury(RawOrigin::Root.into(), new_treasury));

		assert_eq!(TreasuryAccount::<Runtime>::get(), Some(new_treasury));
		assert_eq!(Accounts::<Runtime>::get(new_treasury).free, 1_000_000);
		assert!(Bank::check_total_issuance());
	});
}

#[test]
fn funds_can_unlock_after_treasury_rotation() {
	MockGenesisConfig::with_balances(vec![(ALICE, 1_000)]).build().execute_with(|| {
		TreasuryAccount::<Runtime>::set(Some(TREASURY));
		let initial_block = 10;
		let lock_period = 20;
		let unlock_block = 10 + 20;
		let treasury_initial = Accounts::<Runtime>::get(TREASURY).free;
		System::set_block_number(initial_block);

		let new_treasury = 10u32;
		assert_ok!(Roles::register_role(&BOB, Role::Auditor));

		assert_ok!(Bank::lock_funds_auditor(RuntimeOrigin::signed(BOB), ALICE, 100, lock_period));
		assert_ok!(Bank::lock_funds_auditor(
			RuntimeOrigin::signed(BOB),
			TREASURY,
			100,
			lock_period
		));
		assert_eq!(Accounts::<Runtime>::get(ALICE).locked.len(), 1);
		assert_eq!(Accounts::<Runtime>::get(TREASURY).locked.len(), 1);

		assert_ok!(Bank::rotate_treasury(RuntimeOrigin::root(), new_treasury));
		assert_eq!(Accounts::<Runtime>::get(new_treasury).locked.len(), 1);

		// Should unlock all funds
		Bank::on_finalize(unlock_block);

		assert!(Accounts::<Runtime>::get(ALICE).locked.is_empty());
		assert!(Accounts::<Runtime>::get(new_treasury).locked.is_empty());

		assert_eq!(Accounts::<Runtime>::get(ALICE).free, 1_000);
		assert_eq!(Accounts::<Runtime>::get(new_treasury).free, treasury_initial);

		assert!(Bank::check_total_issuance());
	});
}
