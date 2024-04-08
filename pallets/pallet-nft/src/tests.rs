#![cfg(test)]

use crate::{mock::*, *};
use frame_support::{assert_noop, assert_ok};

/// Set up an Nft to each storage which is easier to test burn, transfer and audit functionality.
fn set_up_nfts() {
	let nft_id_1 = Nft::next_nft_id();
	let nft_id_2 = Nft::next_nft_id();

	Nfts::<Runtime>::insert(
		nft_id_1,
		NftData { data: vec![0x4E, 0x46, 0x54], file_name: vec![0x46, 0x49, 0x4C, 0x45] },
	);
	Owners::<Runtime>::insert(nft_id_1, ALICE);
	PendingNft::<Runtime>::insert(
		nft_id_2,
		(NftData { data: vec![0x4E, 0x46, 0x54], file_name: vec![0x46, 0x49, 0x4C, 0x45] }, BOB),
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
		assert_ok!(Nft::approve_nft(RuntimeOrigin::signed(FERDIE), 2u32, true));

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
		assert_ok!(Nft::approve_nft(RuntimeOrigin::signed(FERDIE), 2u32, false));

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
			Nft::approve_nft(RuntimeOrigin::signed(FERDIE), 3u32, true),
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
