use crate::*;

#[test]
fn can_set_and_accrue_interest_rate() {
	ExtBuilder::default().build().execute_with(|| {
		// Set interest rate to 10%
		assert_ok!(Bank::set_interest_rate(RuntimeOrigin::signed(MANAGER.into()), 1_000u32));

		let amount = 1_000 * DOLLAR;

		assert_ok!(Bank::stake_funds(RuntimeOrigin::signed(ALICE.into()), amount));

		// Staking period = 150 blocks
		Bank::on_finalize(System::block_number() + 150u32);

		assert_staked(ALICE.into(), amount);
	});
}
