#![cfg(test)]

use super::*;
use frame_support::{
	construct_runtime, derive_impl, parameter_types,
	traits::{ConstU32, ConstU64, Everything},
};

use sp_runtime::{testing::H256, traits::IdentityLookup, BuildStorage};

use crate as pallet_roles;

pub type AccountId = u32;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const CHARLIE: AccountId = 3;

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

parameter_types! {}

impl Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type EnsureGovernance = traits::SuccessOrigin<Runtime>;
}

type Block = frame_system::mocking::MockBlock<Runtime>;

construct_runtime!(
	pub enum Runtime
	{
		System: frame_system,
		Roles: pallet_roles,
	}
);

#[derive(Default)]
pub struct MockGenesisConfig {
	roles: Vec<(AccountId, Role)>,
}

impl MockGenesisConfig {
	pub fn with_roles(mut self, roles: Vec<(AccountId, Role)>) -> Self {
		self.roles = roles;
		self
	}
	pub fn build(self) -> sp_io::TestExternalities {
		let config = RuntimeGenesisConfig {
			system: frame_system::GenesisConfig::default(),
			roles: crate::GenesisConfig { roles: self.roles },
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
