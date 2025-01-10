use core::panic;

use crate::*;

pub(crate) fn dispatch_governance_call(call: Box<RuntimeCall>) {
	let mut gov_iter = Governance::authorities().into_iter();
	assert_ok!(Governance::initiate_proposal(
		RuntimeOrigin::signed(gov_iter.next().unwrap()),
		call
	));

	gov_iter.for_each(|member| {
		assert_ok!(Governance::vote(
			RuntimeOrigin::signed(member),
			Governance::proposal_id(),
			true
		));
	});

	Governance::on_finalize(System::block_number());
}

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
		assert_balance(PrizePool.account(), 100 * DOLLAR);

		// LotteryPayoutPeriod is one DAY
		Lottery::on_finalize(DAY);

		if let RuntimeEvent::Lottery(pallet_lottery::Event::<Runtime>::LotteryWon {
			user,
			won_fund,
			tax,
		}) = System::events().into_iter().last().unwrap().event
		{
			assert_balance(user, INITIAL_BALANCE - 40 * DOLLAR + won_fund);
			assert_balance(Treasury.account(), tax);
			assert_balance(PrizePool.account(), 0);
		} else {
			panic!("Last event is not LotteryWon. Events: \n{:?}", System::events());
		}

		// Governance set prize split.
		let prize_split = vec![Percent::from_percent(80), Percent::from_percent(20)];
		let call = Box::new(RuntimeCall::Lottery(pallet_lottery::Call::set_prize_split {
			prize_split: prize_split.clone(),
		}));

		dispatch_governance_call(call);

		assert_eq!(Lottery::prize_split(), prize_split.clone());

		// Reset events
		System::reset_events();

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

#[test]
fn can_draw_lottery_without_enough_winners() {
	ExtBuilder::default().build().execute_with(|| {
		// first round with default prize split.
		assert_ok!(Lottery::buy_ticket(Alice.sign(), 100));

		// Verify
		System::assert_has_event(RuntimeEvent::Lottery(
			pallet_lottery::Event::<Runtime>::TicketsBought {
				id: Alice.account(),
				number: 100u32,
				total_price: 100 * DOLLAR,
			},
		));

		assert_balance(Alice.account(), INITIAL_BALANCE - 100 * DOLLAR);

		//PrizePoolAccount: AccountId = AccountId::from([0xFF; 32]);
		assert_balance(PrizePool.account(), 100 * DOLLAR);

		// Governance set prize split.
		let prize_split = vec![Percent::from_percent(80), Percent::from_percent(20)];
		let call = Box::new(RuntimeCall::Lottery(pallet_lottery::Call::set_prize_split {
			prize_split: prize_split.clone(),
		}));

		dispatch_governance_call(call);

		assert_eq!(Lottery::prize_split(), prize_split.clone());

		Lottery::on_finalize(DAY);

		// Verify
		let won_1 = prize_split[0] * Percent::from_percent(95) * 100 * DOLLAR;
		let tax_1 = prize_split[0] * Percent::from_percent(5) * 100 * DOLLAR;
		let rest = prize_split[1] * 100 * DOLLAR;

		assert_balance(Alice.account(), INITIAL_BALANCE - 100 * DOLLAR + won_1);

		// Check the treasury account received the tax.
		assert_balance(Treasury.account(), tax_1);

		// Check the rest prize is stay in the prize pool account for next round lottery draw.
		assert_balance(PrizePool.account(), rest);

		System::assert_has_event(RuntimeEvent::Lottery(
			pallet_lottery::Event::<Runtime>::LotteryWon {
				user: Alice.account(),
				won_fund: won_1,
				tax: tax_1,
			},
		));

		// Reset events
		System::reset_events();

		// Test the second round if the winner is enough, then all the fund from prize pool will
		// draw to the winners.
		assert_ok!(Lottery::buy_ticket(Bob.sign(), 50));
		assert_ok!(Lottery::buy_ticket(Charlie.sign(), 50));

		Lottery::on_finalize(DAY);

		// Verify
		let won = prize_split[0] * Percent::from_percent(95) * (100 * DOLLAR + rest);
		let tax = prize_split[0] * Percent::from_percent(5) * (100 * DOLLAR + rest);
		let won_2 = prize_split[1] * Percent::from_percent(95) * (100 * DOLLAR + rest);
		let tax_2 = prize_split[1] * Percent::from_percent(5) * (100 * DOLLAR + rest);

		assert_balance(Bob.account(), INITIAL_BALANCE - 50 * DOLLAR + won);
		assert_balance(Charlie.account(), INITIAL_BALANCE - 50 * DOLLAR + won_2);

		// Check the treasury account received the tax.
		assert_balance(Treasury.account(), tax_1 + tax + tax_2);

		// No fund is left here.
		assert_balance(PrizePool.account(), 0);

		System::assert_has_event(RuntimeEvent::Lottery(
			pallet_lottery::Event::<Runtime>::LotteryWon {
				user: Bob.account(),
				won_fund: won,
				tax,
			},
		));
		System::assert_last_event(RuntimeEvent::Lottery(
			pallet_lottery::Event::<Runtime>::LotteryWon {
				user: Charlie.account(),
				won_fund: won_2,
				tax: tax_2,
			},
		));
	});
}

#[test]
fn can_force_draw_lottery() {
	ExtBuilder::default().build().execute_with(|| {
		// first round with default prize split.
		assert_ok!(Lottery::buy_ticket(Alice.sign(), 80));
		assert_ok!(Lottery::buy_ticket(Bob.sign(), 20));

		// Verify
		System::assert_has_event(RuntimeEvent::Lottery(
			pallet_lottery::Event::<Runtime>::TicketsBought {
				id: Alice.account(),
				number: 80u32,
				total_price: 80 * DOLLAR,
			},
		));

		assert_balance(Alice.account(), INITIAL_BALANCE - 80 * DOLLAR);

		//PrizePoolAccount: AccountId = AccountId::from([0xFF; 32]);
		assert_balance(PrizePool.account(), 100 * DOLLAR);

		// Governance force draw.
		let call = Box::new(RuntimeCall::Lottery(pallet_lottery::Call::force_draw {}));

		dispatch_governance_call(call);

		Lottery::on_finalize(System::block_number());

		// Verify
		let won = Percent::from_percent(95) * 100 * DOLLAR;
		let tax = Percent::from_percent(5) * 100 * DOLLAR;

		assert_balance(Alice.account(), INITIAL_BALANCE - 80 * DOLLAR);
		assert_balance(Bob.account(), INITIAL_BALANCE - 20 * DOLLAR + won);

		// Check the treasury account received the tax.
		assert_balance(Treasury.account(), tax);

		System::assert_last_event(RuntimeEvent::Lottery(
			pallet_lottery::Event::<Runtime>::LotteryWon {
				user: Bob.account(),
				won_fund: won,
				tax,
			},
		));
	});
}
