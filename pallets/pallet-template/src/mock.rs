//! Mocks for the template module.

#![cfg(test)]

use super::*;
use frame_support::{
    construct_runtime, parameter_types,
    traits::{ConstU32, ConstU64, Everything},
};

use sp_runtime::{testing::H256, traits::IdentityLookup, BuildStorage};

use crate as pallet_template;

pub type AccountId = u32;

pub const ALICE: AccountId = 1;

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

parameter_types! {}

impl Config for Runtime {
    type RuntimeEvent = RuntimeEvent;

    type WeightInfo = ();
}

type Block = frame_system::mocking::MockBlock<Runtime>;

construct_runtime!(
    pub enum Runtime
    {
        System: frame_system,
        Template: pallet_template,
    }
);

#[derive(Default)]
pub struct MockGenesisConfig;

impl MockGenesisConfig {
    pub fn build(self) -> sp_io::TestExternalities {
        let config = RuntimeGenesisConfig {
            system: frame_system::GenesisConfig::default(),
            template: crate::GenesisConfig::default(),
        };

        let mut ext: sp_io::TestExternalities = config.build_storage().unwrap().into();

        ext.execute_with(|| {
            System::set_block_number(1);
        });

        ext
    }
}

// Build genesis storage according to the mock runtime.
pub fn default_test_ext() -> sp_io::TestExternalities {
    MockGenesisConfig::default().build()
}
