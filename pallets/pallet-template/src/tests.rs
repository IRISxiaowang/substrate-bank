#![cfg(test)]

use crate::mock::{default_test_ext, Runtime, ALICE};
use crate::*;

#[test]
fn example_test() {
    default_test_ext().execute_with(|| {
        ExampleStorage::<Runtime>::insert(ALICE, Role::Customer);
        assert_eq!(ExampleStorage::<Runtime>::get(&ALICE), Some(Role::Customer));
    });
}
