//! Unit tests for the tokens module.

#![cfg(test)]

use frame_support::{assert_ok, assert_noop};
use super::*;
use crate::mock::{default_test_ext, MockGenesisConfig, Runtime, ALICE, BOB, Bank, System, RuntimeOrigin};
use crate::AccountData;

#[test]
fn can_deposit() {
    default_test_ext().execute_with(|| {
        assert_eq!(Bank::accounts(&ALICE), AccountData::default());
        assert_eq!(Bank::accounts(&BOB), AccountData::default());
    });

    MockGenesisConfig::with_balances(vec![(ALICE, 1_000_000), (BOB, 50)]).build().execute_with(||{
        assert_eq!(Accounts::<Runtime>::get(&ALICE), AccountData{
            free: 1_000_000, reserved: 0
        });
        assert_eq!(Accounts::<Runtime>::get(&BOB), AccountData{
            free: 50, reserved: 0
        });
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
        let expected_event = mock::RuntimeEvent::Bank(Event::<Runtime>::RoleRegistered{ user: ALICE.clone(), role});
        assert_eq!(System::events()[0].event, expected_event);

        // Check that Alice's role was registered
        assert_eq!(Bank::role(&ALICE), Some(role));

        assert_noop!(Bank::register(RuntimeOrigin::signed(ALICE.clone()), role), Error::<Runtime>::AccountAleadyRegistered);
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
        assert_noop!(Bank::register(RuntimeOrigin::signed(ALICE.clone()), Role::Auditor), Error::<Runtime>::AccountAleadyRegistered);
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
        let expected_event = mock::RuntimeEvent::Bank(Event::<Runtime>::RoleUnregistered{ user: ALICE.clone()});
        assert_eq!(System::events()[0].event, expected_event);

        // Check that Alice's role was unregistered
        assert_noop!(Bank::unregister(RuntimeOrigin::signed(ALICE.clone())), Error::<Runtime>::AccountRoleNotRegistered);
    });
}