//! Mocks for the tokens module.

#![cfg(test)]

use super::*;
use frame_support::{
	construct_runtime, derive_impl, parameter_types,
	traits::{ConstU32, ConstU64, Everything},
};

use primitives::YEAR;
use sp_runtime::{testing::H256, traits::IdentityLookup, BuildStorage};

use crate as pallet_bank;

pub type AccountId = u32;
pub type Balance = u128;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const TREASURY: AccountId = 0;
pub const ED: u128 = 3u128;
pub const MIN: u128 = 5u128;
pub const INITIAL_BALANCE: u128 = 1_000_000u128;
pub const REDEEM_PERIOD: u64 = 200;
pub const STAKE_PERIOD: u64 = 150;
pub const INTEREST_PAYOUT_PERIOD: u64 = 100;

type Block = frame_system::mocking::MockBlock<Runtime>;
type BlockNumber = u64;

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
	pub const ExistentialDeposit: Balance = ED;
	pub const MinimumAmount: Balance = MIN;
	pub const RedeemPeriod: BlockNumber = REDEEM_PERIOD;
	pub const StakePeriod: BlockNumber = STAKE_PERIOD;
	pub const InterestPayoutPeriod: BlockNumber = INTEREST_PAYOUT_PERIOD;
	pub const TotalBlocksPerYear: BlockNumber = YEAR as BlockNumber;
}

impl Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type Balance = Balance;
	type RoleManager = Roles;
	type EnsureGovernance = traits::SuccessOrigin<Runtime>;
	type ExistentialDeposit = ExistentialDeposit;
	type MinimumAmount = MinimumAmount;
	type RedeemPeriod = RedeemPeriod;
	type StakePeriod = StakePeriod;
	type InterestPayoutPeriod = InterestPayoutPeriod;
	type TotalBlocksPerYear = TotalBlocksPerYear;
}

impl pallet_roles::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type EnsureGovernance = traits::SuccessOrigin<Runtime>;
}

construct_runtime!(
	pub enum Runtime
	{
		System: frame_system,
		Bank: pallet_bank,
		Roles: pallet_roles,
	}
);

#[derive(Default)]
pub struct MockGenesisConfig {
	balances: Vec<(AccountId, Balance, Balance)>,
}

impl MockGenesisConfig {
	pub fn with_balances(mut self, balances: Vec<(AccountId, Balance, Balance)>) -> Self {
		self.balances = balances;
		self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let mut endowed = self.balances;
		endowed.push((TREASURY, INITIAL_BALANCE, 0u128));

		let roles = endowed.iter().map(|(id, _, _)| (*id, Role::Customer)).collect();
		let config = RuntimeGenesisConfig {
			system: frame_system::GenesisConfig::default(),
			bank: crate::GenesisConfig { balances: endowed, treasury: None },
			roles: pallet_roles::GenesisConfig { roles },
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
