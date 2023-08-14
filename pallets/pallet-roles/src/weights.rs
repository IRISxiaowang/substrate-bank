
#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

pub trait WeightInfo {
	fn deposit() -> Weight;
}

/// Default weights.
impl WeightInfo for () {
	fn deposit() -> Weight {
		Weight::from_parts(1, 0)
	}
}
