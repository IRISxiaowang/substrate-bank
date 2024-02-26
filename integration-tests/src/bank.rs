use crate::*;

#[test]
fn can_set_and_accrue_interest_rate() {
	ExtBuilder::default().build().execute_with(|| {
		// Set interest rate to 10%
		assert_ok!(Bank::set_interest_rate(Manager.sign(), 1_000u32));

		let amount = 1_000 * DOLLAR;

		assert_ok!(Bank::stake_funds(Alice.sign(), amount));

		// Staking period = 2day blocks
		Bank::on_finalize(System::block_number() + (2 * DAY));

		assert_staked(Alice.account(), amount);

		// InterestPayoutPeriod = 1day blocks
		Bank::on_finalize(DAY);

		let interest = pallet_bank::InterestRate::<Runtime>::get() / 365u128 * amount;
		assert_staked(Alice.account(), amount + interest);

		// Change to a higher interest rate 20%.
		assert_ok!(Bank::set_interest_rate(Manager.sign(), 2_000u32));

		// Verify
		Bank::on_finalize(DAY);
		let staked_2 = amount + interest;
		let interest_2 = pallet_bank::InterestRate::<Runtime>::get() / 365u128 * staked_2;
		assert_staked(Alice.account(), staked_2 + interest_2);

		// Change to a lower interest rate 5%.
		assert_ok!(Bank::set_interest_rate(Manager.sign(), 500u32));

		// Verify
		Bank::on_finalize(DAY);
		let staked_3 = staked_2 + interest_2;
		let interest_3 = pallet_bank::InterestRate::<Runtime>::get() / 365u128 * staked_3;
		assert_staked(Alice.account(), staked_3 + interest_3);
	});
}

#[test]
fn can_print_test_accounts() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			Gov0.to_string(),
			"Gov0: 0xf0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0"
		);
		assert_eq!(
			Alice.to_string(),
			"Alice: 0x0000000000000000000000000000000000000000000000000000000000000000"
		);
	});
}
