//! Unit tests for the tokens module.

#![cfg(test)]

use super::*;
use crate::mock::{
    default_test_ext, Bank, MockGenesisConfig, Runtime, RuntimeEvent, RuntimeOrigin, System, ALICE,
    BOB,
};
use crate::AccountData;
use frame_support::{assert_noop, assert_ok};

#[test]
fn can_deposit() {
    default_test_ext().execute_with(|| {
        assert_eq!(Bank::accounts(&ALICE), AccountData::default());
        assert_eq!(Bank::accounts(&BOB), AccountData::default());
        assert_ok!(Bank::register_role(&ALICE, Role::Manager));
        assert_ok!(Bank::register_role(&BOB, Role::Customer));
        System::reset_events();
        let sender = RuntimeOrigin::signed(ALICE);
        assert_ok!(Bank::deposit(sender, BOB, 1_000));
        // Check that the event was emitted
        assert_eq!(
            System::events()[0].event,
            RuntimeEvent::Bank(Event::<Runtime>::Mint { user: BOB, amount: 1_000 } )
        );
        assert_eq!(
            System::events()[1].event,
            RuntimeEvent::Bank(Event::<Runtime>::Deposit { user: BOB, amount: 1_000 } )
        );
        assert_eq!(
            Accounts::<Runtime>::get(&BOB),
            AccountData {
                free: 1_000,
                reserved: 0
            }
        );
        assert_noop!(Bank::deposit(RuntimeOrigin::signed(ALICE), ALICE, 500), Error::<Runtime>::IncorrectRole);      
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
            assert_eq!(Bank::accounts(&ALICE), AccountData::default());
            assert_ok!(Bank::register_role(&ALICE, Role::Manager));
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
                RuntimeEvent::Bank(Event::<Runtime>::Burn { user: BOB, amount: 100 } )
            );
            assert_eq!(
                System::events()[1].event,
                RuntimeEvent::Bank(Event::<Runtime>::Withdraw { user: BOB, amount: 100 } )
            );
            assert_eq!(
                Accounts::<Runtime>::get(&BOB),
                AccountData {
                    free: 400,
                    reserved: 0
                }
            );
            assert_noop!(Bank::withdraw(RuntimeOrigin::signed(ALICE), BOB, 500), Error::<Runtime>::InsufficientBalance);
            assert_noop!(Bank::withdraw(RuntimeOrigin::signed(ALICE), ALICE, 500), Error::<Runtime>::IncorrectRole);      
            assert!(Bank::check_total_issuance());
        });
}


#[test]
fn can_transfer() {
    MockGenesisConfig::with_balances(vec![(ALICE, 1_000), (BOB, 500)])
        .build()
        .execute_with(|| {
            assert_eq!(
                Accounts::<Runtime>::get(&ALICE),
                AccountData {
                    free: 1_000,
                    reserved: 0
                }
            );
            assert_eq!(
                Accounts::<Runtime>::get(&BOB),
                AccountData {
                    free: 500,
                    reserved: 0
                }
            );
            System::reset_events();
            let sender = RuntimeOrigin::signed(ALICE);
            assert_ok!(Bank::transfer(sender, BOB, 100));
            // Check that the event was emitted
            assert_eq!(
                System::events()[0].event,
                RuntimeEvent::Bank(Event::<Runtime>::Transfer { from: ALICE, to: BOB, amount: 100 } )
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
            assert_noop!(Bank::transfer(RuntimeOrigin::signed(ALICE), BOB, 1_000), Error::<Runtime>::InsufficientBalance);
            assert_eq!(Bank::accounts(3), AccountData::default());
            assert_ok!(Bank::register_role(&3, Role::Manager));
            assert_noop!(Bank::transfer(RuntimeOrigin::signed(3), BOB, 100), Error::<Runtime>::IncorrectRole);
            assert!(Bank::check_total_issuance());
        });
}

#[test]
fn can_register_role() {
    default_test_ext().execute_with(|| {
        let role = Role::Customer;
        assert_eq!(None, AccountRoles::<Runtime>::get(&ALICE));

        // Register Alice with the role
        assert_ok!(Bank::register(RuntimeOrigin::signed(ALICE.clone()), role));

        // Check that the event was emitted
        assert_eq!(
            System::events()[0].event,
            RuntimeEvent::Bank(Event::<Runtime>::RoleRegistered {
                user: ALICE.clone(),
                role
            })
        );

        // Check that Alice's role was registered
        assert_eq!(Bank::role(&ALICE), Some(role));
    });
}

#[test]
fn cannot_reregister_role() {
    default_test_ext().execute_with(|| {
        let role = Role::Customer;
        // Register Alice with the role
        assert_ok!(Bank::register(RuntimeOrigin::signed(ALICE.clone()), role));
        System::reset_events();

        // Try to register again
        assert_noop!(
            Bank::register(RuntimeOrigin::signed(ALICE.clone()), Role::Auditor),
            Error::<Runtime>::AccountAleadyRegistered
        );
        assert_eq!(Bank::role(&ALICE), Some(role));
        assert!(System::events().is_empty());
    });
}

#[test]
fn can_unregister_role() {
    default_test_ext().execute_with(|| {
        let role = Role::Customer;
        assert_ok!(Bank::register(RuntimeOrigin::signed(ALICE.clone()), role));
        assert_eq!(Bank::role(&ALICE), Some(role));
        System::reset_events();

        assert_ok!(Bank::unregister(RuntimeOrigin::signed(ALICE.clone())));
        assert_eq!(Bank::role(&ALICE), None);
        // Check that the event was emitted
        assert_eq!(
            System::events()[0].event,
            RuntimeEvent::Bank(Event::<Runtime>::RoleUnregistered {
                user: ALICE.clone()
            })
        );

        // Check that Alice's role was unregistered
        assert_noop!(
            Bank::unregister(RuntimeOrigin::signed(ALICE.clone())),
            Error::<Runtime>::AccountRoleNotRegistered
        );
    });
}
