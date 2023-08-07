//! Unit tests for the tokens module.

#![cfg(test)]

use super::*;
use crate::mock::{default_test_ext, MockGenesisConfig, Runtime, ALICE, BOB, Bank};
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
