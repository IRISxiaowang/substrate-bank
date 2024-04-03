#![cfg(test)]

use crate::{mock::*, *};
use frame_support::{assert_noop, assert_ok};

/// Set up an Nft to each storage which is easier to test burn, transfer and audit functionality.
fn set_up_nfts() {
	Nfts::<Runtime>::insert(
		1u32,
		NftData {
			nft_id: 1u32,
			data: vec![0x4E, 0x46, 0x54],
			file_name: vec![0x46, 0x49, 0x4C, 0x45],
		},
	);
	Owners::<Runtime>::insert(1u32, ALICE);
	PendingNft::<Runtime>::insert(
		2u32,
		(
			NftData {
				nft_id: 2u32,
				data: vec![0x4E, 0x46, 0x54],
				file_name: vec![0x46, 0x49, 0x4C, 0x45],
			},
			BOB,
		),
	);
	assert!(Owners::<Runtime>::contains_key(1));
	assert!(Nfts::<Runtime>::contains_key(1));
	assert!(PendingNft::<Runtime>::contains_key(2));
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

		System::assert_last_event(RuntimeEvent::Nft(Event::<Runtime>::NftBurned {
			owner: ALICE,
			nft_id: 1u32,
		}));
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
fn can_approve_nft_pass() {
	default_test_ext().execute_with(|| {
		set_up_nfts();

		assert_ok!(Nft::approve_nft(RuntimeOrigin::signed(FERDIE), 2u32, true));

		assert_eq!(Owners::<Runtime>::get(2), Some(BOB));
		assert!(Nfts::<Runtime>::contains_key(2));
		assert!(!PendingNft::<Runtime>::contains_key(2));

		System::assert_has_event(RuntimeEvent::Nft(Event::<Runtime>::NftMinted {
			owner: BOB,
			nft_id: 2u32,
		}));

		System::assert_last_event(RuntimeEvent::Nft(Event::<Runtime>::NftAudited {
			nft_id: 2u32,
			approve: true,
		}));
	});
}

#[test]
fn can_approve_nft_fail() {
	default_test_ext().execute_with(|| {
		set_up_nfts();
		assert!(PendingNft::<Runtime>::contains_key(2));

		assert_ok!(Nft::approve_nft(RuntimeOrigin::signed(FERDIE), 2u32, false));

		assert!(!Owners::<Runtime>::contains_key(2));
		assert!(!Nfts::<Runtime>::contains_key(2));
		assert!(!PendingNft::<Runtime>::contains_key(2));

		System::assert_last_event(RuntimeEvent::Nft(Event::<Runtime>::NftAudited {
			nft_id: 2u32,
			approve: false,
		}));
	});
}

#[test]
fn can_force_burn() {
	default_test_ext().execute_with(|| {
		set_up_nfts();

		assert_ok!(Nft::force_burn(RuntimeOrigin::root(), 1u32));

		assert!(!Owners::<Runtime>::contains_key(1));
		assert!(!Nfts::<Runtime>::contains_key(1));

		System::assert_last_event(RuntimeEvent::Nft(Event::<Runtime>::NftBurned {
			owner: ALICE,
			nft_id: 1u32,
		}));
	});
}

#[test]
fn test_error_cases() {
	default_test_ext().execute_with(|| {
		let valid_file_name = vec![0x46, 0x49, 0x4C, 0x45];
		let valid_data = vec![0x4E, 0x46, 0x54];
		let invalid_file_name = vec![0x00; 260];
		let invalid_data = vec![0x00; 1_001];

		// Ferdie is auditor who can not create nft
		assert_noop!(
			Nft::request_mint(
				RuntimeOrigin::signed(FERDIE),
				valid_file_name.clone(),
				valid_data.clone()
			),
			Error::<Runtime>::IncorrectRole
		);
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

		// Created an Nft successfully.
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

		// NftId is not exist
		assert_noop!(
			Nft::approve_nft(RuntimeOrigin::signed(FERDIE), 2u32, true),
			Error::<Runtime>::InvalidNftId
		);

		// Approved the Nft 1
		assert_ok!(Nft::approve_nft(RuntimeOrigin::signed(FERDIE), 1u32, true));

		System::assert_last_event(RuntimeEvent::Nft(Event::<Runtime>::NftAudited {
			nft_id: 1u32,
			approve: true,
		}));

		// User can not burn the Nft is not belong to him.
		assert_noop!(Nft::burned(RuntimeOrigin::signed(BOB), 1u32), Error::<Runtime>::Unauthorised);

		// Burned the Nft 1
		assert_ok!(Nft::burned(RuntimeOrigin::signed(ALICE), 1u32));

		System::assert_last_event(RuntimeEvent::Nft(Event::<Runtime>::NftBurned {
			owner: ALICE,
			nft_id: 1u32,
		}));
	});
}
