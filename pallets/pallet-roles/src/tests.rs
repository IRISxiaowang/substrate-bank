#![cfg(test)]

use crate::{
	mock::{
		default_test_ext, AccountRole, MockGenesisConfig, Runtime, RuntimeEvent, RuntimeOrigin,
		System, ALICE, BOB,
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
		assert_ok!(AccountRole::register(RuntimeOrigin::signed(ALICE), role));

		// Check that the event was emitted
		assert_eq!(
			System::events()[0].event,
			RuntimeEvent::AccountRole(Event::<Runtime>::RoleRegistered { user: ALICE, role })
		);

		// Check that Alice's role was registered
		assert_eq!(AccountRole::role(&ALICE), Some(role));
	});
}

#[test]
fn cannot_reregister_role() {
	default_test_ext().execute_with(|| {
		let role = Role::Customer;
		// Register Alice with the role
		assert_ok!(AccountRole::register(RuntimeOrigin::signed(ALICE), role));
		System::reset_events();

		// Try to register again
		assert_noop!(
			AccountRole::register(RuntimeOrigin::signed(ALICE), Role::Auditor),
			Error::<Runtime>::AccountAleadyRegistered
		);
		assert_eq!(AccountRole::role(&ALICE), Some(role));
		assert!(System::events().is_empty());
	});
}

#[test]
fn can_unregister_role() {
	default_test_ext().execute_with(|| {
		let role = Role::Customer;
		assert_ok!(AccountRole::register(RuntimeOrigin::signed(ALICE), role));
		assert_eq!(AccountRole::role(&ALICE), Some(role));
		System::reset_events();

		assert_ok!(AccountRole::unregister(RuntimeOrigin::signed(ALICE)));
		assert_eq!(AccountRole::role(&ALICE), None);
		// Check that the event was emitted
		assert_eq!(
			System::events()[0].event,
			RuntimeEvent::AccountRole(Event::<Runtime>::RoleUnregistered { user: ALICE })
		);

		// Check that Alice's role was unregistered
		assert_noop!(
			AccountRole::unregister(RuntimeOrigin::signed(ALICE)),
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
