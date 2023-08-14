
#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

pub trait WeightInfo {
	fn example_extrinsic() -> Weight;
}

/// Default weights.
impl WeightInfo for () {
	fn example_extrinsic() -> Weight {
		Weight::from_parts(1_000_000, 0)
	}
}
