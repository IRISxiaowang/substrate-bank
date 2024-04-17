
//! Autogenerated weights for pallet_bank
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2024-02-27, STEPS: `20`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `Xiaos-MacBook-Pro.local`, CPU: `<UNKNOWN>`
//! EXECUTION: , WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 1024

// Executed Command:
// ./target/release/xy-chain
// benchmark
// pallet
// --extrinsic
// *
// --pallet
// pallet_bank
// --output
// ./pallets/pallet-bank/src/weights.rs
// --steps=20
// --repeat=20
// --template=weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

/// Weight functions needed for pallet_bank.
pub trait WeightInfo {
	fn deposit() -> Weight;
	fn withdraw() -> Weight;
	fn transfer() -> Weight;
	fn stake_funds() -> Weight;
	fn redeem_funds() -> Weight;
	fn lock_funds_auditor() -> Weight;
	fn unlock_funds_auditor() -> Weight;
	fn set_interest_rate() -> Weight;
	fn rotate_treasury() -> Weight;
	fn force_transfer() -> Weight;
}

/// Weights for pallet_bank using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	/// Storage: `Roles::AccountRoles` (r:2 w:0)
	/// Proof: `Roles::AccountRoles` (`max_values`: None, `max_size`: Some(49), added: 2524, mode: `MaxEncodedLen`)
	/// Storage: `Bank::Accounts` (r:1 w:1)
	/// Proof: `Bank::Accounts` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Bank::TotalIssuance` (r:1 w:1)
	/// Proof: `Bank::TotalIssuance` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn deposit() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `635`
		//  Estimated: `6038`
		// Minimum execution time: 24_000_000 picoseconds.
		Weight::from_parts(25_000_000, 6038)
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	/// Storage: `Roles::AccountRoles` (r:2 w:0)
	/// Proof: `Roles::AccountRoles` (`max_values`: None, `max_size`: Some(49), added: 2524, mode: `MaxEncodedLen`)
	/// Storage: `Bank::Accounts` (r:1 w:1)
	/// Proof: `Bank::Accounts` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Bank::TotalIssuance` (r:1 w:1)
	/// Proof: `Bank::TotalIssuance` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn withdraw() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `635`
		//  Estimated: `6038`
		// Minimum execution time: 25_000_000 picoseconds.
		Weight::from_parts(25_000_000, 6038)
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	/// Storage: `Roles::AccountRoles` (r:2 w:0)
	/// Proof: `Roles::AccountRoles` (`max_values`: None, `max_size`: Some(49), added: 2524, mode: `MaxEncodedLen`)
	/// Storage: `Bank::Accounts` (r:2 w:2)
	/// Proof: `Bank::Accounts` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn transfer() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `674`
		//  Estimated: `6614`
		// Minimum execution time: 26_000_000 picoseconds.
		Weight::from_parts(27_000_000, 6614)
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	/// Storage: `Roles::AccountRoles` (r:1 w:0)
	/// Proof: `Roles::AccountRoles` (`max_values`: None, `max_size`: Some(49), added: 2524, mode: `MaxEncodedLen`)
	/// Storage: `Bank::Accounts` (r:1 w:1)
	/// Proof: `Bank::Accounts` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Bank::NextLockId` (r:1 w:1)
	/// Proof: `Bank::NextLockId` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Bank::AccountWithUnlockedFund` (r:1 w:1)
	/// Proof: `Bank::AccountWithUnlockedFund` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn stake_funds() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `577`
		//  Estimated: `4042`
		// Minimum execution time: 23_000_000 picoseconds.
		Weight::from_parts(24_000_000, 4042)
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
	}
	/// Storage: `Roles::AccountRoles` (r:1 w:0)
	/// Proof: `Roles::AccountRoles` (`max_values`: None, `max_size`: Some(49), added: 2524, mode: `MaxEncodedLen`)
	/// Storage: `Bank::Accounts` (r:1 w:1)
	/// Proof: `Bank::Accounts` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Bank::NextLockId` (r:1 w:1)
	/// Proof: `Bank::NextLockId` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Bank::AccountWithUnlockedFund` (r:1 w:1)
	/// Proof: `Bank::AccountWithUnlockedFund` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn redeem_funds() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `577`
		//  Estimated: `4042`
		// Minimum execution time: 23_000_000 picoseconds.
		Weight::from_parts(24_000_000, 4042)
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
	}
	/// Storage: `Roles::AccountRoles` (r:2 w:0)
	/// Proof: `Roles::AccountRoles` (`max_values`: None, `max_size`: Some(49), added: 2524, mode: `MaxEncodedLen`)
	/// Storage: `Bank::Accounts` (r:1 w:1)
	/// Proof: `Bank::Accounts` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Bank::NextLockId` (r:1 w:1)
	/// Proof: `Bank::NextLockId` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Bank::AccountWithUnlockedFund` (r:1 w:1)
	/// Proof: `Bank::AccountWithUnlockedFund` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn lock_funds_auditor() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `635`
		//  Estimated: `6038`
		// Minimum execution time: 27_000_000 picoseconds.
		Weight::from_parts(28_000_000, 6038)
			.saturating_add(T::DbWeight::get().reads(5_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
	}
	/// Storage: `Roles::AccountRoles` (r:2 w:0)
	/// Proof: `Roles::AccountRoles` (`max_values`: None, `max_size`: Some(49), added: 2524, mode: `MaxEncodedLen`)
	/// Storage: `Bank::Accounts` (r:1 w:1)
	/// Proof: `Bank::Accounts` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn unlock_funds_auditor() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `658`
		//  Estimated: `6038`
		// Minimum execution time: 22_000_000 picoseconds.
		Weight::from_parts(23_000_000, 6038)
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `Roles::AccountRoles` (r:1 w:0)
	/// Proof: `Roles::AccountRoles` (`max_values`: None, `max_size`: Some(49), added: 2524, mode: `MaxEncodedLen`)
	/// Storage: `Bank::InterestRate` (r:1 w:1)
	/// Proof: `Bank::InterestRate` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn set_interest_rate() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `362`
		//  Estimated: `3514`
		// Minimum execution time: 14_000_000 picoseconds.
		Weight::from_parts(15_000_000, 3514)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `Bank::Accounts` (r:2 w:1)
	/// Proof: `Bank::Accounts` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Roles::AccountRoles` (r:1 w:0)
	/// Proof: `Roles::AccountRoles` (`max_values`: None, `max_size`: Some(49), added: 2524, mode: `MaxEncodedLen`)
	/// Storage: `Bank::TreasuryAccount` (r:1 w:1)
	/// Proof: `Bank::TreasuryAccount` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Bank::AccountWithUnlockedFund` (r:1 w:0)
	/// Proof: `Bank::AccountWithUnlockedFund` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn rotate_treasury() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `522`
		//  Estimated: `6462`
		// Minimum execution time: 25_000_000 picoseconds.
		Weight::from_parts(26_000_000, 6462)
			.saturating_add(T::DbWeight::get().reads(5_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	/// Storage: `Roles::AccountRoles` (r:2 w:0)
	/// Proof: `Roles::AccountRoles` (`max_values`: None, `max_size`: Some(49), added: 2524, mode: `MaxEncodedLen`)
	/// Storage: `Bank::Accounts` (r:2 w:2)
	/// Proof: `Bank::Accounts` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn force_transfer() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `674`
		//  Estimated: `6614`
		// Minimum execution time: 26_000_000 picoseconds.
		Weight::from_parts(26_000_000, 6614)
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}


}

// For backwards compatibility and tests
impl WeightInfo for () {
	/// Storage: `Roles::AccountRoles` (r:2 w:0)
	/// Proof: `Roles::AccountRoles` (`max_values`: None, `max_size`: Some(49), added: 2524, mode: `MaxEncodedLen`)
	/// Storage: `Bank::Accounts` (r:1 w:1)
	/// Proof: `Bank::Accounts` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Bank::TotalIssuance` (r:1 w:1)
	/// Proof: `Bank::TotalIssuance` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn deposit() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `635`
		//  Estimated: `6038`
		// Minimum execution time: 24_000_000 picoseconds.
		Weight::from_parts(25_000_000, 6038)
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	/// Storage: `Roles::AccountRoles` (r:2 w:0)
	/// Proof: `Roles::AccountRoles` (`max_values`: None, `max_size`: Some(49), added: 2524, mode: `MaxEncodedLen`)
	/// Storage: `Bank::Accounts` (r:1 w:1)
	/// Proof: `Bank::Accounts` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Bank::TotalIssuance` (r:1 w:1)
	/// Proof: `Bank::TotalIssuance` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn withdraw() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `635`
		//  Estimated: `6038`
		// Minimum execution time: 25_000_000 picoseconds.
		Weight::from_parts(25_000_000, 6038)
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	/// Storage: `Roles::AccountRoles` (r:2 w:0)
	/// Proof: `Roles::AccountRoles` (`max_values`: None, `max_size`: Some(49), added: 2524, mode: `MaxEncodedLen`)
	/// Storage: `Bank::Accounts` (r:2 w:2)
	/// Proof: `Bank::Accounts` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn transfer() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `674`
		//  Estimated: `6614`
		// Minimum execution time: 26_000_000 picoseconds.
		Weight::from_parts(27_000_000, 6614)
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	/// Storage: `Roles::AccountRoles` (r:1 w:0)
	/// Proof: `Roles::AccountRoles` (`max_values`: None, `max_size`: Some(49), added: 2524, mode: `MaxEncodedLen`)
	/// Storage: `Bank::Accounts` (r:1 w:1)
	/// Proof: `Bank::Accounts` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Bank::NextLockId` (r:1 w:1)
	/// Proof: `Bank::NextLockId` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Bank::AccountWithUnlockedFund` (r:1 w:1)
	/// Proof: `Bank::AccountWithUnlockedFund` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn stake_funds() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `577`
		//  Estimated: `4042`
		// Minimum execution time: 23_000_000 picoseconds.
		Weight::from_parts(24_000_000, 4042)
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
	}
	/// Storage: `Roles::AccountRoles` (r:1 w:0)
	/// Proof: `Roles::AccountRoles` (`max_values`: None, `max_size`: Some(49), added: 2524, mode: `MaxEncodedLen`)
	/// Storage: `Bank::Accounts` (r:1 w:1)
	/// Proof: `Bank::Accounts` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Bank::NextLockId` (r:1 w:1)
	/// Proof: `Bank::NextLockId` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Bank::AccountWithUnlockedFund` (r:1 w:1)
	/// Proof: `Bank::AccountWithUnlockedFund` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn redeem_funds() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `577`
		//  Estimated: `4042`
		// Minimum execution time: 23_000_000 picoseconds.
		Weight::from_parts(24_000_000, 4042)
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
	}
	/// Storage: `Roles::AccountRoles` (r:2 w:0)
	/// Proof: `Roles::AccountRoles` (`max_values`: None, `max_size`: Some(49), added: 2524, mode: `MaxEncodedLen`)
	/// Storage: `Bank::Accounts` (r:1 w:1)
	/// Proof: `Bank::Accounts` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Bank::NextLockId` (r:1 w:1)
	/// Proof: `Bank::NextLockId` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Bank::AccountWithUnlockedFund` (r:1 w:1)
	/// Proof: `Bank::AccountWithUnlockedFund` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn lock_funds_auditor() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `635`
		//  Estimated: `6038`
		// Minimum execution time: 27_000_000 picoseconds.
		Weight::from_parts(28_000_000, 6038)
			.saturating_add(RocksDbWeight::get().reads(5_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
	}
	/// Storage: `Roles::AccountRoles` (r:2 w:0)
	/// Proof: `Roles::AccountRoles` (`max_values`: None, `max_size`: Some(49), added: 2524, mode: `MaxEncodedLen`)
	/// Storage: `Bank::Accounts` (r:1 w:1)
	/// Proof: `Bank::Accounts` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn unlock_funds_auditor() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `658`
		//  Estimated: `6038`
		// Minimum execution time: 22_000_000 picoseconds.
		Weight::from_parts(23_000_000, 6038)
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: `Roles::AccountRoles` (r:1 w:0)
	/// Proof: `Roles::AccountRoles` (`max_values`: None, `max_size`: Some(49), added: 2524, mode: `MaxEncodedLen`)
	/// Storage: `Bank::InterestRate` (r:1 w:1)
	/// Proof: `Bank::InterestRate` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn set_interest_rate() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `362`
		//  Estimated: `3514`
		// Minimum execution time: 14_000_000 picoseconds.
		Weight::from_parts(15_000_000, 3514)
			.saturating_add(RocksDbWeight::get().reads(2_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: `Bank::Accounts` (r:2 w:1)
	/// Proof: `Bank::Accounts` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Roles::AccountRoles` (r:1 w:0)
	/// Proof: `Roles::AccountRoles` (`max_values`: None, `max_size`: Some(49), added: 2524, mode: `MaxEncodedLen`)
	/// Storage: `Bank::TreasuryAccount` (r:1 w:1)
	/// Proof: `Bank::TreasuryAccount` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Bank::AccountWithUnlockedFund` (r:1 w:0)
	/// Proof: `Bank::AccountWithUnlockedFund` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn rotate_treasury() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `522`
		//  Estimated: `6462`
		// Minimum execution time: 25_000_000 picoseconds.
		Weight::from_parts(26_000_000, 6462)
			.saturating_add(RocksDbWeight::get().reads(5_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	/// Storage: `Roles::AccountRoles` (r:2 w:0)
	/// Proof: `Roles::AccountRoles` (`max_values`: None, `max_size`: Some(49), added: 2524, mode: `MaxEncodedLen`)
	/// Storage: `Bank::Accounts` (r:2 w:2)
	/// Proof: `Bank::Accounts` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn force_transfer() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `674`
		//  Estimated: `6614`
		// Minimum execution time: 26_000_000 picoseconds.
		Weight::from_parts(26_000_000, 6614)
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}

}
