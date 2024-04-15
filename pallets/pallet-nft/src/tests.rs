#![cfg(test)]

use crate::{mock::*, *};
use frame_support::{assert_noop, assert_ok};
use primitives::{NftId, DOLLAR};

/// Set up an Nft to each storage which is easier to test burn, transfer and audit functionality.
fn set_up_nfts() {
	let nft_id_1 = Nft::next_nft_id();
	let nft_id_2 = Nft::next_nft_id();

	Nfts::<Runtime>::insert(
		nft_id_1,
		NftData {
			data: vec![0x4E, 0x46, 0x54],
			file_name: vec![0x46, 0x49, 0x4C, 0x45],
			state: NftState::Free,
		},
	);
	Owners::<Runtime>::insert(nft_id_1, ALICE);
	PendingNft::<Runtime>::insert(
		nft_id_2,
		(
			NftData {
				data: vec![0x4E, 0x46, 0x54],
				file_name: vec![0x46, 0x49, 0x4C, 0x45],
				state: NftState::Free,
			},
			BOB,
		),
	);
	assert!(Owners::<Runtime>::contains_key(nft_id_1));
	assert!(Nfts::<Runtime>::contains_key(nft_id_1));
	assert!(PendingNft::<Runtime>::contains_key(nft_id_2));
}

#[test]
fn can_request_mint() {
	default_test_ext().execute_with(|| {
		let file_name = vec![0x46, 0x49, 0x4C, 0x45];
		let data = vec![0x4E, 0x46, 0x54];
		assert_ok!(Nft::request_mint(RuntimeOrigin::signed(ALICE), file_name.clone(), data));

		assert!(PendingNft::<Runtime>::contains_key(1));

		System::assert_last_event(RuntimeEvent::Nft(Event::<Runtime>::NFTPending {
			file_name,
			nft_id: 1u32,
		}));
	});
}

#[test]
fn can_burned() {
	default_test_ext().execute_with(|| {
		set_up_nfts();
		assert_ok!(Nft::burned(RuntimeOrigin::signed(ALICE), 1u32));

		assert!(!Owners::<Runtime>::contains_key(1));
		assert!(!Nfts::<Runtime>::contains_key(1));

		System::assert_last_event(RuntimeEvent::Nft(Event::<Runtime>::NftBurned { nft_id: 1u32 }));
	});
}

#[test]
fn can_transfer() {
	default_test_ext().execute_with(|| {
		set_up_nfts();
		assert_eq!(Owners::<Runtime>::get(1), Some(ALICE));

		assert_ok!(Nft::transfer(RuntimeOrigin::signed(ALICE), BOB, 1u32));

		assert_eq!(Owners::<Runtime>::get(1), Some(BOB));
		assert!(Nfts::<Runtime>::contains_key(1));

		System::assert_last_event(RuntimeEvent::Nft(Event::<Runtime>::NftTransferred {
			from: ALICE,
			to: BOB,
			nft_id: 1u32,
		}));
	});
}

#[test]
fn can_auditor_approve_nft() {
	default_test_ext().execute_with(|| {
		set_up_nfts();

		// Check nft 2 does not exist
		assert!(!Nfts::<Runtime>::contains_key(2));
		assert!(!Owners::<Runtime>::contains_key(2));
		assert!(PendingNft::<Runtime>::contains_key(2));

		// Auditor approved the nft
		assert_ok!(Nft::approve_nft(RuntimeOrigin::signed(FERDIE), 2u32, Response::Accept));

		// The nft minted
		assert_eq!(Owners::<Runtime>::get(2), Some(BOB));
		assert!(Nfts::<Runtime>::contains_key(2));
		assert!(!PendingNft::<Runtime>::contains_key(2));

		System::assert_has_event(RuntimeEvent::Nft(Event::<Runtime>::NftMinted {
			owner: BOB,
			nft_id: 2u32,
		}));
	});
}

#[test]
fn can_auditor_reject_nft() {
	default_test_ext().execute_with(|| {
		set_up_nfts();

		// Check nft 2 does not exist
		assert!(!Owners::<Runtime>::contains_key(2));
		assert!(!Nfts::<Runtime>::contains_key(2));
		assert!(PendingNft::<Runtime>::contains_key(2));

		// Auditor rejected the nft
		assert_ok!(Nft::approve_nft(RuntimeOrigin::signed(FERDIE), 2u32, Response::Reject));

		// The nft is deleted from pendingNft storage.
		assert!(!Owners::<Runtime>::contains_key(2));
		assert!(!Nfts::<Runtime>::contains_key(2));
		assert!(!PendingNft::<Runtime>::contains_key(2));

		System::assert_last_event(RuntimeEvent::Nft(Event::<Runtime>::NftRejected {
			nft_id: 2u32,
		}));
	});
}

#[test]
fn can_force_burn() {
	default_test_ext().execute_with(|| {
		set_up_nfts();

		// Check the nft 1 is exist.
		assert!(Owners::<Runtime>::contains_key(1));
		assert!(Nfts::<Runtime>::contains_key(1));

		// Force burned by governance.
		assert_ok!(Nft::force_burn(RuntimeOrigin::root(), 1u32));

		// The nft 1 is deleted.
		assert!(!Owners::<Runtime>::contains_key(1));
		assert!(!Nfts::<Runtime>::contains_key(1));

		System::assert_last_event(RuntimeEvent::Nft(Event::<Runtime>::NftBurned { nft_id: 1u32 }));
	});
}

#[test]
fn test_error_unauthorise() {
	default_test_ext().execute_with(|| {
		set_up_nfts();
		// Bob cannot burn nft which belong to Alice.
		assert_noop!(Nft::burned(RuntimeOrigin::signed(BOB), 1u32), Error::<Runtime>::Unauthorised);

		// Bob cannot transfer the nft which belong to Alice.
		assert_noop!(
			Nft::transfer(RuntimeOrigin::signed(BOB), ALICE, 1u32),
			Error::<Runtime>::Unauthorised
		);
	});
}

#[test]
fn test_error_invalid_nft_id() {
	default_test_ext().execute_with(|| {
		set_up_nfts();
		assert_noop!(
			Nft::burned(RuntimeOrigin::signed(ALICE), 2u32),
			Error::<Runtime>::InvalidNftId
		);

		assert_noop!(
			Nft::transfer(RuntimeOrigin::signed(ALICE), BOB, 2u32),
			Error::<Runtime>::InvalidNftId
		);

		assert_noop!(
			Nft::approve_nft(RuntimeOrigin::signed(FERDIE), 3u32, Response::Accept),
			Error::<Runtime>::InvalidNftId
		);
	});
}

#[test]
fn test_error_data_and_file_name_too_large() {
	default_test_ext().execute_with(|| {
		let valid_file_name = vec![0x00; 255];
		let valid_data = vec![0x00; 1_000];
		let invalid_file_name = vec![0x00; 256];
		let invalid_data = vec![0x00; 1_001];

		// File name is too large
		assert_noop!(
			Nft::request_mint(RuntimeOrigin::signed(ALICE), invalid_file_name, valid_data.clone()),
			Error::<Runtime>::FileNameTooLarge
		);
		// Data is too large
		assert_noop!(
			Nft::request_mint(RuntimeOrigin::signed(ALICE), valid_file_name.clone(), invalid_data),
			Error::<Runtime>::DataTooLarge
		);

		// Created an Nft successfully when file name length within 255 and data length within
		// 1_000.
		assert_ok!(Nft::request_mint(
			RuntimeOrigin::signed(ALICE),
			valid_file_name.clone(),
			valid_data
		));

		assert!(PendingNft::<Runtime>::contains_key(1));

		System::assert_last_event(RuntimeEvent::Nft(Event::<Runtime>::NFTPending {
			file_name: valid_file_name,
			nft_id: 1u32,
		}));
	});
}

#[test]
fn test_ensure_nft_state_is_free() {
	default_test_ext().execute_with(|| {
		Nfts::<Runtime>::insert(
			1u32,
			NftData {
				data: vec![0x4E, 0x46, 0x54],
				file_name: vec![0x46, 0x49, 0x4C, 0x45],
				state: NftState::Free,
			},
		);
		Nfts::<Runtime>::insert(
			2u32,
			NftData {
				data: vec![0x4E, 0x46, 0x54],
				file_name: vec![0x46, 0x49, 0x4C, 0x45],
				state: NftState::POD,
			},
		);

		assert_ok!(Nft::ensure_nft_state(1u32, NftState::Free));

		assert_noop!(
			Nft::ensure_nft_state(2u32, NftState::Free),
			Error::<Runtime>::NftStateNotMatch
		);

		assert_noop!(Nft::ensure_nft_state(3u32, NftState::Free), Error::<Runtime>::InvalidNftId);
	});
}

#[test]
fn can_change_nft_state() {
	default_test_ext().execute_with(|| {
		Nfts::<Runtime>::insert(
			1u32,
			NftData {
				data: vec![0x4E, 0x46, 0x54],
				file_name: vec![0x46, 0x49, 0x4C, 0x45],
				state: NftState::Free,
			},
		);

		assert_ok!(Nft::change_nft_state(1u32, NftState::POD));

		assert_eq!(Nfts::<Runtime>::get(1u32).unwrap().state, NftState::POD);

		assert_noop!(Nft::change_nft_state(2u32, NftState::Free), Error::<Runtime>::InvalidNftId);
	});
}

#[test]
fn can_create_pod() {
	default_test_ext().execute_with(|| {
		set_up_nfts();

		// Check the nft 1 is exist.
		assert!(Nfts::<Runtime>::contains_key(1));

		// Create pod.
		assert_ok!(Nft::create_pod(RuntimeOrigin::signed(ALICE), BOB, 1u32, DOLLAR));

		// The nft 1 is on pod.
		assert!(UnlockNft::<Runtime>::contains_key(
			System::block_number() + NftLockedPeriod::get()
		));
		assert!(PendingPodNfts::<Runtime>::contains_key(1));

		System::assert_last_event(RuntimeEvent::Nft(Event::<Runtime>::NftPodCreated {
			from: ALICE,
			to: BOB,
			nft_id: 1u32,
			price: DOLLAR,
		}));
	});
}

fn set_up_pod(pod_id: PodId, nft_id: NftId) {
	// Set up created pod data.
	Nfts::<Runtime>::insert(
		nft_id,
		NftData {
			data: vec![0x4E, 0x46, 0x54],
			file_name: vec![0x46, 0x49, 0x4C, 0x45],
			state: NftState::POD,
		},
	);
	Owners::<Runtime>::insert(nft_id, ALICE);
	UnlockNft::<Runtime>::insert(
		System::block_number() + NFT_LOCKED_PERIOD,
		vec![(pod_id, nft_id)],
	);
	PendingPodNfts::<Runtime>::insert(pod_id, PodInfo { nft_id, to_user: BOB, price: DOLLAR });
}

#[test]
fn can_receive_pod() {
	default_test_ext().execute_with(|| {
		let pod_id = Nft::next_pod_id();
		let nft_id = Nft::next_nft_id();
		set_up_pod(pod_id, nft_id);

		// Receive pod.
		assert_ok!(Nft::receive_pod(
			RuntimeOrigin::signed(BOB),
			pod_id,
			Response::Accept,
			Some(DOLLAR)
		));

		// The nft 1 is on pod.
		assert_eq!(Owners::<Runtime>::get(nft_id), Some(BOB));
		assert!(!PendingPodNfts::<Runtime>::contains_key(1));

		System::assert_last_event(RuntimeEvent::Nft(Event::<Runtime>::NftDelivered {
			seller: ALICE,
			buyer: BOB,
			nft_id,
			price: DOLLAR,
			tips: DOLLAR,
		}));
	});
}

#[test]
fn can_buyer_reject_pod() {
	default_test_ext().execute_with(|| {
		let pod_id = Nft::next_pod_id();
		let nft_id = Nft::next_nft_id();
		set_up_pod(pod_id, nft_id);

		// Receive pod.
		assert_ok!(Nft::receive_pod(RuntimeOrigin::signed(BOB), pod_id, Response::Reject, None));

		// The nft 1 is on pod.
		assert_eq!(Owners::<Runtime>::get(nft_id), Some(ALICE));
		assert!(!PendingPodNfts::<Runtime>::contains_key(1));

		System::assert_last_event(RuntimeEvent::Nft(Event::<Runtime>::NftRejected { nft_id }));
	});
}

#[test]
fn can_seller_cancel_pod() {
	default_test_ext().execute_with(|| {
		let pod_id = Nft::next_pod_id();
		let nft_id = Nft::next_nft_id();
		set_up_pod(pod_id, nft_id);

		// Receive pod.
		assert_ok!(Nft::cancel_pod(RuntimeOrigin::signed(ALICE), pod_id));

		// The nft 1 is on pod.
		assert_eq!(Owners::<Runtime>::get(nft_id), Some(ALICE));
		assert!(!PendingPodNfts::<Runtime>::contains_key(1));
		assert_eq!(
			Nfts::<Runtime>::get(nft_id),
			Some(NftData {
				data: vec![0x4E, 0x46, 0x54],
				file_name: vec![0x46, 0x49, 0x4C, 0x45],
				state: NftState::Free
			})
		);

		System::assert_last_event(RuntimeEvent::Nft(Event::<Runtime>::CancelReason {
			nft_id,
			reason: UnlockReason::Canceled,
		}));
	});
}

#[test]
fn test_error_nft_not_for_pod() {
	default_test_ext().execute_with(|| {
		let pod_id = Nft::next_pod_id();
		let nft_id = Nft::next_nft_id();
		set_up_pod(pod_id, nft_id);

		assert_noop!(
			Nft::receive_pod(RuntimeOrigin::signed(BOB), 2u32, Response::Accept, None),
			Error::<Runtime>::NftNotForPod
		);

		assert_noop!(
			Nft::cancel_pod(RuntimeOrigin::signed(ALICE), 2u32),
			Error::<Runtime>::NftNotForPod
		);
	});
}

#[test]
fn test_error_incorrect_receiver() {
	default_test_ext().execute_with(|| {
		let pod_id = Nft::next_pod_id();
		let nft_id = Nft::next_nft_id();
		set_up_pod(pod_id, nft_id);

		assert_noop!(
			Nft::receive_pod(RuntimeOrigin::signed(ALICE), pod_id, Response::Accept, None),
			Error::<Runtime>::IncorrectReceiver
		);
	});
}

#[test]
fn test_error_nft_state_not_match() {
	default_test_ext().execute_with(|| {
		// nft 1 state = pod, belong to Alice.
		let pod_id = Nft::next_pod_id();
		let nft_id = Nft::next_nft_id();
		set_up_pod(pod_id, nft_id);

		assert_noop!(
			Nft::create_pod(RuntimeOrigin::signed(ALICE), BOB, nft_id, DOLLAR),
			Error::<Runtime>::NftStateNotMatch
		);
	});
}

#[test]
fn can_nft_pod_expire() {
	default_test_ext().execute_with(|| {
		// nft 1 state = pod, belong to Alice.
		let pod_id = Nft::next_pod_id();
		let nft_id = Nft::next_nft_id();
		set_up_pod(pod_id, nft_id);
		let block = System::block_number() + NftLockedPeriod::get();

		assert!(PendingPodNfts::<Runtime>::contains_key(1));
		assert!(UnlockNft::<Runtime>::contains_key(block));

		Nft::on_finalize(block);

		assert!(!PendingPodNfts::<Runtime>::contains_key(1));
		assert!(!UnlockNft::<Runtime>::contains_key(block));

		System::assert_last_event(RuntimeEvent::Nft(Event::<Runtime>::CancelReason {
			nft_id,
			reason: UnlockReason::Expired,
		}));
	});
}
