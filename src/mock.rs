//! Mocks for the tokens module.

#![cfg(test)]

use super::*;
use frame_support::{
    construct_runtime,
    traits::{ConstU32, ConstU64, Everything}, parameter_types,
};

use sp_runtime::{testing::H256, traits::IdentityLookup, BuildStorage};

use crate as pallet_bank;

pub type AccountId = u32;
pub type Balance = u128;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const TREASURY: AccountId = 0;

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

parameter_types! {
    pub const ExistentialDeposit: Balance = 3u128;
    pub const TreasuryAccount: AccountId = TREASURY;
    pub const MinimumAmount: Balance = 5u128;
}

impl Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Balance = Balance;
    type ExistentialDeposit = ExistentialDeposit;
    type TreasuryAccount = TreasuryAccount;
    type MinimumAmount = MinimumAmount;
}

type Block = frame_system::mocking::MockBlock<Runtime>;

construct_runtime!(
    pub enum Runtime
    {
        System: frame_system,
        Bank: pallet_bank,
    }
);

#[derive(Default)]
pub struct MockGenesisConfig {
    balances: Vec<(AccountId, Balance)>,
}

impl MockGenesisConfig {
    pub fn with_balances(balances: Vec<(AccountId, Balance)>) -> Self {
        Self { balances }
    }

    pub fn build(self) -> sp_io::TestExternalities {
        let mut endowed = self.balances;
        endowed.push((TREASURY, 1_000_000));
        let config = RuntimeGenesisConfig {
            system: frame_system::GenesisConfig::default(),
            bank: crate::GenesisConfig {
                balances: endowed,
            },
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