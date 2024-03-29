
//! Autogenerated weights for pallet_roles
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
// pallet_roles
// --output
// ./pallets/pallet-roles/src/weights.rs
// --steps=20
// --repeat=20
// --template=weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

/// Weight functions needed for pallet_roles.
pub trait WeightInfo {
	fn register_customer() -> Weight;
	fn unregister() -> Weight;
	fn register_role_governance() -> Weight;
}

/// Weights for pallet_roles using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	/// Storage: `Roles::AccountRoles` (r:1 w:1)
	/// Proof: `Roles::AccountRoles` (`max_values`: None, `max_size`: Some(49), added: 2524, mode: `MaxEncodedLen`)
	fn register_customer() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `216`
		//  Estimated: `3514`
		// Minimum execution time: 11_000_000 picoseconds.
		Weight::from_parts(12_000_000, 3514)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `Roles::AccountRoles` (r:1 w:1)
	/// Proof: `Roles::AccountRoles` (`max_values`: None, `max_size`: Some(49), added: 2524, mode: `MaxEncodedLen`)
	fn unregister() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `253`
		//  Estimated: `3514`
		// Minimum execution time: 12_000_000 picoseconds.
		Weight::from_parts(13_000_000, 3514)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `Roles::AccountRoles` (r:1 w:1)
	/// Proof: `Roles::AccountRoles` (`max_values`: None, `max_size`: Some(49), added: 2524, mode: `MaxEncodedLen`)
	fn register_role_governance() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `216`
		//  Estimated: `3514`
		// Minimum execution time: 11_000_000 picoseconds.
		Weight::from_parts(12_000_000, 3514)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	/// Storage: `Roles::AccountRoles` (r:1 w:1)
	/// Proof: `Roles::AccountRoles` (`max_values`: None, `max_size`: Some(49), added: 2524, mode: `MaxEncodedLen`)
	fn register_customer() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `216`
		//  Estimated: `3514`
		// Minimum execution time: 11_000_000 picoseconds.
		Weight::from_parts(12_000_000, 3514)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: `Roles::AccountRoles` (r:1 w:1)
	/// Proof: `Roles::AccountRoles` (`max_values`: None, `max_size`: Some(49), added: 2524, mode: `MaxEncodedLen`)
	fn unregister() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `253`
		//  Estimated: `3514`
		// Minimum execution time: 12_000_000 picoseconds.
		Weight::from_parts(13_000_000, 3514)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: `Roles::AccountRoles` (r:1 w:1)
	/// Proof: `Roles::AccountRoles` (`max_values`: None, `max_size`: Some(49), added: 2524, mode: `MaxEncodedLen`)
	fn register_role_governance() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `216`
		//  Estimated: `3514`
		// Minimum execution time: 11_000_000 picoseconds.
		Weight::from_parts(12_000_000, 3514)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
}
