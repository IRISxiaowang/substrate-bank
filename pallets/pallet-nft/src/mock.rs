#![cfg(test)]

use super::*;
use frame_support::{
	construct_runtime, derive_impl, parameter_types,
	traits::{ConstU32, ConstU64, Everything},
};

use sp_runtime::{testing::H256, traits::IdentityLookup, BuildStorage};

use primitives::Balance;

use crate as pallet_nft;

pub type AccountId = u32;

type Block = frame_system::mocking::MockBlock<Runtime>;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const FERDIE: AccountId = 6;
pub const TREASURY: AccountId = 255;

pub const MAX_SIZE: u32 = 1_000u32;
pub const NFT_LOCKED_PERIOD: u64 = 100;
pub const FEE: u128 = 1u128;

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
	pub static TransferHistory: Vec<(AccountId, AccountId, Balance)> = Default::default();
}

pub struct MockBank;
impl BasicAccounting<AccountId, Balance> for MockBank {
	fn deposit(_user: &AccountId, _amount: Balance) -> DispatchResult {
		unimplemented!();
	}
	fn withdraw(_user: &AccountId, _amount: Balance) -> DispatchResult {
		unimplemented!();
	}
	fn transfer(from: &AccountId, to: &AccountId, amount: Balance) -> DispatchResult {
		let mut history = TransferHistory::get();
		history.push((*from, *to, amount));
		TransferHistory::set(history);
		Ok(())
	}
	fn free_balance(_user: &AccountId) -> Balance {
		unimplemented!();
	}
}
impl GetTreasury<AccountId> for MockBank {
	fn treasury() -> Result<AccountId, DispatchError> {
		Ok(TREASURY)
	}
}

parameter_types! {
	pub const MaxSize: u32 = MAX_SIZE;
	pub const Fee: u128 = FEE;
	pub const NftLockedPeriod: u64 = NFT_LOCKED_PERIOD;

}

impl Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type RoleManager = Roles;
	type Balance = Balance;
	type Bank = MockBank;
	type EnsureGovernance = traits::SuccessOrigin<Runtime>;
	type MaxSize = MaxSize;
	type Fee = Fee;
	type NftLockedPeriod = NftLockedPeriod;
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
		Nft: pallet_nft,
		Roles: pallet_roles,
	}
);

#[derive(Default)]
pub struct MockGenesisConfig {}

impl MockGenesisConfig {
	pub fn build(self) -> sp_io::TestExternalities {
		let config = RuntimeGenesisConfig {
			system: Default::default(),

			roles: pallet_roles::GenesisConfig {
				roles: vec![
					(ALICE, Role::Customer),
					(BOB, Role::Customer),
					(FERDIE, Role::Auditor),
				],
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
