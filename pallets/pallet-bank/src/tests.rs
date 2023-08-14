//! Unit tests for the tokens module.

#![cfg(test)]

use crate::mock::{
    default_test_ext, AccountId, Bank, MockGenesisConfig, Runtime, RuntimeEvent, RuntimeOrigin,
    System, TreasuryAccount, ALICE, BOB, Roles,
};
use crate::*;
use frame_support::{assert_noop, assert_ok};

#[test]
fn can_deposit() {
    default_test_ext().execute_with(|| {
        assert_eq!(Bank::accounts(&ALICE), AccountData::default());
        assert_eq!(Bank::accounts(&BOB), AccountData::default());
        assert_ok!(Roles::register_role(&ALICE, Role::Manager));
        assert_ok!(Roles::register_role(&BOB, Role::Customer));
        System::reset_events();
        let sender = RuntimeOrigin::signed(ALICE);
        assert_ok!(Bank::deposit(sender, BOB, 1_000));
        // Check that the event was emitted
        assert_eq!(
            System::events()[0].event,
            RuntimeEvent::Bank(Event::<Runtime>::Deposit {
                user: BOB,
                amount: 1_000
            })
        );
        assert_eq!(
            Accounts::<Runtime>::get(&BOB),
            AccountData {
                free: 1_000,
                reserved: 0
            }
        );
        assert_noop!(
            Bank::deposit(RuntimeOrigin::signed(ALICE), ALICE, 500),
            pallet_roles::Error::<Runtime>::IncorrectRole
        );
        assert!(Bank::check_total_issuance());
    });

    MockGenesisConfig::with_balances(vec![(ALICE, 1_000_000), (BOB, 50)])
        .build()
        .execute_with(|| {
            assert_eq!(
                Accounts::<Runtime>::get(&ALICE),
                AccountData {
                    free: 1_000_000,
                    reserved: 0
                }
            );
            assert_eq!(
                Accounts::<Runtime>::get(&BOB),
                AccountData {
                    free: 50,
                    reserved: 0
                }
            );
        });
}

#[test]
fn can_withdraw() {
    MockGenesisConfig::with_balances(vec![(BOB, 500)])
        .build()
        .execute_with(|| {
            assert_ok!(Roles::register_role(&ALICE, Role::Manager));
            assert_eq!(
                Accounts::<Runtime>::get(&BOB),
                AccountData {
                    free: 500,
                    reserved: 0
                }
            );
            System::reset_events();
            assert_ok!(Bank::withdraw(RuntimeOrigin::signed(ALICE), BOB, 100));
            assert_eq!(
                System::events()[0].event,
                RuntimeEvent::Bank(Event::<Runtime>::Withdraw {
                    user: BOB,
                    amount: 100
                })
            );
            assert_eq!(
                Accounts::<Runtime>::get(&BOB),
                AccountData {
                    free: 400,
                    reserved: 0
                }
            );
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
            assert_eq!(
                System::events()[0].event,
                RuntimeEvent::Bank(Event::<Runtime>::Transfer {
                    from: ALICE,
                    to: BOB,
                    amount: 100
                })
            );

            assert_eq!(
                Accounts::<Runtime>::get(&BOB),
                AccountData {
                    free: 600,
                    reserved: 0
                }
            );
            assert_eq!(
                Accounts::<Runtime>::get(&ALICE),
                AccountData {
                    free: 900,
                    reserved: 0
                }
            );
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
        assert_eq!(Bank::accounts(&ALICE), AccountData::default());
        assert_eq!(Bank::accounts(&BOB), AccountData::default());
        assert_eq!(Bank::accounts(&charlie), AccountData::default());
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
    });
}

#[test]
fn can_reaped() {
    default_test_ext().execute_with(|| {
        let charlie: AccountId = 3u32;
        assert_eq!(Bank::accounts(&ALICE), AccountData::default());
        assert_eq!(Bank::accounts(&BOB), AccountData::default());
        assert_eq!(Bank::accounts(&charlie), AccountData::default());
        assert_ok!(Roles::register_role(&ALICE, Role::Customer));
        assert_ok!(Roles::register_role(&BOB, Role::Customer));
        assert_ok!(Roles::register_role(&charlie, Role::Manager));

        assert_ok!(Bank::deposit(RuntimeOrigin::signed(charlie), BOB, 100));
        assert_eq!(
            Accounts::<Runtime>::get(&BOB),
            AccountData {
                free: 100,
                reserved: 0
            }
        );
        assert_ok!(Bank::transfer(RuntimeOrigin::signed(BOB), ALICE, 98));
        assert_eq!(
            Accounts::<Runtime>::get(&BOB),
            AccountData {
                free: 2,
                reserved: 0
            }
        );
        assert_eq!(
            Accounts::<Runtime>::get(&ALICE),
            AccountData {
                free: 98,
                reserved: 0
            }
        );

        System::reset_events();
        Bank::reap_accounts();
        assert_eq!(
            System::events()[0].event,
            RuntimeEvent::Bank(Event::<Runtime>::Reaped {
                user: BOB.clone(),
                dust: 2
            })
        );
        assert_eq!(Accounts::<Runtime>::get(&BOB), Default::default());
        assert_eq!(
            Accounts::<Runtime>::get(&TreasuryAccount::get()),
            AccountData {
                free: 1_000_002,
                reserved: 0
            }
        );

        assert!(Bank::check_total_issuance());
    });
}
