
//! Autogenerated weights for pallet_lottery
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2024-02-07, STEPS: `20`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
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
// pallet_lottery
// --output
// ./pallets/pallet-lottery/src/weights.rs
// --steps=20
// --repeat=20
// --template=weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

/// Weight functions needed for pallet_lottery.
pub trait WeightInfo {
	fn set_prize_split() -> Weight;
	fn update_ticket_price() -> Weight;
	fn buy_ticket() -> Weight;
}

/// Weights for pallet_lottery using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	/// Storage: `Lottery::PrizeSplit` (r:0 w:1)
	/// Proof: `Lottery::PrizeSplit` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn set_prize_split() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 5_000_000 picoseconds.
		Weight::from_parts(6_000_000, 0)
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `Roles::AccountRoles` (r:1 w:0)
	/// Proof: `Roles::AccountRoles` (`max_values`: None, `max_size`: Some(49), added: 2524, mode: `MaxEncodedLen`)
	/// Storage: `Lottery::TicketPrice` (r:1 w:1)
	/// Proof: `Lottery::TicketPrice` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn update_ticket_price() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `320`
		//  Estimated: `3514`
		// Minimum execution time: 14_000_000 picoseconds.
		Weight::from_parts(15_000_000, 3514)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `Roles::AccountRoles` (r:1 w:0)
	/// Proof: `Roles::AccountRoles` (`max_values`: None, `max_size`: Some(49), added: 2524, mode: `MaxEncodedLen`)
	/// Storage: `Lottery::TicketPrice` (r:1 w:0)
	/// Proof: `Lottery::TicketPrice` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Bank::Accounts` (r:2 w:2)
	/// Proof: `Bank::Accounts` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Lottery::PlayersAndLotteries` (r:1 w:1)
	/// Proof: `Lottery::PlayersAndLotteries` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn buy_ticket() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `610`
		//  Estimated: `6550`
		// Minimum execution time: 32_000_000 picoseconds.
		Weight::from_parts(32_000_000, 6550)
			.saturating_add(T::DbWeight::get().reads(5_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	/// Storage: `Lottery::PrizeSplit` (r:0 w:1)
	/// Proof: `Lottery::PrizeSplit` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn set_prize_split() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 5_000_000 picoseconds.
		Weight::from_parts(6_000_000, 0)
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: `Roles::AccountRoles` (r:1 w:0)
	/// Proof: `Roles::AccountRoles` (`max_values`: None, `max_size`: Some(49), added: 2524, mode: `MaxEncodedLen`)
	/// Storage: `Lottery::TicketPrice` (r:1 w:1)
	/// Proof: `Lottery::TicketPrice` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn update_ticket_price() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `320`
		//  Estimated: `3514`
		// Minimum execution time: 14_000_000 picoseconds.
		Weight::from_parts(15_000_000, 3514)
			.saturating_add(RocksDbWeight::get().reads(2_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: `Roles::AccountRoles` (r:1 w:0)
	/// Proof: `Roles::AccountRoles` (`max_values`: None, `max_size`: Some(49), added: 2524, mode: `MaxEncodedLen`)
	/// Storage: `Lottery::TicketPrice` (r:1 w:0)
	/// Proof: `Lottery::TicketPrice` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Bank::Accounts` (r:2 w:2)
	/// Proof: `Bank::Accounts` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Lottery::PlayersAndLotteries` (r:1 w:1)
	/// Proof: `Lottery::PlayersAndLotteries` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn buy_ticket() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `610`
		//  Estimated: `6550`
		// Minimum execution time: 32_000_000 picoseconds.
		Weight::from_parts(32_000_000, 6550)
			.saturating_add(RocksDbWeight::get().reads(5_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
	}
}
