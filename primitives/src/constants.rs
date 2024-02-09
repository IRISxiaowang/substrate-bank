#![allow(dead_code)]

/// Defines all the constants for the project
use crate::{Balance, BlockNumber};

/// This determines the average expected block time that we are targeting.
/// Blocks will be produced at a minimum duration defined by `SLOT_DURATION`.
/// `SLOT_DURATION` is picked up by `pallet_timestamp` which is in turn picked
/// up by `pallet_aura` to implement `fn slot_duration()`.
///
/// Change this to adjust the block time.
pub const MILLISECS_PER_BLOCK: u64 = 6000;

// NOTE: Currently it is not possible to change the slot duration after the chain has started.
//       Attempting to do so will brick block production.
pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

// Time is measured by number of blocks.
pub const MINUTE: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOUR: BlockNumber = MINUTE * 60;
pub const DAY: BlockNumber = HOUR * 24;
pub const YEAR: BlockNumber = DAY * 365;

pub const DOLLAR: Balance = 1_000_000_000_000u128;
pub const CENT: Balance = 10_000_000_000u128;
pub const MILLICENT: Balance = 10_000_000u128;
