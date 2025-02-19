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
use primitives::YEAR;

// Directly moved the stake fund to the reserved account.
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

	MockGenesisConfig::default()
		.with_balances(vec![(ALICE, 1_000_000, 0), (BOB, 50, 0)])
		.build()
		.execute_with(|| {
			assert_eq!(Accounts::<Runtime>::get(ALICE).free, 1_000_000);
			assert_eq!(Accounts::<Runtime>::get(BOB).free, 50);
		});
}

#[test]
fn can_withdraw() {
	MockGenesisConfig::default()
		.with_balances(vec![(BOB, 500, 0)])
		.build()
		.execute_with(|| {
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
	MockGenesisConfig::default()
		.with_balances(vec![(ALICE, 1_000, 0), (BOB, 500, 0)])
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
fn cannot_deal_with_smaller_than_min() {
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
fn cannot_reap_accounts_without_setting_treasury_account() {
	default_test_ext().execute_with(|| {
		// Set up data to reap BOB
		let charlie: AccountId = 3u32;
		assert_ok!(Roles::register_role(&ALICE, Role::Customer));
		assert_ok!(Roles::register_role(&BOB, Role::Customer));
		assert_ok!(Roles::register_role(&charlie, Role::Manager));
		assert_ok!(Bank::deposit(RuntimeOrigin::signed(charlie), BOB, 100));
		assert_eq!(Accounts::<Runtime>::get(BOB).free, 100);
		assert_ok!(Bank::transfer(RuntimeOrigin::signed(BOB), ALICE, 98));
		assert_eq!(Accounts::<Runtime>::get(BOB).free, 2);
		// cannot reap accounts when Treasury is not set.
		Bank::on_finalize(1);
		assert_eq!(Accounts::<Runtime>::get(BOB).free, 2);
		// After setting TreasuryAccount, reap_account is working.
		TreasuryAccount::<Runtime>::set(Some(TREASURY));
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
	MockGenesisConfig::default()
		.with_balances(vec![(ALICE, 1_000, 0)])
		.build()
		.execute_with(|| {
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
	MockGenesisConfig::default()
		.with_balances(vec![(ALICE, 1_000, 0)])
		.build()
		.execute_with(|| {
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
	MockGenesisConfig::default()
		.with_balances(vec![(ALICE, 1_000, 0)])
		.build()
		.execute_with(|| {
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
	MockGenesisConfig::default()
		.with_balances(vec![(ALICE, 1_000, 0)])
		.build()
		.execute_with(|| {
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
	MockGenesisConfig::default()
		.with_balances(vec![(ALICE, 1_000, 0)])
		.build()
		.execute_with(|| {
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
	MockGenesisConfig::default()
		.with_balances(vec![(ALICE, 1_000, 0)])
		.build()
		.execute_with(|| {
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
	MockGenesisConfig::default()
		.with_balances(vec![(ALICE, 1_000, 0)])
		.build()
		.execute_with(|| {
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
	MockGenesisConfig::default()
		.with_balances(vec![(ALICE, 1_000, 0)])
		.build()
		.execute_with(|| {
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
	MockGenesisConfig::default()
		.with_balances(vec![(ALICE, 1_000_000_000, 0)])
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
	MockGenesisConfig::default()
		.with_balances(vec![(ALICE, 100, 0)])
		.build()
		.execute_with(|| {
			// Setup old treasury account and data
			let new_treasury = 10u32;
			let old_treasury_total = Accounts::<Runtime>::get(TREASURY).total();
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
			System::assert_last_event(RuntimeEvent::Bank(
				Event::<Runtime>::TreasuryAccountRotated { old: Some(TREASURY), new: new_treasury },
			));

			// Verify all data are migrated
			assert_eq!(Bank::treasury(), Ok(new_treasury));
			assert_eq!(TreasuryAccount::<Runtime>::get(), Some(new_treasury));
			assert_eq!(Accounts::<Runtime>::get(new_treasury), account_data);
			// Verify old data do not exist
			assert_eq!(Accounts::<Runtime>::get(TREASURY), AccountData::default());

			TotalIssuance::<Runtime>::mutate(|total| {
				*total =
					total.saturating_sub(old_treasury_total).saturating_add(account_data.total());
			});
			assert!(Bank::check_total_issuance());
		});
}

#[test]
fn can_rotate_treasury_without_setting_treasury() {
	MockGenesisConfig::default()
		.with_balances(vec![(ALICE, 100, 0)])
		.build()
		.execute_with(|| {
			let new_treasury = 10u32;
			// Verify treasury account is not set.
			assert_err!(Bank::treasury(), Error::<Runtime>::TreasuryAccountNotSet);

			// Rotate treasury account
			assert_ok!(Bank::rotate_treasury(RawOrigin::Root.into(), new_treasury));
			System::assert_last_event(RuntimeEvent::Bank(
				Event::<Runtime>::TreasuryAccountRotated { old: None, new: new_treasury },
			));

			// Verify all data are migrated
			assert_eq!(Bank::treasury(), Ok(new_treasury));
			assert_eq!(TreasuryAccount::<Runtime>::get(), Some(new_treasury));
			assert_eq!(Accounts::<Runtime>::get(new_treasury), AccountData::default());

			assert!(Bank::check_total_issuance());
		});
}

#[test]
fn test_error_cases_in_rotating_treasury() {
	MockGenesisConfig::default()
		.with_balances(vec![(ALICE, 100, 0)])
		.build()
		.execute_with(|| {
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
	MockGenesisConfig::default()
		.with_balances(vec![(ALICE, 1_000, 0)])
		.build()
		.execute_with(|| {
			TreasuryAccount::<Runtime>::set(Some(TREASURY));
			let initial_block = 10;
			let lock_period = 20;
			let unlock_block = 10 + 20;
			let treasury_initial = Accounts::<Runtime>::get(TREASURY).free;
			System::set_block_number(initial_block);

			let new_treasury = 10u32;
			assert_ok!(Roles::register_role(&BOB, Role::Auditor));

			assert_ok!(Bank::lock_funds_auditor(
				RuntimeOrigin::signed(BOB),
				ALICE,
				100,
				lock_period
			));
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

#[test]
fn can_force_transfer() {
	MockGenesisConfig::default()
		.with_balances(vec![(ALICE, 1_000, 0), (BOB, 500, 0)])
		.build()
		.execute_with(|| {
			System::reset_events();
			assert_ok!(Bank::force_transfer(RuntimeOrigin::root(), ALICE, BOB, 100));
			// Check that the event was emitted
			System::assert_last_event(RuntimeEvent::Bank(Event::<Runtime>::Transferred {
				from: ALICE,
				to: BOB,
				amount: 100,
			}));

			assert_eq!(Accounts::<Runtime>::get(BOB).free, 600);
			assert_eq!(Accounts::<Runtime>::get(ALICE).free, 900);
			assert_noop!(
				Bank::force_transfer(RuntimeOrigin::root(), ALICE, BOB, 1_000),
				Error::<Runtime>::InsufficientBalance
			);

			assert!(Bank::check_total_issuance());
		});
}

#[test]
fn can_check_fund_unlock_at() {
	MockGenesisConfig::default()
		.with_balances(vec![(ALICE, 1_000, 0), (BOB, 1_000, 0)])
		.build()
		.execute_with(|| {
			// Check if it works with stake function.
			let unlock_block = System::block_number() + StakePeriod::get();
			assert_ok!(Bank::stake_funds(RuntimeOrigin::signed(ALICE), 100));

			// Verify
			assert_eq!(unlock_block, Bank::fund_unlock_at(ALICE, 1));

			// Check it works with `AccountWithUnlockedFund` stores with multiple lock id.
			let charlie: AccountId = 3u32;
			AccountWithUnlockedFund::<Runtime>::insert(
				10,
				vec![(ALICE, 2), (BOB, 3), (charlie, 4), (ALICE, 5), (ALICE, 6)],
			);
			AccountWithUnlockedFund::<Runtime>::insert(
				20,
				vec![(ALICE, 7), (charlie, 8), (BOB, 9), (ALICE, 10)],
			);

			// Verify
			assert_eq!(10, Bank::fund_unlock_at(ALICE, 2));
			assert_eq!(10, Bank::fund_unlock_at(BOB, 3));
			assert_eq!(10, Bank::fund_unlock_at(charlie, 4));
			assert_eq!(10, Bank::fund_unlock_at(ALICE, 5));
			assert_eq!(10, Bank::fund_unlock_at(ALICE, 6));

			assert_eq!(20, Bank::fund_unlock_at(ALICE, 7));
			assert_eq!(20, Bank::fund_unlock_at(charlie, 8));
			assert_eq!(20, Bank::fund_unlock_at(BOB, 9));
			assert_eq!(20, Bank::fund_unlock_at(ALICE, 10));

			// Verify that if lock id does not exist, the function `fund_unlock_at` will return 0.
			assert_eq!(0, Bank::fund_unlock_at(ALICE, 8));
			assert_eq!(0, Bank::fund_unlock_at(charlie, 9));
			assert_eq!(0, Bank::fund_unlock_at(BOB, 10));
			assert_eq!(0, Bank::fund_unlock_at(4u32, 10));
		});
}

#[test]
fn can_calculate_interest_pa() {
	MockGenesisConfig::default()
		.with_balances(vec![(ALICE, 1_000, 0)])
		.build()
		.execute_with(|| {
			let initial_balance: Balance = 1_000_000_000_000_000_000u128;

			calculate_interest_pa_with_interest_rate(initial_balance, 0u32);

			calculate_interest_pa_with_interest_rate(initial_balance, 1u32);

			calculate_interest_pa_with_interest_rate(initial_balance, 10u32);

			calculate_interest_pa_with_interest_rate(initial_balance, 9_999u32);

			calculate_interest_pa_with_interest_rate(initial_balance, 10_000u32);
		});
}

/// Test interest can be received per year by using Alice account with different interest rate and
/// staked balance.
fn calculate_interest_pa_with_interest_rate(initial_balance: Balance, interest_rate_bps: u32) {
	let payout_times = YEAR / INTEREST_PAYOUT_PERIOD as u32;
	let interest_rate = Perbill::from_rational(interest_rate_bps, 10_000u32);

	Accounts::<Runtime>::mutate(ALICE, |account_data| account_data.reserved = initial_balance);

	InterestRate::<Runtime>::set(interest_rate);

	let expect_interest = Bank::interest_pa(ALICE);

	// Simulating the real interest as it is accumulated through the year
	for _ in 0..payout_times {
		Bank::on_finalize(INTEREST_PAYOUT_PERIOD);
	}

	let actual_interest = Bank::accounts(ALICE).reserved - initial_balance;

	assert_eq_with_precision(expect_interest, actual_interest, 1_000_000u128);
}

/// Assert a and b are equal, within the precision limit. a/p == b / p
fn assert_eq_with_precision<N: Eq + sp_std::ops::Div<Output = N> + Debug + std::marker::Copy>(
	a: N,
	b: N,
	precision: N,
) {
	assert_eq!(a / precision, b / precision);
}
