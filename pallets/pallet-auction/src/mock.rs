#![cfg(test)]

use super::*;
use frame_support::{
	construct_runtime, derive_impl, parameter_types,
	traits::{ConstU32, ConstU64, Everything},
};

use sp_runtime::{testing::H256, traits::IdentityLookup, BuildStorage};

use primitives::{Balance, Role};

use crate as pallet_auction;

pub type AccountId = u32;

type Block = frame_system::mocking::MockBlock<Runtime>;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const FERDIE: AccountId = 6;
pub const TREASURY: AccountId = 255;

pub const MAX_SIZE: u32 = 1_000u32;
pub const NFT_LOCKED_PERIOD: u64 = 100;
pub const FEE: u128 = 1u128;

pub const BIDS_POOL_ACCOUNT: AccountId = 100;
pub const AUCTION_SUCCESS_FEE_PERCENTAGE: Percent = Percent::from_percent(10);
pub const AUCTION_START_FEE: u128 = 50u128;
pub const MINIMUM_INCREASE: u128 = 10u128;
pub const AUCTION_LENGTH: u64 = 200;
pub const EXTENDED_LENGTH: u64 = 10;

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
		Ok(())
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
	pub const BidsPoolAccount: AccountId = BIDS_POOL_ACCOUNT;
	pub const AuctionSuccessFeePercentage: Percent = AUCTION_SUCCESS_FEE_PERCENTAGE;
	pub const AuctionStartFee: u128 = AUCTION_START_FEE;
	pub const MinimumIncrease: u128 = MINIMUM_INCREASE;
	pub const AuctionLength: u64 = AUCTION_LENGTH;
	pub const ExtendedLength: u64 = EXTENDED_LENGTH;
}

impl Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type RoleManager = Roles;
	type Balance = Balance;
	type Bank = MockBank;
	type NftManager = Nft;
	type BidsPoolAccount = BidsPoolAccount;
	type AuctionSuccessFeePercentage = AuctionSuccessFeePercentage;
	type AuctionStartFee = AuctionStartFee;
	type MinimumIncrease = MinimumIncrease;
	type AuctionLength = AuctionLength;
	type ExtendedLength = ExtendedLength;
}

parameter_types! {
	pub const MaxSize: u32 = MAX_SIZE;
	pub const Fee: u128 = FEE;
	pub const NftLockedPeriod: u64 = NFT_LOCKED_PERIOD;

}

impl pallet_nft::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type RoleManager = Roles;
	type Balance = Balance;
	type Bank = MockBank;
	type EnsureGovernance = traits::SuccessOrigin<Runtime>;
	type MaxSize = MaxSize;
	type PodFee = Fee;
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
		Auction: pallet_auction,
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
			let file_name = vec![0x46, 0x49, 0x4C, 0x45];
			let data = vec![0x4E, 0x46, 0x54];
			let nft_id = Nft::next_nft() + 1;
			let _ = Nft::request_mint(RuntimeOrigin::signed(ALICE), file_name, data);
			let _ = Nft::approve_nft(
				RuntimeOrigin::signed(FERDIE),
				nft_id,
				primitives::Response::Accept,
			);
		});

		ext
	}
}

// Build genesis storage according to the mock runtime.
pub fn default_test_ext() -> sp_io::TestExternalities {
	MockGenesisConfig::default().build()
}
