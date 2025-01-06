#![cfg(test)]

use super::*;
use frame_support::{
	construct_runtime, derive_impl, parameter_types,
	traits::{ConstU32, ConstU64, Everything},
};

use sp_runtime::{testing::H256, traits::IdentityLookup, BuildStorage};

use crate as pallet_governance;

pub type AccountId = u32;

type BlockNumber = u64;
type Block = frame_system::mocking::MockBlock<Runtime>;

pub const EXPIRY_PERIOD: u64 = 100;
pub const MAJORITY_THRESHOLD: Percent = Percent::from_percent(80);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig as frame_system::DefaultConfig)]
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
	pub const ExpiryPeriod: BlockNumber = EXPIRY_PERIOD;
	pub const MajorityThreshold: Percent = MAJORITY_THRESHOLD;
}

impl Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type BlockNumberProvider = System;
	type ExpiryPeriod = ExpiryPeriod;
	type MajorityThreshold = MajorityThreshold;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type EnsureGovernance = crate::EnsureGovernance;
}

construct_runtime!(
	pub enum Runtime
	{
		System: frame_system,
		Governance: pallet_governance,
	}
);

#[derive(Default)]
pub struct MockGenesisConfig {
	pub authorities: Vec<AccountId>,
}

impl MockGenesisConfig {
	pub fn with_authorities(mut self, authorities: Vec<AccountId>) -> Self {
		self.authorities = authorities;
		self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let config = RuntimeGenesisConfig {
			system: Default::default(),
			governance: crate::GenesisConfig { initial_authorities: self.authorities },
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
