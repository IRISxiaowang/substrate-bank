//! Benchmarking setup for pallet-template
#![cfg(feature = "runtime-benchmarks")]
use super::*;

use frame_benchmarking::v2::*;
use frame_support::{assert_ok, traits::UnfilteredDispatchable};
use frame_system::RawOrigin;
use primitives::DOLLAR;
use sp_runtime::traits::BlockNumberProvider;

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn request_mint() {
		let caller: T::AccountId = whitelisted_caller();
		let data: Vec<u8> = vec![0x4E, 0x46, 0x54];
		let file_name: Vec<u8> = vec![0x46, 0x49, 0x4C, 0x45];
		assert_ok!(T::RoleManager::register_role(&caller, Role::Customer));

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
				data: vec![0x4E, 0x46, 0x54],
				file_name: vec![0x46, 0x49, 0x4C, 0x45],
				state: NftState::Free,
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
		assert_ok!(T::RoleManager::register_role(&caller, Role::Customer));
		assert_ok!(T::RoleManager::register_role(&to_user, Role::Customer));

		Nfts::<T>::insert(
			1u32,
			NftData {
				data: vec![0x4E, 0x46, 0x54],
				file_name: vec![0x46, 0x49, 0x4C, 0x45],
				state: NftState::Free,
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
					data: vec![0x4E, 0x46, 0x54],
					file_name: vec![0x46, 0x49, 0x4C, 0x45],
					state: NftState::Free,
				},
				owner.clone(),
			),
		);

		#[extrinsic_call]
		approve_nft(RawOrigin::Signed(caller), 1u32, Response::Accept);

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
				data: vec![0x4E, 0x46, 0x54],
				file_name: vec![0x46, 0x49, 0x4C, 0x45],
				state: NftState::Free,
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

	fn set_up_pod<T: Config>(caller: T::AccountId, to_user: T::AccountId) {
		// Deposit fund to accounts.
		let fund = 10u128 * DOLLAR;
		let _ = T::Bank::deposit(&caller, fund.into());
		let _ = T::Bank::deposit(&to_user, fund.into());

		// Nft 1 for receiving pod.
		let nft_id_1 = Pallet::<T>::next_nft_id();
		let pod_id_1 = Pallet::<T>::next_pod_id();
		Nfts::<T>::insert(
			nft_id_1,
			NftData {
				data: vec![0x4E, 0x46, 0x54],
				file_name: vec![0x46, 0x49, 0x4C, 0x45],
				state: NftState::POD,
			},
		);
		Owners::<T>::insert(nft_id_1, caller.clone());
		PendingPodNfts::<T>::insert(
			pod_id_1,
			PodInfo { nft_id: nft_id_1, to_user, price: DOLLAR.into() },
		);
		// Nft 2 for creating pod
		let nft_id_2 = Pallet::<T>::next_nft_id();
		Nfts::<T>::insert(
			nft_id_2,
			NftData {
				data: vec![0x4E, 0x46, 0x54],
				file_name: vec![0x46, 0x49, 0x4C, 0x45],
				state: NftState::Free,
			},
		);
		Owners::<T>::insert(nft_id_2, caller);
	}

	#[benchmark]
	fn create_pod() {
		let caller: T::AccountId = whitelisted_caller();
		let to_user: T::AccountId = account("to", 0u32, 0u32);

		assert_ok!(T::RoleManager::register_role(&caller, Role::Customer));
		assert_ok!(T::RoleManager::register_role(&to_user, Role::Customer));

		set_up_pod::<T>(caller.clone(), to_user.clone());

		#[extrinsic_call]
		create_pod(RawOrigin::Signed(caller), to_user, 2u32, DOLLAR.into());

		// Verify
		assert!(UnlockNft::<T>::contains_key(
			frame_system::Pallet::<T>::current_block_number() + T::NftLockedPeriod::get()
		));
		assert!(PendingPodNfts::<T>::contains_key(2));
	}

	#[benchmark]
	fn receive_pod() {
		let caller: T::AccountId = whitelisted_caller();
		let to_user: T::AccountId = account("to", 0u32, 0u32);

		assert_ok!(T::RoleManager::register_role(&caller, Role::Customer));
		assert_ok!(T::RoleManager::register_role(&to_user, Role::Customer));

		set_up_pod::<T>(caller.clone(), to_user.clone());

		#[extrinsic_call]
		receive_pod(RawOrigin::Signed(to_user.clone()), 1u32, Response::Accept, None);

		// Verify
		assert_eq!(Owners::<T>::get(1), Some(to_user));
		assert!(!PendingPodNfts::<T>::contains_key(1));
	}

	#[benchmark]
	fn cancel_pod() {
		let caller: T::AccountId = whitelisted_caller();
		let to_user: T::AccountId = account("to", 0u32, 0u32);

		assert_ok!(T::RoleManager::register_role(&caller, Role::Customer));
		assert_ok!(T::RoleManager::register_role(&to_user, Role::Customer));

		set_up_pod::<T>(caller.clone(), to_user.clone());

		#[extrinsic_call]
		cancel_pod(RawOrigin::Signed(caller.clone()), 1u32);

		// Verify
		assert_eq!(Owners::<T>::get(1), Some(caller));
		assert!(!PendingPodNfts::<T>::contains_key(1));
		assert_eq!(
			Nfts::<T>::get(1),
			Some(NftData {
				data: vec![0x4E, 0x46, 0x54],
				file_name: vec![0x46, 0x49, 0x4C, 0x45],
				state: NftState::Free
			})
		);
	}

	impl_benchmark_test_suite!(Pallet, crate::mock::default_test_ext(), crate::mock::Runtime);
}
