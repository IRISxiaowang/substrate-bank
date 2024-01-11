#![allow(dead_code)]

/// Defines all the constants for the project
use crate::BlockNumber;

// 6 seconds per block, same as Polkadot
pub const SECONDS_PER_BLOCK: BlockNumber = 6u32;

// approx. number of seconds in a year: 365 * 24 * 3600
pub const SECONDS_PER_YEAR: BlockNumber = 31_536_000u32;

pub const DAY: BlockNumber = 86_400 / SECONDS_PER_BLOCK;
pub const YEAR: BlockNumber = SECONDS_PER_YEAR / SECONDS_PER_BLOCK;
