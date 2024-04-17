#![cfg(test)]

use crate::{mock::*, *};

use codec::Decode;
use frame_support::{assert_noop, assert_ok, traits::Randomness};
use primitives::{Balance, DOLLAR};
use sp_runtime::Percent;

const INITIAL_BALANCE: u128 = 1_000 * DOLLAR;

#[test]
fn can_control_random_output() {
	default_test_ext().execute_with(|| {
		RandomOutput::set(10u32);

		let (random, _) = MockRandom::random(&[]);
		let target =
			<u32>::decode(&mut random.as_ref()).expect("hash should always be > 32 bits") % 100;

		assert_eq!(target, 10u32);
	});
}

#[test]
fn can_set_prize_split() {
	default_test_ext().execute_with(|| {
		let split =
			vec![Percent::from_percent(50), Percent::from_percent(30), Percent::from_percent(20)];
		assert_ok!(Lottery::set_prize_split(RuntimeOrigin::root(), split.clone()));
		System::assert_last_event(RuntimeEvent::Lottery(Event::<Runtime>::PrizeSplitUpdated {
			split,
		}));
	});
}

#[test]
fn cannot_set_prize_split_total_not_equal_to_one() {
	default_test_ext().execute_with(|| {
		let split_more_than_one = vec![
			Percent::from_percent(50),
			Percent::from_percent(30),
			Percent::from_percent(20),
			Percent::from_percent(20),
		];
		let split_less_than_one = vec![Percent::from_percent(50), Percent::from_percent(30)];
		assert_noop!(
			Lottery::set_prize_split(RuntimeOrigin::root(), split_more_than_one),
			Error::<Runtime>::InvalidPrizeSplitTotal
		);
		assert_noop!(
			Lottery::set_prize_split(RuntimeOrigin::root(), split_less_than_one),
			Error::<Runtime>::InvalidPrizeSplitTotal
		);
	});
}

#[test]
fn auditor_and_manager_can_update_ticket_price() {
	default_test_ext().execute_with(|| {
		// Manager Eve
		assert_ok!(Lottery::update_ticket_price(RuntimeOrigin::signed(EVE), 1));
		System::assert_last_event(RuntimeEvent::Lottery(Event::<Runtime>::TicketPriceUpdated {
			new: 1,
		}));

		// Auditor Ferdie
		assert_ok!(Lottery::update_ticket_price(RuntimeOrigin::signed(FERDIE), 2));
		System::assert_last_event(RuntimeEvent::Lottery(Event::<Runtime>::TicketPriceUpdated {
			new: 2,
		}));
	});
}

#[test]
fn customer_cannot_update_ticket_price() {
	default_test_ext().execute_with(|| {
		// Customer Alice
		assert_noop!(
			Lottery::update_ticket_price(RuntimeOrigin::signed(ALICE), 1),
			Error::<Runtime>::IncorrectRole
		);
	});
}

#[test]
fn can_buy_tickets() {
	let mock = MockGenesisConfig { balance: vec![], lotteries: vec![] };
	mock.with_balances(vec![(ALICE, INITIAL_BALANCE), (BOB, INITIAL_BALANCE)])
		.build()
		.execute_with(|| {
			// Manager Eve set ticket price
			assert_ok!(Lottery::update_ticket_price(RuntimeOrigin::signed(EVE), 5 * DOLLAR));

			// Buy tickets
			assert_ok!(Lottery::buy_ticket(RuntimeOrigin::signed(ALICE), 10));
			assert_ok!(Lottery::buy_ticket(RuntimeOrigin::signed(BOB), 20));

			// Verify data
			assert_eq!(TicketsBought::<Runtime>::get(ALICE), 10);
			assert_eq!(Bank::free_balance(&ALICE), 950 * DOLLAR);
			assert_eq!(TicketsBought::<Runtime>::get(BOB), 20);
			assert_eq!(Bank::free_balance(&BOB), 900 * DOLLAR);
			assert_eq!(Bank::free_balance(&PRIZE_POOL_ACCOUNT), 150 * DOLLAR);

			System::assert_has_event(RuntimeEvent::Lottery(Event::<Runtime>::TicketsBought {
				id: ALICE,
				number: 10u32,
				total_price: 50 * DOLLAR,
			}));
			System::assert_last_event(RuntimeEvent::Lottery(Event::<Runtime>::TicketsBought {
				id: BOB,
				number: 20u32,
				total_price: 100 * DOLLAR,
			}));
		});
}

#[test]
fn cannot_buy_tickets_without_setting_ticket_price() {
	let mock = MockGenesisConfig { balance: vec![], lotteries: vec![] };
	mock.with_balances(vec![(ALICE, INITIAL_BALANCE), (BOB, INITIAL_BALANCE)])
		.build()
		.execute_with(|| {
			// Buy tickets without setting ticket price
			TicketPrice::<Runtime>::kill();
			assert_noop!(
				Lottery::buy_ticket(RuntimeOrigin::signed(ALICE), 10),
				Error::<Runtime>::TicketPriceNotSet
			);
		});
}

#[test]
fn basic_end_to_end_works() {
	let mock = MockGenesisConfig { balance: vec![], lotteries: vec![] };
	mock.with_balances(vec![
		(ALICE, INITIAL_BALANCE),
		(BOB, INITIAL_BALANCE),
		(CHARLIE, INITIAL_BALANCE),
		(DAVE, INITIAL_BALANCE),
	])
	.with_lotteries(vec![(ALICE, 10), (BOB, 5), (CHARLIE, 4), (DAVE, 1)])
	.build()
	.execute_with(|| {
		RandomOutput::set(0u32);

		// Manager Eve initialize ticket price $1, so that the pool has $20.
		let total: Balance = 20 * DOLLAR;

		// Default the prize-split is one.
		let won_fund = (Percent::one() - TAX_RATE) * total;

		// Select the winner
		Lottery::on_finalize(LOTTERY_PAYOUT_PERIOD);

		// Verify data
		assert_eq!(Bank::free_balance(&DAVE), INITIAL_BALANCE + won_fund);
		assert_eq!(Bank::free_balance(&PRIZE_POOL_ACCOUNT), Default::default());
		assert_eq!(Bank::free_balance(&TREASURY), DOLLAR);

		assert_eq!(Bank::free_balance(&ALICE), INITIAL_BALANCE);
		assert_eq!(Bank::free_balance(&BOB), INITIAL_BALANCE);
		assert_eq!(Bank::free_balance(&CHARLIE), INITIAL_BALANCE);

		System::assert_last_event(RuntimeEvent::Lottery(Event::<Runtime>::LotteryWon {
			user: DAVE,
			won_fund,
			tax: TAX_RATE * total,
		}));
	});
}

#[test]
fn choose_multiple_winners_works() {
	let mock = MockGenesisConfig { balance: vec![], lotteries: vec![] };
	mock.with_balances(vec![
		(ALICE, INITIAL_BALANCE),
		(BOB, INITIAL_BALANCE),
		(CHARLIE, INITIAL_BALANCE),
		(DAVE, INITIAL_BALANCE),
	])
	.with_lotteries(vec![(ALICE, 10), (BOB, 50), (CHARLIE, 40), (DAVE, 100)])
	.build()
	.execute_with(|| {
		RandomOutput::set(0u32);

		// Set prize split
		let split =
			vec![Percent::from_percent(50), Percent::from_percent(30), Percent::from_percent(20)];
		assert_ok!(Lottery::set_prize_split(RuntimeOrigin::root(), split));

		// Manager Eve initialize ticket price $1, so that the pool has $20.
		// Storage Map iterate order is random but deterministic.
		// In this case the iterate order is Dave Bob Alice
		let total: Balance = 200 * DOLLAR;
		let dave_won: Balance = Percent::from_percent(50) * total;
		let bob_won: Balance = Percent::from_percent(30) * total;
		let alice_won: Balance = Percent::from_percent(20) * total;
		let dave_tax: Balance = TAX_RATE * dave_won;
		let bob_tax: Balance = TAX_RATE * bob_won;
		let alice_tax: Balance = TAX_RATE * alice_won;

		// Select the winner
		Lottery::on_finalize(LOTTERY_PAYOUT_PERIOD);

		// Verify data
		assert_eq!(Bank::free_balance(&ALICE), INITIAL_BALANCE + alice_won - alice_tax);
		assert_eq!(Bank::free_balance(&BOB), INITIAL_BALANCE + bob_won - bob_tax);
		assert_eq!(Bank::free_balance(&CHARLIE), INITIAL_BALANCE);
		assert_eq!(Bank::free_balance(&DAVE), INITIAL_BALANCE + dave_won - dave_tax);

		assert_eq!(Bank::free_balance(&PRIZE_POOL_ACCOUNT), Default::default());
		assert_eq!(Bank::free_balance(&TREASURY), TAX_RATE * total);

		System::assert_has_event(RuntimeEvent::Lottery(Event::<Runtime>::LotteryWon {
			user: DAVE,
			won_fund: dave_won - dave_tax,
			tax: dave_tax,
		}));
		System::assert_has_event(RuntimeEvent::Lottery(Event::<Runtime>::LotteryWon {
			user: BOB,
			won_fund: bob_won - bob_tax,
			tax: bob_tax,
		}));
		System::assert_last_event(RuntimeEvent::Lottery(Event::<Runtime>::LotteryWon {
			user: ALICE,
			won_fund: alice_won - alice_tax,
			tax: alice_tax,
		}));
	});
}

#[test]
fn not_enough_players_works() {
	let mock = MockGenesisConfig { balance: vec![], lotteries: vec![] };
	mock.with_balances(vec![(ALICE, INITIAL_BALANCE), (BOB, INITIAL_BALANCE)])
		.with_lotteries(vec![(ALICE, 100), (BOB, 100)])
		.build()
		.execute_with(|| {
			RandomOutput::set(0u32);

			// Set prize split
			let split = vec![
				Percent::from_percent(50),
				Percent::from_percent(30),
				Percent::from_percent(20),
			];
			assert_ok!(Lottery::set_prize_split(RuntimeOrigin::root(), split));

			// Manager Eve initialize ticket price $1, so that the pool has $20.
			// Storage Map iterate order is random but deterministic.
			// In this case the iterate order is Bob Alice
			let total: Balance = 200 * DOLLAR;
			let bob_won: Balance = Percent::from_percent(50) * total;
			let alice_won: Balance = Percent::from_percent(30) * total;
			let bob_tax: Balance = TAX_RATE * bob_won;
			let alice_tax: Balance = TAX_RATE * alice_won;

			// Select the winner
			Lottery::on_finalize(LOTTERY_PAYOUT_PERIOD);

			// Verify data
			assert_eq!(Bank::free_balance(&ALICE), INITIAL_BALANCE + alice_won - alice_tax);
			assert_eq!(Bank::free_balance(&BOB), INITIAL_BALANCE + bob_won - bob_tax);

			assert_eq!(Bank::free_balance(&PRIZE_POOL_ACCOUNT), total - alice_won - bob_won);
			assert_eq!(Bank::free_balance(&TREASURY), alice_tax + bob_tax);

			System::assert_has_event(RuntimeEvent::Lottery(Event::<Runtime>::LotteryWon {
				user: BOB,
				won_fund: bob_won - bob_tax,
				tax: bob_tax,
			}));
			System::assert_last_event(RuntimeEvent::Lottery(Event::<Runtime>::LotteryWon {
				user: ALICE,
				won_fund: alice_won - alice_tax,
				tax: alice_tax,
			}));
		});
}

#[test]
fn select_winner_works() {
	default_test_ext().execute_with(|| {
		// Set the vec with value from the storage TicketsBought.
		let tickets_bought = (11..21).map(|p| (p, 10u32)).collect::<Vec<_>>();

		// Select winner thousand times.
		for i in 0..1000 {
			RandomOutput::set(i);

			let actual_winner = Lottery::select_winner(tickets_bought.clone());
			let expected_winner = i % 100 / 10 + 11;

			// Verify
			assert_eq!(actual_winner, expected_winner);
		}
	});
}

#[test]
fn force_draw_works() {
	let mock = MockGenesisConfig { balance: vec![], lotteries: vec![] };
	mock.with_balances(vec![
		(ALICE, INITIAL_BALANCE),
		(BOB, INITIAL_BALANCE),
		(CHARLIE, INITIAL_BALANCE),
		(DAVE, INITIAL_BALANCE),
	])
	.with_lotteries(vec![(ALICE, 10), (BOB, 50), (CHARLIE, 40), (DAVE, 100)])
	.build()
	.execute_with(|| {
		RandomOutput::set(0u32);
		System::set_block_number(9999);
		let total: Balance = 200 * DOLLAR;
		let tax = TAX_RATE * total;
		assert_ok!(Lottery::force_draw(RuntimeOrigin::root()));

		// Select the winner
		Lottery::on_finalize(System::block_number());

		// Verify winner DAVE is chosen.
		System::assert_last_event(RuntimeEvent::Lottery(Event::<Runtime>::LotteryWon {
			user: DAVE,
			won_fund: total - tax,
			tax,
		}));

		// Next round
		// Buy tickets
		assert_ok!(Lottery::buy_ticket(RuntimeOrigin::signed(ALICE), 10));
		assert_ok!(Lottery::buy_ticket(RuntimeOrigin::signed(BOB), 20));

		let total_2 = 30 * DOLLAR;
		let tax_2 = TAX_RATE * total_2;

		// Test the lottery is not ready to draw.
		Lottery::on_finalize(System::block_number() + LOTTERY_PAYOUT_PERIOD - 2);
		Lottery::on_finalize(System::block_number() + LOTTERY_PAYOUT_PERIOD - 1);
		Lottery::on_finalize(System::block_number() + LOTTERY_PAYOUT_PERIOD + 1);
		Lottery::on_finalize(System::block_number() + LOTTERY_PAYOUT_PERIOD + 2);

		// Verify `PrizePoolAccount` has total lottery
		assert_eq!(Bank::free_balance(&PRIZE_POOL_ACCOUNT), total_2);

		// Verify draw on the correct block
		Lottery::on_finalize(System::block_number() + LOTTERY_PAYOUT_PERIOD);

		// Verify `PrizePoolAccount` paid out all the prize.
		assert_eq!(Bank::free_balance(&PRIZE_POOL_ACCOUNT), 0);

		System::assert_has_event(RuntimeEvent::Lottery(Event::<Runtime>::TicketsBought {
			id: ALICE,
			number: 10u32,
			total_price: 10 * DOLLAR,
		}));
		System::assert_has_event(RuntimeEvent::Lottery(Event::<Runtime>::TicketsBought {
			id: BOB,
			number: 20u32,
			total_price: 20 * DOLLAR,
		}));
		// Verify winner Alice is chosen.
		System::assert_last_event(RuntimeEvent::Lottery(Event::<Runtime>::LotteryWon {
			user: BOB,
			won_fund: total_2 - tax_2,
			tax: tax_2,
		}));
	});
}
