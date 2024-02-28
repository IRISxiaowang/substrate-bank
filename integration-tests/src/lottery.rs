use core::panic;

use crate::*;

#[test]
fn can_set_and_accrue_lottery_prize_split() {
	ExtBuilder::default().build().execute_with(|| {
		// first round with default prize split.
		assert_ok!(Lottery::buy_ticket(Alice.sign(), 10));
		assert_ok!(Lottery::buy_ticket(Bob.sign(), 20));
		assert_ok!(Lottery::buy_ticket(Charlie.sign(), 30));
		assert_ok!(Lottery::buy_ticket(Dave.sign(), 40));

		// Verify
		System::assert_has_event(RuntimeEvent::Lottery(
			pallet_lottery::Event::<Runtime>::TicketsBought {
				id: Alice.account(),
				number: 10u32,
				total_price: 10 * DOLLAR,
			},
		));

		assert_balance(Alice.account(), INITIAL_BALANCE - 10 * DOLLAR);
		assert_balance(Bob.account(), INITIAL_BALANCE - 20 * DOLLAR);
		assert_balance(Charlie.account(), INITIAL_BALANCE - 30 * DOLLAR);
		assert_balance(Dave.account(), INITIAL_BALANCE - 40 * DOLLAR);

		//PrizePoolAccount: AccountId = AccountId::from([0xFF; 32]);
		assert_balance([0xFF; 32].into(), 100 * DOLLAR);

		// LotteryPayoutPeriod is one DAY
		Lottery::on_finalize(DAY);

		if let RuntimeEvent::Lottery(pallet_lottery::Event::<Runtime>::LotteryWon {
			user,
			won_fund,
			tax,
		}) = System::events().into_iter().last().unwrap().event
		{
			println!("{:?} won. Amount: {}, tax: {}", user, won_fund, tax);
			assert_balance(user, INITIAL_BALANCE - 40 * DOLLAR + won_fund);
			assert_balance(Treasury.account(), tax);
			assert_balance([0xFF; 32].into(), 0);
		} else {
			panic!("Last event is not LotteryWon. Events: \n{:?}", System::events());
		}

		// Governance set prize split.
		let prize_split = vec![Percent::from_percent(80), Percent::from_percent(20)];
		let call = Box::new(RuntimeCall::Lottery(pallet_lottery::Call::set_prize_split {
			prize_split: prize_split.clone(),
		}));
		assert_ok!(Governance::initiate_proposal(Gov0.sign(), call));
		assert_ok!(Governance::vote(Gov1.sign(), 1, true));
		assert_ok!(Governance::vote(Gov2.sign(), 1, true));

		Governance::on_finalize(System::block_number());

		assert_eq!(Lottery::prize_split(), prize_split.clone());

		// Second round lottery with new prize split.
		assert_ok!(Lottery::buy_ticket(Alice.sign(), 10));
		assert_ok!(Lottery::buy_ticket(Bob.sign(), 20));
		assert_ok!(Lottery::buy_ticket(Charlie.sign(), 30));
		assert_ok!(Lottery::buy_ticket(Dave.sign(), 40));

		// Verify
		System::assert_has_event(RuntimeEvent::Lottery(
			pallet_lottery::Event::<Runtime>::TicketsBought {
				id: Alice.account(),
				number: 10u32,
				total_price: 10 * DOLLAR,
			},
		));
		// Alice bought 20 tickets in total.
		assert_balance(Alice.account(), INITIAL_BALANCE - 20 * DOLLAR);

		Lottery::on_finalize(DAY);

		let won_1 = prize_split[0] * Percent::from_percent(95) * 100 * DOLLAR;
		let won_2 = prize_split[1] * Percent::from_percent(95) * 100 * DOLLAR;
		let tax_1 = prize_split[0] * Percent::from_percent(5) * 100 * DOLLAR;
		let tax_2 = prize_split[1] * Percent::from_percent(5) * 100 * DOLLAR;

		assert_balance(Dave.account(), INITIAL_BALANCE - 80 * DOLLAR + 95 * DOLLAR + won_1);
		assert_balance(Alice.account(), INITIAL_BALANCE - 20 * DOLLAR);
		assert_balance(Bob.account(), INITIAL_BALANCE - 40 * DOLLAR + won_2);
		assert_balance(Charlie.account(), INITIAL_BALANCE - 60 * DOLLAR);

		// Check the treasury account received the tax.
		assert_balance(Treasury.account(), (tax_1 + tax_2) * 2);

		System::assert_has_event(RuntimeEvent::Lottery(
			pallet_lottery::Event::<Runtime>::LotteryWon {
				user: Dave.account(),
				won_fund: won_1,
				tax: tax_1,
			},
		));

		System::assert_last_event(RuntimeEvent::Lottery(
			pallet_lottery::Event::<Runtime>::LotteryWon {
				user: Bob.account(),
				won_fund: won_2,
				tax: tax_2,
			},
		));
	});
}
