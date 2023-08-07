//! Mocks for the tokens module.

#![cfg(test)]

use super::*;
use frame_support::{
    construct_runtime,
    traits::{ConstU32, ConstU64, Everything},
    PalletId,
};

use sp_runtime::{testing::H256, traits::IdentityLookup, BuildStorage};

use crate as PalletBank;

pub type AccountId = u32;
pub type Balance = u128;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;

impl frame_system::Config for Runtime {
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Hash = H256;
    type Hashing = ::sp_runtime::traits::BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = ConstU64<250>;
    type BlockWeights = ();
    type BlockLength = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type DbWeight = ();
    type BaseCallFilter = Everything;
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
    type Nonce = u128;
    type Block = Block;
}

impl Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Balance = Balance;
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system,
        Bank: PalletBank,
    }
);

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let config = GenesisConfig::default();

    let mut ext: sp_io::TestExternalities = config.build_storage().unwrap().into();

    ext.execute_with(|| {
        System::set_block_number(1);
    });

    ext
}
