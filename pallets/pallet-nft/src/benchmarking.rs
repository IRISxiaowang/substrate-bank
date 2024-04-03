//! Benchmarking setup for pallet-template
#![cfg(feature = "runtime-benchmarks")]
use super::*;

use frame_benchmarking::v2::*;
use frame_support::{assert_ok, traits::UnfilteredDispatchable};
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn request_mint() {
		let caller: T::AccountId = whitelisted_caller();
		let data: Vec<u8> = vec![0x4E, 0x46, 0x54];
		let file_name: Vec<u8> = vec![0x46, 0x49, 0x4C, 0x45];

		#[extrinsic_call]
		request_mint(RawOrigin::Signed(caller), data, file_name);

		// Verify
		assert!(PendingNft::<T>::contains_key(1u32));
	}

	#[benchmark]
	fn burned() {
		let caller: T::AccountId = whitelisted_caller();
		Nfts::<T>::insert(
			1u32,
			NftData {
				nft_id: 1u32,
				data: vec![0x4E, 0x46, 0x54],
				file_name: vec![0x46, 0x49, 0x4C, 0x45],
			},
		);
		Owners::<T>::insert(1u32, caller.clone());

		#[extrinsic_call]
		burned(RawOrigin::Signed(caller), 1u32);

		// Verify
		assert!(!Nfts::<T>::contains_key(1));
		assert!(!Owners::<T>::contains_key(1));
	}

	#[benchmark]
	fn transfer() {
		let caller: T::AccountId = whitelisted_caller();
		let to_user: T::AccountId = account("user", 0u32, 0u32);
		Nfts::<T>::insert(
			1u32,
			NftData {
				nft_id: 1u32,
				data: vec![0x4E, 0x46, 0x54],
				file_name: vec![0x46, 0x49, 0x4C, 0x45],
			},
		);
		Owners::<T>::insert(1u32, caller.clone());

		#[extrinsic_call]
		transfer(RawOrigin::Signed(caller), to_user.clone(), 1u32);

		// Verify
		assert_eq!(Owners::<T>::get(1), Some(to_user));
	}

	#[benchmark]
	fn approve_nft() {
		let caller: T::AccountId = whitelisted_caller();
		let owner: T::AccountId = account("owner", 0u32, 0u32);

		assert_ok!(T::RoleManager::register_role(&caller, Role::Auditor));

		PendingNft::<T>::insert(
			1u32,
			(
				NftData {
					nft_id: 1u32,
					data: vec![0x4E, 0x46, 0x54],
					file_name: vec![0x46, 0x49, 0x4C, 0x45],
				},
				owner.clone(),
			),
		);

		#[extrinsic_call]
		approve_nft(RawOrigin::Signed(caller), 1u32, true);

		// Verify
		assert_eq!(Owners::<T>::get(1), Some(owner));
		assert!(Nfts::<T>::contains_key(1));
	}

	#[benchmark]
	fn force_burn() {
		let caller: T::AccountId = whitelisted_caller();

		Nfts::<T>::insert(
			1u32,
			NftData {
				nft_id: 1u32,
				data: vec![0x4E, 0x46, 0x54],
				file_name: vec![0x46, 0x49, 0x4C, 0x45],
			},
		);
		Owners::<T>::insert(1u32, caller);

		let call = Call::<T>::force_burn { nft_id: 1u32 };
		let origin = T::EnsureGovernance::try_successful_origin().unwrap();

		#[block]
		{
			assert_ok!(call.dispatch_bypass_filter(origin));
		}

		// Verify
		assert!(!Nfts::<T>::contains_key(1));
		assert!(!Owners::<T>::contains_key(1));
	}

	impl_benchmark_test_suite!(Pallet, crate::mock::default_test_ext(), crate::mock::Runtime);
}
