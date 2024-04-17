#![cfg(test)]

use crate::{
	mock::{
		default_test_ext, MockGenesisConfig, Roles, Runtime, RuntimeEvent, RuntimeOrigin, System,
		ALICE, BOB, CHARLIE,
	},
	*,
};
use frame_support::{assert_noop, assert_ok};

#[test]
fn can_register_role() {
	default_test_ext().execute_with(|| {
		let role = Role::Customer;
		assert_eq!(None, AccountRoles::<Runtime>::get(ALICE));

		// Register Alice with the role
		assert_ok!(Roles::register_customer(RuntimeOrigin::signed(ALICE)));

		// Check that the event was emitted
		assert_eq!(
			System::events()[0].event,
			RuntimeEvent::Roles(Event::<Runtime>::RoleRegistered { user: ALICE, role })
		);

		// Check that Alice's role was registered
		assert_eq!(Roles::role(&ALICE), Some(role));
	});
}

#[test]
fn cannot_reregister_role() {
	default_test_ext().execute_with(|| {
		let role = Role::Customer;
		// Register Alice with the role
		assert_ok!(Roles::register_customer(RuntimeOrigin::signed(ALICE)));
		System::reset_events();

		// Try to register again
		assert_noop!(
			Roles::register_customer(RuntimeOrigin::signed(ALICE)),
			Error::<Runtime>::AccountAlreadyRegistered
		);
		assert_eq!(Roles::role(&ALICE), Some(role));
		assert!(System::events().is_empty());
	});
}

#[test]
fn can_unregister_role() {
	default_test_ext().execute_with(|| {
		let role = Role::Customer;
		assert_ok!(Roles::register_customer(RuntimeOrigin::signed(ALICE)));
		assert_eq!(Roles::role(&ALICE), Some(role));
		System::reset_events();

		assert_ok!(Roles::unregister(RuntimeOrigin::signed(ALICE)));
		assert_eq!(Roles::role(&ALICE), None);
		// Check that the event was emitted
		assert_eq!(
			System::events()[0].event,
			RuntimeEvent::Roles(Event::<Runtime>::RoleUnregistered { user: ALICE })
		);

		// Check that Alice's role was unregistered
		assert_noop!(
			Roles::unregister(RuntimeOrigin::signed(ALICE)),
			Error::<Runtime>::AccountRoleNotRegistered
		);
	});
}

#[test]
fn test_ensure_not_role() {
	default_test_ext().execute_with(|| {
		AccountRoles::<Runtime>::insert(&ALICE, Role::Customer);
		AccountRoles::<Runtime>::insert(&BOB, Role::Manager);
		AccountRoles::<Runtime>::insert(&CHARLIE, Role::Auditor);

		assert_ok!(Roles::ensure_not_role(&ALICE, Role::Auditor));
		assert_ok!(Roles::ensure_not_role(&BOB, Role::Auditor));

		assert_noop!(
			Roles::ensure_not_role(&ALICE, Role::Customer),
			Error::<Runtime>::IncorrectRole
		);

		assert_noop!(
			Roles::ensure_not_role(&4u32, Role::Customer),
			Error::<Runtime>::AccountRoleNotRegistered
		);
	});
}

#[test]
fn can_bulid() {
	MockGenesisConfig::with_roles(vec![(ALICE, Role::Manager), (BOB, Role::Customer)])
		.build()
		.execute_with(|| {
			assert_eq!(AccountRoles::<Runtime>::get(ALICE), Some(Role::Manager));
			assert_eq!(AccountRoles::<Runtime>::get(BOB), Some(Role::Customer));
		});
}

#[test]
fn can_governance_register_role() {
	default_test_ext().execute_with(|| {
		let manager = Role::Manager;
		let auditor = Role::Auditor;
		let customer = Role::Customer;
		assert_eq!(None, AccountRoles::<Runtime>::get(ALICE));
		assert_eq!(None, AccountRoles::<Runtime>::get(BOB));
		assert_eq!(None, AccountRoles::<Runtime>::get(CHARLIE));

		// Register Alice with the manager role
		assert_ok!(Roles::register_role_governance(RuntimeOrigin::root(), ALICE, manager));

		// Check that the event was emitted
		assert_eq!(
			System::events()[0].event,
			RuntimeEvent::Roles(Event::<Runtime>::RoleRegistered { user: ALICE, role: manager })
		);

		// Check that Alice's role was registered
		assert_eq!(Roles::role(&ALICE), Some(manager));

		// Register Bob with the manager role
		assert_ok!(Roles::register_role_governance(RuntimeOrigin::root(), BOB, auditor));

		// Check that the event was emitted
		assert_eq!(
			System::events()[1].event,
			RuntimeEvent::Roles(Event::<Runtime>::RoleRegistered { user: BOB, role: auditor })
		);

		// Check that Bob's role was registered
		assert_eq!(Roles::role(&BOB), Some(auditor));

		// Register Charlie with the manager role
		assert_ok!(Roles::register_role_governance(RuntimeOrigin::root(), CHARLIE, customer));

		// Check that the event was emitted
		assert_eq!(
			System::events()[2].event,
			RuntimeEvent::Roles(Event::<Runtime>::RoleRegistered { user: CHARLIE, role: customer })
		);

		// Check that Charlie's role was registered
		assert_eq!(Roles::role(&CHARLIE), Some(customer));
	});
}
