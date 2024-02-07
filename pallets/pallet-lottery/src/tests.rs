#![cfg(test)]

use crate::{mock::*, *};

use codec::Decode;
use frame_support::{assert_noop, assert_ok, traits::Randomness};
use primitives::DOLLAR;
use sp_runtime::Percent;

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
			old: DOLLAR,
			new: 1,
		}));
		// Auditor Ferdie
		assert_ok!(Lottery::update_ticket_price(RuntimeOrigin::signed(FERDIE), 2));
		System::assert_last_event(RuntimeEvent::Lottery(Event::<Runtime>::TicketPriceUpdated {
			old: 1,
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
	mock.with_balances(vec![(ALICE, 1_000 * DOLLAR), (BOB, 1_000 * DOLLAR)])
		.build()
		.execute_with(|| {
			// Manager Eve set ticket price
			assert_ok!(Lottery::update_ticket_price(RuntimeOrigin::signed(EVE), 5 * DOLLAR));

			// Buy tickets
			assert_ok!(Lottery::buy_ticket(RuntimeOrigin::signed(ALICE), 10));
			assert_ok!(Lottery::buy_ticket(RuntimeOrigin::signed(BOB), 20));

			// Verify data
			assert_eq!(PlayersAndLotteries::<Runtime>::get(ALICE), Some(10));
			assert_eq!(Bank::free_balance(&ALICE), 950 * DOLLAR);
			assert_eq!(PlayersAndLotteries::<Runtime>::get(BOB), Some(20));
			assert_eq!(Bank::free_balance(&BOB), 900 * DOLLAR);
			assert_eq!(Bank::free_balance(&PRIZE_POOL_ACCOUNT), 150 * DOLLAR);

			System::assert_has_event(RuntimeEvent::Lottery(Event::<Runtime>::TicketsBought {
				id: ALICE,
				number: 10u32,
			}));
			System::assert_last_event(RuntimeEvent::Lottery(Event::<Runtime>::TicketsBought {
				id: BOB,
				number: 20u32,
			}));
		});
}

#[test]
fn cannot_buy_tickets_without_setting_ticket_price() {
	let mock = MockGenesisConfig { balance: vec![], lotteries: vec![] };
	mock.with_balances(vec![(ALICE, 1_000 * DOLLAR), (BOB, 1_000 * DOLLAR)])
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
		(ALICE, 1_000 * DOLLAR),
		(BOB, 1_000 * DOLLAR),
		(CHARLIE, 1_000 * DOLLAR),
		(DAVE, 1_000 * DOLLAR),
	])
	.with_lotteries(vec![(ALICE, 10), (BOB, 5), (CHARLIE, 4), (DAVE, 1)])
	.build()
	.execute_with(|| {
		RandomOutput::set(10u32);

		// Manager Eve initialize ticket price $1, so that the pool has $20.
		let total: Balance = 20 * DOLLAR;

		// Select the winner
		Lottery::on_finalize(LOTTERY_PAYOUT_PERIOD);

		// Verify data
		assert_eq!(Bank::free_balance(&ALICE), 1_019 * DOLLAR);
		assert_eq!(Bank::free_balance(&PRIZE_POOL_ACCOUNT), Default::default());
		assert_eq!(Bank::free_balance(&TREASURY), DOLLAR);

		System::assert_last_event(RuntimeEvent::Lottery(Event::<Runtime>::LotteryWon {
			user: ALICE,
			won_fund: (Percent::one() - TAX_RATE) * total,
			tax: TAX_RATE * total,
		}));
	});
}

#[test]
fn choose_multiple_winners_works() {
	let mock = MockGenesisConfig { balance: vec![], lotteries: vec![] };
	mock.with_balances(vec![
		(ALICE, 1_000 * DOLLAR),
		(BOB, 1_000 * DOLLAR),
		(CHARLIE, 1_000 * DOLLAR),
		(DAVE, 1_000 * DOLLAR),
	])
	.with_lotteries(vec![(ALICE, 10), (BOB, 50), (CHARLIE, 40), (DAVE, 100)])
	.build()
	.execute_with(|| {
		RandomOutput::set(10u32);

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
		assert_eq!(Bank::free_balance(&ALICE), 1_000 * DOLLAR + alice_won - alice_tax);
		assert_eq!(Bank::free_balance(&BOB), 1_000 * DOLLAR + bob_won - bob_tax);
		assert_eq!(Bank::free_balance(&CHARLIE), 1_000 * DOLLAR);
		assert_eq!(Bank::free_balance(&DAVE), 1_000 * DOLLAR + dave_won - dave_tax);

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
	mock.with_balances(vec![(ALICE, 1_000 * DOLLAR), (BOB, 1_000 * DOLLAR)])
		.with_lotteries(vec![(ALICE, 100), (BOB, 100)])
		.build()
		.execute_with(|| {
			RandomOutput::set(10u32);

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
			assert_eq!(Bank::free_balance(&ALICE), 1_000 * DOLLAR + alice_won - alice_tax);
			assert_eq!(Bank::free_balance(&BOB), 1_000 * DOLLAR + bob_won - bob_tax);

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
