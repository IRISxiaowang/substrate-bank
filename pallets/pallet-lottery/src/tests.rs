#![cfg(test)]

use crate::{mock::*, *};

use codec::Decode;
use frame_support::{assert_noop, assert_ok, traits::Randomness};
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
			old: 1_000,
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
	mock.with_balances(vec![(ALICE, 1_000), (BOB, 1_000)]).build().execute_with(|| {
		// Manager Eve set ticket price
		assert_ok!(Lottery::update_ticket_price(RuntimeOrigin::signed(EVE), 1));

		// Buy tickets
		assert_ok!(Lottery::buy_ticket(RuntimeOrigin::signed(ALICE), 10));
		assert_ok!(Lottery::buy_ticket(RuntimeOrigin::signed(BOB), 20));

		// Verify data
		assert_eq!(PlayersAndLotteries::<Runtime>::get(ALICE), Some(10));
		assert_eq!(Bank::free_balance(&ALICE), 990);
		assert_eq!(PlayersAndLotteries::<Runtime>::get(BOB), Some(20));
		assert_eq!(Bank::free_balance(&BOB), 980);
		assert_eq!(Bank::free_balance(&PRIZE_POOL_ACCOUNT), 30);

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
fn cannot_buy_tickets_without_seting_ticket_price() {
	let mock = MockGenesisConfig { balance: vec![], lotteries: vec![] };
	mock.with_balances(vec![(ALICE, 1_000), (BOB, 1_000)]).build().execute_with(|| {
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
	mock.with_balances(vec![(ALICE, 1_000), (BOB, 1_000), (CHARLIE, 1_000), (DAVE, 1_000)])
		.with_lotteries(vec![(ALICE, 10), (BOB, 5), (CHARLIE, 4), (DAVE, 1)])
		.build()
		.execute_with(|| {
			RandomOutput::set(10u32);

			// Manager Eve initialize ticket price 1_000, so that the pool has $20_000.
			let total: Balance = 20_000;

			// Select the winner
			Lottery::on_finalize(LOTTERY_PAYOUT_PERIOD);

			// Verify data

			System::assert_last_event(RuntimeEvent::Lottery(Event::<Runtime>::LotteryWon {
				user: ALICE,
				won_fund: (Percent::one() - TAX_RATE) * total,
				tax: TAX_RATE * total,
			}));
		});
}
