//! Benchmarking setup for pallet-template
#![cfg(feature = "runtime-benchmarks")]
use super::*;

use frame_benchmarking::v2::*;
use frame_support::assert_ok;
use frame_system::RawOrigin;
use primitives::{Role, DOLLAR};
use sp_runtime::traits::BlockNumberProvider;

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn create_auction() {
		let caller: T::AccountId = whitelisted_caller();
		assert_ok!(T::RoleManager::register_role(&caller, Role::Customer));
		assert_ok!(T::Bank::deposit(&caller, (1000 * DOLLAR).into()));

		T::NftManager::insert_nft(
			1u32,
			caller.clone(),
			vec![0x46, 0x49, 0x4C, 0x45],
			vec![0x4E, 0x46, 0x54],
		);

		#[extrinsic_call]
		create_auction(
			RawOrigin::Signed(caller),
			1u32,
			Some((5u128 * DOLLAR).into()),
			Some((50u128 * DOLLAR).into()),
			Some((100u128 * DOLLAR).into()),
		);

		// Verify
		assert!(Auctions::<T>::contains_key(1u32));
		assert!(AuctionsExpiryBlock::<T>::contains_key(
			T::AuctionLength::get() + frame_system::Pallet::<T>::current_block_number()
		));
	}

	#[benchmark]
	fn bid() {
		let caller: T::AccountId = whitelisted_caller();
		assert_ok!(T::RoleManager::register_role(&caller, Role::Customer));
		assert_ok!(T::Bank::deposit(&caller, (1000 * DOLLAR).into()));

		T::NftManager::insert_nft(
			1u32,
			caller.clone(),
			vec![0x46, 0x49, 0x4C, 0x45],
			vec![0x4E, 0x46, 0x54],
		);

		Auctions::<T>::insert(
			1u32,
			AuctionData {
				nft_id: 1u32,
				start: Some((5u128 * DOLLAR).into()),
				reserve: Some((50u128 * DOLLAR).into()),
				buy_now: Some((100u128 * DOLLAR).into()),
				expiry_block: 100u32.into(),
				current_bid: None,
			},
		);

		#[extrinsic_call]
		bid(RawOrigin::Signed(caller.clone()), 1u32, (10u128 * DOLLAR).into());

		// Verify
		assert_eq!(
			Auctions::<T>::get(1u32).unwrap().current_bid,
			Some((caller, (10u128 * DOLLAR).into()))
		);
	}

	#[benchmark]
	fn cancel_auction() {
		let caller: T::AccountId = whitelisted_caller();
		let bidder: T::AccountId = account("bidder", 0u32, 0u32);
		assert_ok!(T::RoleManager::register_role(&caller, Role::Customer));
		assert_ok!(T::RoleManager::register_role(&bidder, Role::Customer));

		assert_ok!(T::Bank::deposit(&bidder, (1000 * DOLLAR).into()));

		T::NftManager::insert_nft(
			1u32,
			caller.clone(),
			vec![0x46, 0x49, 0x4C, 0x45],
			vec![0x4E, 0x46, 0x54],
		);

		Auctions::<T>::insert(
			1u32,
			AuctionData {
				nft_id: 1u32,
				start: Some((5u128 * DOLLAR).into()),
				reserve: Some((50u128 * DOLLAR).into()),
				buy_now: Some((100u128 * DOLLAR).into()),
				expiry_block: 100u32.into(),
				current_bid: None,
			},
		);

		assert_ok!(Pallet::<T>::bid(
			RawOrigin::Signed(bidder.clone()).into(),
			1u32,
			(10u128 * DOLLAR).into()
		));

		#[extrinsic_call]
		cancel_auction(RawOrigin::Signed(caller), 1u32);

		// Verify
		assert!(!Auctions::<T>::contains_key(1u32));
	}

	impl_benchmark_test_suite!(Pallet, crate::mock::default_test_ext(), crate::mock::Runtime);
}
