#![cfg(test)]

use crate::{mock::*, *};
use frame_support::{assert_noop, assert_ok};
use primitives::DOLLAR;

#[test]
fn can_create_auction() {
	default_test_ext().execute_with(|| {
		assert_ok!(Auction::create_auction(
			RuntimeOrigin::signed(ALICE),
			1u32,
			Some(100u128),
			Some(10 * DOLLAR),
			Some(20 * DOLLAR)
		));

		assert!(Auctions::<Runtime>::contains_key(1));
		assert!(AuctionsExpiryBlock::<Runtime>::contains_key(AuctionLength::get() + 1));

		System::assert_last_event(RuntimeEvent::Auction(Event::<Runtime>::AuctionCreated {
			who: ALICE,
			auction_id: 1u32,
		}));
	});
}

#[test]
fn can_bid() {
	default_test_ext().execute_with(|| {
		Auctions::<Runtime>::insert(
			1u32,
			AuctionData {
				nft_id: 1u32,
				start: Some(100u128),
				reserve: Some(10 * DOLLAR),
				buy_now: Some(20 * DOLLAR),
				expiry_block: 100,
				current_bid: None,
			},
		);

		assert_ok!(Auction::bid(RuntimeOrigin::signed(BOB), 1u32, 10 * DOLLAR));

		assert_eq!(Auctions::<Runtime>::get(1).unwrap().current_bid, Some((2u32, 10 * DOLLAR)));

		System::assert_last_event(RuntimeEvent::Auction(Event::<Runtime>::BidRegistered {
			auction_id: 1u32,
			new_bidder: 2u32,
			new_price: 10 * DOLLAR,
		}));
	});
}

#[test]
fn can_cancel_auction() {
	default_test_ext().execute_with(|| {
		Auctions::<Runtime>::insert(
			1u32,
			AuctionData {
				nft_id: 1u32,
				start: Some(100u128),
				reserve: Some(10 * DOLLAR),
				buy_now: Some(20 * DOLLAR),
				expiry_block: 100,
				current_bid: Some((2u32, 5 * DOLLAR)),
			},
		);

		assert_ok!(Auction::cancel_auction(RuntimeOrigin::signed(ALICE), 1u32));

		assert!(!Auctions::<Runtime>::contains_key(1));

		System::assert_last_event(RuntimeEvent::Auction(Event::<Runtime>::AuctionCanceled {
			auction_id: 1u32,
		}));
	});
}

#[test]
fn can_force_cancel_auction() {
	default_test_ext().execute_with(|| {
		Auctions::<Runtime>::insert(
			1u32,
			AuctionData {
				nft_id: 1u32,
				start: Some(100u128),
				reserve: Some(10 * DOLLAR),
				buy_now: Some(20 * DOLLAR),
				expiry_block: 100,
				current_bid: Some((2u32, 15 * DOLLAR)),
			},
		);

		assert_ok!(Auction::force_cancel(1u32));

		assert!(!Auctions::<Runtime>::contains_key(1));

		assert_ok!(Nft::ensure_nft_owner(&ALICE, 1u32));

		assert_eq!(TransferHistory::get()[0], (BIDS_POOL_ACCOUNT, BOB, 15 * DOLLAR));

		System::assert_last_event(RuntimeEvent::Auction(Event::<Runtime>::AuctionCanceled {
			auction_id: 1u32,
		}));
	});
}

#[test]
fn can_success_auction() {
	default_test_ext().execute_with(|| {
		Auctions::<Runtime>::insert(
			1u32,
			AuctionData {
				nft_id: 1u32,
				start: Some(100u128),
				reserve: Some(10 * DOLLAR),
				buy_now: Some(20 * DOLLAR),
				expiry_block: 100,
				current_bid: Some((2u32, 10 * DOLLAR)),
			},
		);

		AuctionsExpiryBlock::<Runtime>::insert(100, vec![1u32]);

		Auction::on_finalize(100);

		assert!(!Auctions::<Runtime>::contains_key(1));

		assert_ok!(Nft::ensure_nft_owner(&BOB, 1u32));

		assert_eq!(TransferHistory::get()[0], (BIDS_POOL_ACCOUNT, TREASURY, DOLLAR));
		assert_eq!(TransferHistory::get()[1], (BIDS_POOL_ACCOUNT, ALICE, 9 * DOLLAR));

		System::assert_last_event(RuntimeEvent::Auction(Event::<Runtime>::AuctionSucceeded {
			auction_id: 1u32,
			to: BOB,
			asset: 1u32,
			price: 10 * DOLLAR,
		}));
	});
}

#[test]
fn can_expire_auction() {
	default_test_ext().execute_with(|| {
		Auctions::<Runtime>::insert(
			1u32,
			AuctionData {
				nft_id: 1u32,
				start: Some(100u128),
				reserve: Some(10 * DOLLAR),
				buy_now: Some(20 * DOLLAR),
				expiry_block: 100,
				current_bid: Some((2u32, 5 * DOLLAR)),
			},
		);

		AuctionsExpiryBlock::<Runtime>::insert(100, vec![1u32]);

		Auction::on_finalize(100);

		assert!(!Auctions::<Runtime>::contains_key(1));

		assert_ok!(Nft::ensure_nft_owner(&ALICE, 1u32));

		assert_eq!(TransferHistory::get()[0], (BIDS_POOL_ACCOUNT, BOB, 5 * DOLLAR));

		System::assert_last_event(RuntimeEvent::Auction(Event::<Runtime>::AuctionExpired {
			auction_id: 1u32,
		}));
	});
}

#[test]
fn can_bid_buy_now() {
	default_test_ext().execute_with(|| {
		Auctions::<Runtime>::insert(
			1u32,
			AuctionData {
				nft_id: 1u32,
				start: Some(100u128),
				reserve: Some(10 * DOLLAR),
				buy_now: Some(20 * DOLLAR),
				expiry_block: 100,
				current_bid: None,
			},
		);

		assert_ok!(Auction::bid(RuntimeOrigin::signed(BOB), 1u32, 20 * DOLLAR));

		assert!(!Auctions::<Runtime>::contains_key(1));

		assert_ok!(Nft::ensure_nft_owner(&BOB, 1u32));

		assert_eq!(TransferHistory::get()[0], (BOB, BIDS_POOL_ACCOUNT, 20 * DOLLAR));
		assert_eq!(TransferHistory::get()[1], (BIDS_POOL_ACCOUNT, TREASURY, 2 * DOLLAR));
		assert_eq!(TransferHistory::get()[2], (BIDS_POOL_ACCOUNT, ALICE, 18 * DOLLAR));

		System::assert_last_event(RuntimeEvent::Auction(Event::<Runtime>::AuctionSucceeded {
			auction_id: 1u32,
			to: BOB,
			asset: 1u32,
			price: 20 * DOLLAR,
		}));
	});
}

#[test]
fn can_extend_bid() {
	default_test_ext().execute_with(|| {
		Auctions::<Runtime>::insert(
			1u32,
			AuctionData {
				nft_id: 1u32,
				start: Some(100u128),
				reserve: Some(10 * DOLLAR),
				buy_now: Some(20 * DOLLAR),
				expiry_block: 100,
				current_bid: None,
			},
		);

		System::set_block_number(99);

		assert_ok!(Auction::bid(RuntimeOrigin::signed(BOB), 1u32, 10 * DOLLAR));

		assert_eq!(Auctions::<Runtime>::get(1).unwrap().expiry_block, 109);
		assert_eq!(AuctionsExpiryBlock::<Runtime>::get(109), vec![1u32]);

		System::set_block_number(105);
		assert_ok!(Auction::bid(RuntimeOrigin::signed(BOB), 1u32, 15 * DOLLAR));

		assert_eq!(Auctions::<Runtime>::get(1).unwrap().expiry_block, 115);
		assert_eq!(AuctionsExpiryBlock::<Runtime>::get(115), vec![1u32]);

		assert_eq!(TransferHistory::get()[0], (BOB, BIDS_POOL_ACCOUNT, 10 * DOLLAR));
		assert_eq!(TransferHistory::get()[1], (BIDS_POOL_ACCOUNT, BOB, 10 * DOLLAR));
		assert_eq!(TransferHistory::get()[2], (BOB, BIDS_POOL_ACCOUNT, 15 * DOLLAR));

		System::assert_last_event(RuntimeEvent::Auction(Event::<Runtime>::BidRegistered {
			new_bidder: BOB,
			auction_id: 1u32,
			new_price: 15 * DOLLAR,
		}));
	});
}

#[test]
fn error_handling() {
	default_test_ext().execute_with(|| {
		Auctions::<Runtime>::insert(
			1u32,
			AuctionData {
				nft_id: 1u32,
				start: Some(100u128),
				reserve: Some(10 * DOLLAR),
				buy_now: Some(20 * DOLLAR),
				expiry_block: 100,
				current_bid: Some((2u32, 11 * DOLLAR)),
			},
		);

		assert_noop!(
			Auction::bid(RuntimeOrigin::signed(BOB), 2u32, 5 * DOLLAR),
			Error::<Runtime>::InvalidAuctionId
		);

		assert_noop!(
			Auction::cancel_auction(RuntimeOrigin::signed(ALICE), 2u32),
			Error::<Runtime>::InvalidAuctionId
		);

		assert_noop!(
			Auction::bid(RuntimeOrigin::signed(BOB), 1u32, 4 * DOLLAR),
			Error::<Runtime>::BidPriceTooLow
		);

		assert_noop!(
			Auction::cancel_auction(RuntimeOrigin::signed(ALICE), 1u32),
			Error::<Runtime>::CannotCancelAuction
		);
	});
}
