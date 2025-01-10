use crate::*;
use lottery::dispatch_governance_call;
use pallet_auction::AuctionData;
use pallet_nft::{CancelReason, NftData};
use primitives::{NftState, Response};
use xy_chain_runtime::{Auction, Nft};

fn create_an_nft() {
	// Create an nft.
	let file_name = vec![0x46, 0x49, 0x4C, 0x45];
	let data = vec![0x4E, 0x46, 0x54];
	assert_ok!(Nft::request_mint(Alice.sign(), file_name.clone(), data));

	// Verify
	System::assert_last_event(RuntimeEvent::Nft(pallet_nft::Event::<Runtime>::NFTPending {
		file_name,
		nft_id: 1u32,
	}));

	// Auditor approved the nft
	assert_ok!(Nft::approve_nft(Auditor.sign(), 1u32, Response::Accept));

	// The nft minted
	assert_eq!(pallet_nft::Owners::<Runtime>::get(1), Some(Alice.account()));
	assert!(pallet_nft::Nfts::<Runtime>::contains_key(1));
	assert!(!pallet_nft::PendingNft::<Runtime>::contains_key(1));

	System::assert_last_event(RuntimeEvent::Nft(pallet_nft::Event::<Runtime>::NftMinted {
		owner: Alice.account(),
		nft_id: 1u32,
	}));
}

#[test]
fn can_force_burn() {
	ExtBuilder::default().build().execute_with(|| {
		// Set up an nft.
		create_an_nft();

		const BID: u128 = 7 * DOLLAR;

		// Create an auction.
		assert_ok!(Auction::create_auction(
			Alice.sign(),
			1u32,
			Some(DOLLAR),
			Some(5 * DOLLAR),
			Some(10 * DOLLAR)
		));

		// Check storage Auctions.
		assert!(pallet_auction::Auctions::<Runtime>::contains_key(1u32));

		// Bob bid the auction.
		assert_ok!(Auction::bid(Bob.sign(), 1u32, BID));

		// Check storage Auctions.
		assert_eq!(
			pallet_auction::Auctions::<Runtime>::get(1u32),
			Some(AuctionData {
				nft_id: 1u32,
				start: Some(DOLLAR),
				reserve: Some(5 * DOLLAR),
				buy_now: Some(10 * DOLLAR),
				expiry_block: 1 + DAY,
				current_bid: Some((Bob.account(), BID))
			})
		);

		// Check Bob balance minus 7 dollars for biding.
		assert_balance(Bob.account(), INITIAL_BALANCE - BID);

		// Check Alice balance minus 1 dollar with start auction fee.
		assert_balance(Alice.account(), INITIAL_BALANCE - DOLLAR);

		// Governance force burn.
		let call = Box::new(RuntimeCall::Nft(pallet_nft::Call::force_burn { nft_id: 1u32 }));

		dispatch_governance_call(call);

		// Check Bob balance that the 7 dollars are back to Bob's account.
		assert_balance(Bob.account(), INITIAL_BALANCE);

		// Check all the storages are cleaned.
		assert!(!pallet_nft::Nfts::<Runtime>::contains_key(1u32));
		assert!(!pallet_nft::Owners::<Runtime>::contains_key(1u32));
		assert!(!pallet_auction::Auctions::<Runtime>::contains_key(1u32));

		// Check event
		System::assert_has_event(RuntimeEvent::Nft(pallet_nft::Event::<Runtime>::NftBurned {
			nft_id: 1u32,
		}));
	});
}

#[test]
fn can_create_pod() {
	ExtBuilder::default().build().execute_with(|| {
		// Set up an nft.
		create_an_nft();

		// Create nft pod.
		assert_ok!(Nft::create_pod(Alice.sign(), Bob.account(), 1u32, DOLLAR));

		// NftLockedPeriod = 1day blocks
		assert!(pallet_nft::PodExpiry::<Runtime>::contains_key(System::block_number() + DAY));
		assert!(pallet_nft::PendingPodNfts::<Runtime>::contains_key(1));

		System::assert_last_event(RuntimeEvent::Nft(pallet_nft::Event::<Runtime>::NftPodCreated {
			from: Alice.account(),
			to: Bob.account(),
			nft_id: 1u32,
			price: DOLLAR,
		}));

		// PodFee = 1dollar
		assert_balance(Alice.account(), INITIAL_BALANCE - DOLLAR);
		// Fee is paid.
		assert_balance(Treasury.account(), DOLLAR);
	});
}

#[test]
fn can_receiver_accept_pod() {
	ExtBuilder::default().build().execute_with(|| {
		// Set up an nft.
		create_an_nft();

		// Create nft pod.
		assert_ok!(Nft::create_pod(Alice.sign(), Bob.account(), 1u32, DOLLAR));

		// Receive pod.
		assert_ok!(Nft::receive_pod(Bob.sign(), 1u32, Response::Accept, Some(DOLLAR)));

		// The nft 1 is on pod.
		assert_eq!(pallet_nft::Owners::<Runtime>::get(1), Some(Bob.account()));
		assert!(!pallet_nft::PendingPodNfts::<Runtime>::contains_key(1));

		System::assert_last_event(RuntimeEvent::Nft(pallet_nft::Event::<Runtime>::NftDelivered {
			seller: Alice.account(),
			buyer: Bob.account(),
			nft_id: 1u32,
			price: DOLLAR,
			tips: DOLLAR,
		}));

		// PodFee = 1dollar
		assert_balance(Alice.account(), INITIAL_BALANCE + DOLLAR);
		assert_balance(Bob.account(), INITIAL_BALANCE - 2 * DOLLAR);
		assert_balance(Treasury.account(), DOLLAR);
	});
}

#[test]
fn can_receiver_reject_pod() {
	ExtBuilder::default().build().execute_with(|| {
		// Set up an nft.
		create_an_nft();

		// Create nft pod.
		assert_ok!(Nft::create_pod(Alice.sign(), Bob.account(), 1u32, DOLLAR));

		// Receive pod.
		assert_ok!(Nft::receive_pod(Bob.sign(), 1u32, Response::Reject, None));

		// The nft 1 is on pod.
		assert_eq!(pallet_nft::Owners::<Runtime>::get(1), Some(Alice.account()));
		assert!(!pallet_nft::PendingPodNfts::<Runtime>::contains_key(1));

		System::assert_last_event(RuntimeEvent::Nft(
			pallet_nft::Event::<Runtime>::NftPodRejected { nft_id: 1u32 },
		));

		// PodFee = 1dollar
		assert_balance(Alice.account(), INITIAL_BALANCE - DOLLAR);
		assert_balance(Bob.account(), INITIAL_BALANCE);
		assert_balance(Treasury.account(), DOLLAR);
	});
}

#[test]
fn can_sender_cancel_pod() {
	ExtBuilder::default().build().execute_with(|| {
		// Set up an nft.
		create_an_nft();

		// Create nft pod.
		assert_ok!(Nft::create_pod(Alice.sign(), Bob.account(), 1u32, DOLLAR));

		// Cancel pod.
		assert_ok!(Nft::cancel_pod(Alice.sign(), 1u32));

		// The nft 1 is on pod.
		assert_eq!(pallet_nft::Owners::<Runtime>::get(1), Some(Alice.account()));
		assert!(!pallet_nft::PendingPodNfts::<Runtime>::contains_key(1));
		assert!(matches!(
			pallet_nft::Nfts::<Runtime>::get(1),
			Some(NftData {
				data,
				state: NftState::Free,
				..
			}) if data.len() == 3
		));

		System::assert_last_event(RuntimeEvent::Nft(
			pallet_nft::Event::<Runtime>::NftPodCanceled {
				nft_id: 1u32,
				reason: CancelReason::Canceled,
			},
		));

		// PodFee = 1dollar
		assert_balance(Alice.account(), INITIAL_BALANCE - DOLLAR);
		assert_balance(Bob.account(), INITIAL_BALANCE);
		assert_balance(Treasury.account(), DOLLAR);
	});
}
