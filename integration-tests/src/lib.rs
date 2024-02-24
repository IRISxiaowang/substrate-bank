#![cfg(test)]

use frame_support::{assert_ok, traits::Hooks};
use sp_runtime::BuildStorage;

use primitives::{constants::*, AccountId, Balance, Role};
use traits::{BasicAccounting, Stakable};

#[allow(unused_imports)]
use xy_chain_runtime::{
	Bank, Governance, Lottery, Roles, Runtime, RuntimeError, RuntimeEvent, RuntimeOrigin, System,
	Timestamp,
};

mod bank;

pub const INITIAL_BALANCE: Balance = 1_000_000 * DOLLAR;

pub const MANAGER: [u8; 32] = [0xFF; 32];
pub const AUDITOR: [u8; 32] = [0xFE; 32];
pub const ALICE: [u8; 32] = [0x00; 32];
pub const BOB: [u8; 32] = [0x01; 32];
pub const CHARLIE: [u8; 32] = [0x02; 32];
pub const DAVE: [u8; 32] = [0x03; 32];
pub const EVE: [u8; 32] = [0x04; 32];

pub const GOV_0: [u8; 32] = [0xF0; 32];
pub const GOV_1: [u8; 32] = [0xF1; 32];
pub const GOV_2: [u8; 32] = [0xF2; 32];

// TODO: use macro to convert these [] into AccountIds.
// i.e. alice() will return AccountId([0x00; 32]);
// Issue #52

pub struct ExtBuilder {
	governance_members: Vec<AccountId>,
	balances: Vec<(AccountId, Balance)>,
	roles: Vec<(AccountId, Role)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			governance_members: vec![GOV_0.into(), GOV_1.into(), GOV_2.into()],
			balances: vec![
				(ALICE.into(), INITIAL_BALANCE),
				(BOB.into(), INITIAL_BALANCE),
				(CHARLIE.into(), INITIAL_BALANCE),
				(DAVE.into(), INITIAL_BALANCE),
				(EVE.into(), INITIAL_BALANCE),
			],
			roles: vec![
				(MANAGER.into(), Role::Manager),
				(AUDITOR.into(), Role::Auditor),
				(ALICE.into(), Role::Customer),
				(BOB.into(), Role::Customer),
				(CHARLIE.into(), Role::Customer),
				(DAVE.into(), Role::Customer),
				(EVE.into(), Role::Customer),
			],
		}
	}
}

impl ExtBuilder {
	pub fn governance_members(mut self, members: Vec<AccountId>) -> Self {
		self.governance_members = members;
		self
	}

	pub fn balances(mut self, balances: Vec<(AccountId, Balance)>) -> Self {
		self.balances = balances;
		self
	}

	pub fn roles(mut self, roles: Vec<(AccountId, Role)>) -> Self {
		self.roles = roles;
		self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		// Build storage
		let mut t = frame_system::GenesisConfig::<Runtime>::default().build_storage().unwrap();

		pallet_bank::GenesisConfig::<Runtime> { balances: self.balances }
			.assimilate_storage(&mut t)
			.unwrap();

		pallet_roles::GenesisConfig::<Runtime> { roles: self.roles }
			.assimilate_storage(&mut t)
			.unwrap();

		pallet_lottery::GenesisConfig::<Runtime>::default()
			.assimilate_storage(&mut t)
			.unwrap();

		pallet_governance::GenesisConfig::<Runtime> {
			initial_authorities: self.governance_members,
		}
		.assimilate_storage(&mut t)
		.unwrap();

		let mut ext = sp_io::TestExternalities::new(t.build_storage().unwrap());

		ext.execute_with(|| {
			System::set_block_number(1);
		});

		ext
	}
}

// Contains test utility functions
#[track_caller]
pub fn assert_balance(who: AccountId, amount: Balance) {
	assert_eq!(Bank::free_balance(&who), amount);
}

#[track_caller]
pub fn assert_staked(who: AccountId, amount: Balance) {
	assert_eq!(Bank::staked(&who), amount);
}
