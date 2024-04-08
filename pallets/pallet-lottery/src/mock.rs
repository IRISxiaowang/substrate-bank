#![cfg(test)]

use super::*;
use frame_support::{
	assert_ok, construct_runtime, derive_impl, parameter_types,
	traits::{ConstU32, ConstU64, Everything, Randomness},
};

use primitives::{NftId, DOLLAR, YEAR};
use sp_runtime::{testing::H256, traits::IdentityLookup, BuildStorage};
use traits::ManageNfts;

use crate as pallet_lottery;

pub type AccountId = u32;
pub type Balance = u128;

type BlockNumber = u64;
type Block = frame_system::mocking::MockBlock<Runtime>;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const CHARLIE: AccountId = 3;
pub const DAVE: AccountId = 4;
pub const EVE: AccountId = 5;
pub const FERDIE: AccountId = 6;

pub const LOTTERY_PAYOUT_PERIOD: u64 = 100;
pub const PRIZE_POOL_ACCOUNT: AccountId = 10;
pub const TAX_RATE: Percent = Percent::from_percent(5);

pub const TREASURY: AccountId = 0;
pub const ED: u128 = 3u128;
pub const MIN: u128 = 5u128;
pub const REDEEM_PERIOD: u64 = 200;
pub const STAKE_PERIOD: u64 = 150;
pub const INTEREST_PAYOUT_PERIOD: u64 = 100;

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
	pub static RandomOutput: u32 = Default::default();
}

pub struct MockRandom;
impl Randomness<H256, BlockNumber> for MockRandom {
	fn random(_subject: &[u8]) -> (H256, BlockNumber) {
		let mut bytes = [0u8; 32];
		bytes[..4].copy_from_slice(&RandomOutput::get().to_ne_bytes());

		(bytes.into(), Default::default())
	}
}

parameter_types! {
	pub const LotteryPayoutPeriod: BlockNumber = LOTTERY_PAYOUT_PERIOD;
	pub const PrizePoolAccount: AccountId = PRIZE_POOL_ACCOUNT;
	pub const TaxRate: Percent = TAX_RATE;
}

impl Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type Balance = Balance;
	type RoleManager = Roles;
	type BlockNumberProvider = System;
	type EnsureGovernance = traits::SuccessOrigin<Runtime>;
	type Bank = Bank;
	type Randomness = MockRandom;
	type LotteryPayoutPeriod = LotteryPayoutPeriod;
	type PrizePoolAccount = PrizePoolAccount;
	type TaxRate = TaxRate;
}

pub struct MockNftManager;
impl ManageNfts<AccountId> for MockNftManager {
	fn nft_transfer(
		_from_user: &AccountId,
		_to_user: &AccountId,
		_nft_id: NftId,
	) -> DispatchResult {
		unimplemented!()
	}
	fn ensure_nft_is_valid(_id: &AccountId, _nft_id: NftId) -> DispatchResult {
		unimplemented!()
	}
	fn owner(_nft_id: NftId) -> Option<AccountId> {
		unimplemented!()
	}
}

parameter_types! {
	pub const ExistentialDeposit: Balance = ED;
	pub const MinimumAmount: Balance = MIN;
	pub const RedeemPeriod: BlockNumber = REDEEM_PERIOD;
	pub const StakePeriod: BlockNumber = STAKE_PERIOD;
	pub const InterestPayoutPeriod: BlockNumber = INTEREST_PAYOUT_PERIOD;
	pub const TotalBlocksPerYear: BlockNumber = YEAR as BlockNumber;
}

impl pallet_bank::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type Balance = Balance;
	type RoleManager = Roles;
	type BlockNumberProvider = System;
	type EnsureGovernance = traits::SuccessOrigin<Runtime>;
	type NftManager = MockNftManager;
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
		Lottery: pallet_lottery,
		Bank: pallet_bank,
		Roles: pallet_roles,
	}
);

#[derive(Default)]
pub struct MockGenesisConfig {
	pub(crate) balance: Vec<(AccountId, Balance)>,
	pub(crate) lotteries: Vec<(AccountId, u32)>,
}

impl MockGenesisConfig {
	pub fn with_lotteries(self, lotteries: Vec<(AccountId, u32)>) -> Self {
		Self { balance: self.balance, lotteries }
	}

	pub fn with_balances(self, balance: Vec<(AccountId, Balance)>) -> Self {
		Self { balance, lotteries: self.lotteries }
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let config = RuntimeGenesisConfig {
			system: Default::default(),
			lottery: Default::default(),
			bank: pallet_bank::GenesisConfig {
				balances: self
					.balance
					.into_iter()
					.map(|(account, free)| (account, free, 0u128))
					.collect::<Vec<_>>(),
			},
			roles: pallet_roles::GenesisConfig {
				roles: vec![
					(ALICE, Role::Customer),
					(BOB, Role::Customer),
					(CHARLIE, Role::Customer),
					(DAVE, Role::Customer),
					(EVE, Role::Manager),
					(FERDIE, Role::Auditor),
					(PRIZE_POOL_ACCOUNT, Role::Customer),
				],
			},
		};

		let mut ext: sp_io::TestExternalities = config.build_storage().unwrap().into();

		ext.execute_with(|| {
			System::set_block_number(1);

			assert_ok!(Bank::rotate_treasury(RuntimeOrigin::root(), TREASURY));
			assert_ok!(Lottery::update_ticket_price(RuntimeOrigin::signed(EVE), DOLLAR));

			self.lotteries.into_iter().for_each(|(user, tickets)| {
				assert_ok!(Bank::deposit(
					RuntimeOrigin::signed(EVE),
					PRIZE_POOL_ACCOUNT,
					DOLLAR * tickets as u128
				));
				TicketsBought::<Runtime>::set(user, tickets);
			});
		});

		ext
	}
}

// Build genesis storage according to the mock runtime.
pub fn default_test_ext() -> sp_io::TestExternalities {
	MockGenesisConfig::default().build()
}
