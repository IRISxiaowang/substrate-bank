//! Unit tests for the tokens module.

#![cfg(test)]

use super::*;
use crate::mock::{new_test_ext, Runtime, ALICE, BOB};
use crate::AccountData;

#[test]
fn can_deposit() {
    new_test_ext().execute_with(|| {
        assert_eq!(Accounts::<Runtime>::get(&ALICE), AccountData::default());
    });
}
